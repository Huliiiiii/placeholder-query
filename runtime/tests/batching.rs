mod support;

use std::{cell::RefCell, collections::HashMap};

use placeholder_query_runtime::{
    DataSource, Fetch, FetchEnv, FetchError, Request, fetch, traverse,
};

use support::{TestWake, run_future, yield_once};

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserById(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserCardById(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FeedPosts;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum BatchRequest {
    Users(Vec<i32>),
    UserCards(Vec<i32>),
    FeedPosts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RequestError;

#[derive(Default)]
struct TestBackend {
    users: Vec<User>,
    user_cards: Vec<UserCard>,
    posts: Vec<Post>,
    reqs: RefCell<Vec<BatchRequest>>,
}

impl FetchEnv for TestBackend {
    type Error = RequestError;
}

impl Request for UserById {
    type Output = Option<User>;
}

impl DataSource<UserById> for TestBackend {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a UserById>,
    ) -> Result<HashMap<UserById, Option<User>>, RequestError> {
        let reqs = reqs.into_iter().collect::<Vec<_>>();

        self.reqs
            .borrow_mut()
            .push(BatchRequest::Users(reqs.iter().map(|req| req.0).collect()));
        yield_once().await;

        Ok(reqs
            .into_iter()
            .cloned()
            .map(|req| {
                let user = self.users.iter().find(|user| user.id == req.0).cloned();
                (req, user)
            })
            .collect())
    }
}

impl Request for UserCardById {
    type Output = Option<UserCard>;
}

impl DataSource<UserCardById> for TestBackend {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a UserCardById>,
    ) -> Result<HashMap<UserCardById, Option<UserCard>>, RequestError> {
        let reqs = reqs.into_iter().collect::<Vec<_>>();

        self.reqs.borrow_mut().push(BatchRequest::UserCards(
            reqs.iter().map(|req| req.0).collect(),
        ));
        yield_once().await;

        Ok(reqs
            .into_iter()
            .cloned()
            .map(|req| {
                let card = self
                    .user_cards
                    .iter()
                    .find(|card| card.id == req.0)
                    .cloned();
                (req, card)
            })
            .collect())
    }
}

impl Request for FeedPosts {
    type Output = Vec<Post>;
}

impl DataSource<FeedPosts> for TestBackend {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a FeedPosts>,
    ) -> Result<HashMap<FeedPosts, Vec<Post>>, RequestError> {
        self.reqs.borrow_mut().push(BatchRequest::FeedPosts);
        yield_once().await;

        Ok(reqs
            .into_iter()
            .map(|req| (req.clone(), self.posts.clone()))
            .collect())
    }
}

fn run_fetch<A>(
    fetch: Fetch<TestBackend, A>,
    backend: TestBackend,
) -> (Result<A, FetchError<RequestError>>, Vec<BatchRequest>)
where
    A: 'static,
{
    let result = run_future(FetchEnv::run(&backend, fetch), TestWake::new());

    (result, backend.reqs.into_inner())
}

fn assert_batches(actual: &[BatchRequest], expected: &[BatchRequest]) {
    let batch_counts = |reqs: &[BatchRequest]| {
        let mut counts = HashMap::new();
        for mut req in reqs.iter().cloned() {
            match &mut req {
                BatchRequest::Users(ids) | BatchRequest::UserCards(ids) => ids.sort_unstable(),
                BatchRequest::FeedPosts => {}
            }
            *counts.entry(req).or_insert(0) += 1;
        }
        counts
    };

    assert_eq!(batch_counts(actual), batch_counts(expected));
}

#[test]
fn traverse_returns_results_in_input_order_after_deduping_requests() {
    let (users, reqs) = run_fetch(
        traverse([2, 1, 2], |id| fetch(UserById(id))),
        TestBackend {
            users: vec![
                User { id: 1, name: "Mio" },
                User {
                    id: 2,
                    name: "Ritsu",
                },
            ],
            ..Default::default()
        },
    );
    let users = users.unwrap();

    assert_batches(&reqs, &[BatchRequest::Users(vec![2, 1])]);
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
        ]
    );
}

#[test]
fn different_request_types_are_cached_separately() {
    let (result, reqs) = run_fetch(
        fetch(UserById(7)).zip(fetch(UserCardById(7))),
        TestBackend {
            users: vec![User { id: 7, name: "Mio" }],
            user_cards: vec![UserCard {
                id: 7,
                display_name: "Mio",
            }],
            ..Default::default()
        },
    );
    let result = result.unwrap();

    assert_batches(
        &reqs,
        &[
            BatchRequest::Users(vec![7]),
            BatchRequest::UserCards(vec![7]),
        ],
    );
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
fn dependent_fetch_to_same_request_reuses_cached_row() {
    let fetch = fetch(UserById(7)).and_then(|user| match user {
        Some(user) => fetch(UserById(user.id)),
        None => Fetch::pure(None),
    });
    let (result, reqs) = run_fetch(
        fetch,
        TestBackend {
            users: vec![User { id: 7, name: "Mio" }],
            ..Default::default()
        },
    );
    let result = result.unwrap();

    assert_eq!(reqs, vec![BatchRequest::Users(vec![7])]);
    assert_eq!(result, Some(User { id: 7, name: "Mio" }));
}

#[test]
fn feed_batches_author_and_viewer_requests_before_building_the_result() {
    let fetch = fetch(FeedPosts)
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
    let (result, reqs) = run_fetch(
        fetch,
        TestBackend {
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
        },
    );

    assert_batches(
        &reqs,
        &[BatchRequest::FeedPosts, BatchRequest::Users(vec![7, 8, 9])],
    );
    assert_eq!(result.unwrap(), ("Yui", vec![(10, "Mio"), (20, "Ritsu")]));
}
