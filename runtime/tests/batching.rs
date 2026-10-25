mod support;

use std::{any::Any, cell::RefCell, collections::HashMap};

use placeholder_query_runtime::{DataSource, Env, Request, fetch, traverse};

use support::executor::run;

#[derive(Clone, Debug, PartialEq, Eq)]
struct User {
    id: i32,
    name: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UserCard {
    id: i32,
    display_name: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Post {
    id: i32,
    author_id: i32,
}

#[derive(Default)]
struct TestBackend {
    users: Vec<User>,
    user_cards: Vec<UserCard>,
    posts: Vec<Post>,
    reqs: RefCell<Vec<Box<dyn Any>>>,
}

impl Env for TestBackend {
    type Error = ();
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserById(i32);

impl Request for UserById {
    type Output = Option<User>;
}

impl DataSource<UserById> for TestBackend {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a UserById>,
    ) -> Result<HashMap<UserById, Option<User>>, ()> {
        let mut results = HashMap::new();
        for req in reqs {
            let user = self.users.iter().find(|user| user.id == req.0).cloned();
            assert!(results.insert(req.clone(), user).is_none());
        }

        self.reqs.borrow_mut().push(Box::new(results.clone()));

        Ok(results)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserCardById(i32);

impl Request for UserCardById {
    type Output = Option<UserCard>;
}

impl DataSource<UserCardById> for TestBackend {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a UserCardById>,
    ) -> Result<HashMap<UserCardById, Option<UserCard>>, ()> {
        let mut results = HashMap::new();
        for req in reqs {
            let card = self
                .user_cards
                .iter()
                .find(|card| card.id == req.0)
                .cloned();
            assert!(results.insert(req.clone(), card).is_none());
        }

        self.reqs.borrow_mut().push(Box::new(results.clone()));

        Ok(results)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FeedPosts;

impl Request for FeedPosts {
    type Output = Vec<Post>;
}

impl DataSource<FeedPosts> for TestBackend {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a FeedPosts>,
    ) -> Result<HashMap<FeedPosts, Vec<Post>>, ()> {
        let mut results = HashMap::new();
        for req in reqs {
            assert!(results.insert(req.clone(), self.posts.clone()).is_none());
        }

        self.reqs.borrow_mut().push(Box::new(results.clone()));

        Ok(results)
    }
}

#[test]
fn deduplication_preserves_results_for_each_input() {
    let backend = TestBackend {
        users: vec![
            User { id: 1, name: "Mio" },
            User {
                id: 2,
                name: "Ritsu",
            },
        ],
        ..Default::default()
    };
    let users = run(backend.run(traverse([2, 1, 2, 1], |id| fetch(UserById(id))))).unwrap();

    let reqs = backend.reqs.into_inner();
    assert_eq!(reqs.len(), 1);
    let batch = reqs[0]
        .downcast_ref::<HashMap<UserById, Option<User>>>()
        .unwrap();

    assert_eq!(batch.len(), 2);
    assert!(batch.contains_key(&UserById(1)));
    assert!(batch.contains_key(&UserById(2)));
    assert_eq!(
        users,
        vec![
            Some(User {
                id: 2,
                name: "Ritsu"
            }),
            Some(User { id: 1, name: "Mio" }),
            Some(User {
                id: 2,
                name: "Ritsu"
            }),
            Some(User { id: 1, name: "Mio" }),
        ]
    );
}

#[test]
fn different_request_types_are_fetched_separately() {
    let backend = TestBackend {
        users: vec![User { id: 7, name: "Mio" }],
        user_cards: vec![UserCard {
            id: 7,
            display_name: "Mio",
        }],
        ..Default::default()
    };
    let result = run(backend.run(fetch(UserById(7)).zip(fetch(UserCardById(7))))).unwrap();

    let reqs = backend.reqs.borrow();
    assert_eq!(reqs.len(), 2);
    assert!(reqs.iter().any(|batch| {
        batch
            .downcast_ref::<HashMap<UserById, Option<User>>>()
            .is_some_and(|batch| batch.len() == 1 && batch.contains_key(&UserById(7)))
    }));
    assert!(reqs.iter().any(|batch| {
        batch
            .downcast_ref::<HashMap<UserCardById, Option<UserCard>>>()
            .is_some_and(|batch| batch.len() == 1 && batch.contains_key(&UserCardById(7)))
    }));
    assert_eq!(
        result,
        (
            Some(User { id: 7, name: "Mio" }),
            Some(UserCard {
                id: 7,
                display_name: "Mio"
            }),
        )
    );
}

#[test]
fn cached_results_are_reused_across_dependent_computations() {
    let backend = TestBackend {
        users: vec![User { id: 7, name: "Mio" }],
        ..Default::default()
    };
    let computation = fetch(UserById(7)).and_then(|_| fetch(UserById(7)));
    let result = run(backend.run(computation)).unwrap();

    let reqs = backend.reqs.borrow();
    assert_eq!(reqs.len(), 1);
    let batch = reqs[0]
        .downcast_ref::<HashMap<UserById, Option<User>>>()
        .unwrap();
    assert_eq!(batch.len(), 1);
    assert!(batch.contains_key(&UserById(7)));
    assert_eq!(result, Some(User { id: 7, name: "Mio" }));
}

#[test]
fn requests_from_parallel_branches_are_batched_together() {
    let backend = TestBackend {
        users: vec![
            User { id: 7, name: "Mio" },
            User {
                id: 8,
                name: "Ritsu",
            },
            User { id: 9, name: "Yui" },
        ],
        posts: vec![
            Post {
                id: 10,
                author_id: 7,
            },
            Post {
                id: 20,
                author_id: 8,
            },
        ],
        ..Default::default()
    };
    let computation = fetch(FeedPosts)
        .and_then(|posts| {
            traverse(posts, |post| {
                fetch(UserById(post.author_id))
                    .map(|author| (post, author.expect("post author must exist")))
            })
            .zip(fetch(UserById(9)))
            .map(|(posts, viewer)| (viewer.expect("viewer must exist"), posts))
        })
        .map(|(viewer, posts)| {
            (
                viewer.name,
                posts
                    .into_iter()
                    .map(|(post, author)| (post.id, author.name))
                    .collect::<Vec<_>>(),
            )
        });
    let result = run(backend.run(computation)).unwrap();

    let reqs = backend.reqs.into_inner();
    assert_eq!(reqs.len(), 2);
    let posts = reqs[0]
        .downcast_ref::<HashMap<FeedPosts, Vec<Post>>>()
        .unwrap();
    assert_eq!(posts.len(), 1);
    assert!(posts.contains_key(&FeedPosts));
    let users = reqs[1]
        .downcast_ref::<HashMap<UserById, Option<User>>>()
        .unwrap();
    assert_eq!(users.len(), 3);
    assert!(users.contains_key(&UserById(7)));
    assert!(users.contains_key(&UserById(8)));
    assert!(users.contains_key(&UserById(9)));
    assert_eq!(result, ("Yui", vec![(10, "Mio"), (20, "Ritsu")]));
}
