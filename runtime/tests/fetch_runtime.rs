use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use indexmap::IndexMap;
use placeholder_query_runtime::{DataSource, Fetch, FetchEnv, FetchKey};

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

#[derive(Clone, Debug, PartialEq, Eq)]
struct PostWithAuthor {
    post_id: i32,
    title: &'static str,
    author_name: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PostWithAuthorAndComment {
    post_id: i32,
    author_name: &'static str,
    comment_body: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UserWithManager {
    user_id: i32,
    user_name: &'static str,
    manager_email: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserById(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserCardById(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PostsByAuthor(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PostWithAuthorById(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PostWithAuthorAndCommentById(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserWithManagerById(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RequiredUserById(i32);

#[derive(Clone, Debug, PartialEq, Eq)]
enum UserLookupError {
    NotFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Request {
    Users(Vec<i32>),
    UserCards(Vec<i32>),
    Posts(Vec<i32>),
    PostsWithAuthors(Vec<i32>),
    PostAuthorComments(Vec<i32>),
    UserManagers(Vec<i32>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Row {
    User(User),
    UserCard(UserCard),
    Post(Post),
    PostWithAuthor(PostWithAuthor),
    PostWithAuthorAndComment(PostWithAuthorAndComment),
    UserWithManager(UserWithManager),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RequestError;

struct TestBackend<F> {
    respond: RefCell<F>,
    rounds: RefCell<Vec<Vec<Request>>>,
    pending_round: RefCell<Vec<Request>>,
}

impl<F> TestBackend<F> {
    fn new(respond: F) -> Self {
        Self {
            respond: RefCell::new(respond),
            rounds: RefCell::new(Vec::new()),
            pending_round: RefCell::new(Vec::new()),
        }
    }

    fn rounds(&self) -> Vec<Vec<Request>> {
        self.rounds.borrow().clone()
    }

    fn finish_round(&self) {
        let mut pending_round = self.pending_round.borrow_mut();
        if !pending_round.is_empty() {
            self.rounds
                .borrow_mut()
                .push(std::mem::take(&mut *pending_round));
        }
    }
}

impl<F> TestBackend<F>
where
    F: FnMut(Request) -> Result<Vec<Row>, RequestError>,
{
    fn record_request(&self, request: &Request) {
        self.pending_round.borrow_mut().push(request.clone());
    }

    fn rows(&self, request: Request) -> Result<Vec<Row>, RequestError> {
        (self.respond.borrow_mut())(request)
    }
}

impl<F> FetchEnv for TestBackend<F> {
    type Error = RequestError;
}

impl FetchKey for UserById {
    type Output = Option<User>;
}

impl<F> DataSource<UserById> for TestBackend<F>
where
    F: FnMut(Request) -> Result<Vec<Row>, RequestError>,
{
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [UserById],
    ) -> impl Future<Output = Result<IndexMap<UserById, Option<User>>, RequestError>> + 'a {
        let request = Request::Users(keys.iter().map(|key| key.0).collect());
        self.record_request(&request);

        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, None))
            .collect::<IndexMap<_, _>>();

        async move {
            yield_once().await;

            self.rows(request).map(|rows| {
                for row in rows {
                    let Row::User(user) = row else {
                        continue;
                    };
                    values.insert(UserById(user.id), Some(user));
                }

                values
            })
        }
    }
}

impl FetchKey for UserCardById {
    type Output = Option<UserCard>;
}

impl<F> DataSource<UserCardById> for TestBackend<F>
where
    F: FnMut(Request) -> Result<Vec<Row>, RequestError>,
{
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [UserCardById],
    ) -> impl Future<Output = Result<IndexMap<UserCardById, Option<UserCard>>, RequestError>> + 'a
    {
        let request = Request::UserCards(keys.iter().map(|key| key.0).collect());
        self.record_request(&request);

        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, None))
            .collect::<IndexMap<_, _>>();

        async move {
            yield_once().await;

            self.rows(request).map(|rows| {
                for row in rows {
                    let Row::UserCard(card) = row else {
                        continue;
                    };
                    values.insert(UserCardById(card.id), Some(card));
                }

                values
            })
        }
    }
}

impl FetchKey for PostsByAuthor {
    type Output = Vec<Post>;
}

impl<F> DataSource<PostsByAuthor> for TestBackend<F>
where
    F: FnMut(Request) -> Result<Vec<Row>, RequestError>,
{
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [PostsByAuthor],
    ) -> impl Future<Output = Result<IndexMap<PostsByAuthor, Vec<Post>>, RequestError>> + 'a {
        let request = Request::Posts(keys.iter().map(|key| key.0).collect());
        self.record_request(&request);

        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, Vec::new()))
            .collect::<IndexMap<_, _>>();

        async move {
            yield_once().await;

            self.rows(request).map(|rows| {
                for row in rows {
                    let Row::Post(post) = row else {
                        continue;
                    };
                    values
                        .entry(PostsByAuthor(post.author_id))
                        .or_default()
                        .push(post);
                }

                values
            })
        }
    }
}

impl FetchKey for PostWithAuthorById {
    type Output = Option<PostWithAuthor>;
}

impl<F> DataSource<PostWithAuthorById> for TestBackend<F>
where
    F: FnMut(Request) -> Result<Vec<Row>, RequestError>,
{
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [PostWithAuthorById],
    ) -> impl Future<
        Output = Result<IndexMap<PostWithAuthorById, Option<PostWithAuthor>>, RequestError>,
    > + 'a {
        let request = Request::PostsWithAuthors(keys.iter().map(|key| key.0).collect());
        self.record_request(&request);

        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, None))
            .collect::<IndexMap<_, _>>();

        async move {
            yield_once().await;

            self.rows(request).map(|rows| {
                for row in rows {
                    let Row::PostWithAuthor(post) = row else {
                        continue;
                    };
                    values.insert(PostWithAuthorById(post.post_id), Some(post));
                }

                values
            })
        }
    }
}

impl FetchKey for PostWithAuthorAndCommentById {
    type Output = Option<PostWithAuthorAndComment>;
}

impl<F> DataSource<PostWithAuthorAndCommentById> for TestBackend<F>
where
    F: FnMut(Request) -> Result<Vec<Row>, RequestError>,
{
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [PostWithAuthorAndCommentById],
    ) -> impl Future<
        Output = Result<
            IndexMap<PostWithAuthorAndCommentById, Option<PostWithAuthorAndComment>>,
            RequestError,
        >,
    > + 'a {
        let request = Request::PostAuthorComments(keys.iter().map(|key| key.0).collect());
        self.record_request(&request);

        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, None))
            .collect::<IndexMap<_, _>>();

        async move {
            yield_once().await;

            self.rows(request).map(|rows| {
                for row in rows {
                    let Row::PostWithAuthorAndComment(post) = row else {
                        continue;
                    };
                    values.insert(PostWithAuthorAndCommentById(post.post_id), Some(post));
                }

                values
            })
        }
    }
}

impl FetchKey for UserWithManagerById {
    type Output = Option<UserWithManager>;
}

impl<F> DataSource<UserWithManagerById> for TestBackend<F>
where
    F: FnMut(Request) -> Result<Vec<Row>, RequestError>,
{
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [UserWithManagerById],
    ) -> impl Future<
        Output = Result<IndexMap<UserWithManagerById, Option<UserWithManager>>, RequestError>,
    > + 'a {
        let request = Request::UserManagers(keys.iter().map(|key| key.0).collect());
        self.record_request(&request);

        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, None))
            .collect::<IndexMap<_, _>>();

        async move {
            yield_once().await;

            self.rows(request).map(|rows| {
                for row in rows {
                    let Row::UserWithManager(user) = row else {
                        continue;
                    };
                    values.insert(UserWithManagerById(user.user_id), Some(user));
                }

                values
            })
        }
    }
}

impl FetchKey for RequiredUserById {
    type Output = Result<User, UserLookupError>;
}

impl<F> DataSource<RequiredUserById> for TestBackend<F>
where
    F: FnMut(Request) -> Result<Vec<Row>, RequestError>,
{
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [RequiredUserById],
    ) -> impl Future<
        Output = Result<IndexMap<RequiredUserById, Result<User, UserLookupError>>, RequestError>,
    > + 'a {
        let request = Request::Users(keys.iter().map(|key| key.0).collect());
        self.record_request(&request);

        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, Err(UserLookupError::NotFound)))
            .collect::<IndexMap<_, _>>();

        async move {
            yield_once().await;

            self.rows(request).map(|rows| {
                for row in rows {
                    let Row::User(user) = row else {
                        continue;
                    };
                    values.insert(RequiredUserById(user.id), Ok(user));
                }

                values
            })
        }
    }
}

fn run_fetch<A, F>(
    fetch: Fetch<TestBackend<F>, A>,
    respond: F,
) -> (Result<A, RequestError>, Vec<Vec<Request>>)
where
    A: 'static,
    F: FnMut(Request) -> Result<Vec<Row>, RequestError> + 'static,
{
    let backend = TestBackend::new(respond);
    let result = run_test_fetch(fetch, &backend);

    (result, backend.rounds())
}

fn empty_rows(_: Request) -> Result<Vec<Row>, RequestError> {
    Ok(Vec::new())
}

fn run_test_fetch<A, F>(
    fetch: Fetch<TestBackend<F>, A>,
    backend: &TestBackend<F>,
) -> Result<A, RequestError>
where
    A: 'static,
    F: FnMut(Request) -> Result<Vec<Row>, RequestError> + 'static,
{
    let mut future = std::pin::pin!(fetch.run(backend));
    let mut cx = Context::from_waker(Waker::noop());

    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => backend.finish_round(),
        }
    }
}

fn yield_once() -> YieldOnce {
    YieldOnce(false)
}

struct YieldOnce(bool);

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();

            Poll::Pending
        }
    }
}

#[test]
fn full_table_fetch_batches_and_collects_full_row() {
    let user = User { id: 7, name: "Mio" };
    let row_user = user.clone();
    let (result, rounds) = run_fetch(Fetch::new(|cx| cx.fetch(UserById(7))), move |_| {
        Ok(vec![Row::User(row_user.clone())])
    });
    let result = result.unwrap();

    assert_eq!(rounds, vec![vec![Request::Users(vec![7])]]);
    assert_eq!(result, Some(user));
}

#[test]
fn partial_table_fetch_batches_and_collects_projected_row() {
    let card = UserCard {
        id: 7,
        display_name: "Mio",
    };
    let row_card = card.clone();
    let (result, rounds) = run_fetch(Fetch::new(|cx| cx.fetch(UserCardById(7))), move |_| {
        Ok(vec![Row::UserCard(row_card.clone())])
    });
    let result = result.unwrap();

    assert_eq!(rounds, vec![vec![Request::UserCards(vec![7])]]);
    assert_eq!(result, Some(card));
}

#[test]
fn join_fetch_batches_and_collects_join_row() {
    let post = PostWithAuthor {
        post_id: 99,
        title: "Intro",
        author_name: "Mio",
    };
    let row_post = post.clone();
    let (result, rounds) = run_fetch(
        Fetch::new(|cx| cx.fetch(PostWithAuthorById(99))),
        move |_| Ok(vec![Row::PostWithAuthor(row_post.clone())]),
    );
    let result = result.unwrap();

    assert_eq!(rounds, vec![vec![Request::PostsWithAuthors(vec![99])]]);
    assert_eq!(result, Some(post));
}

#[test]
fn nested_join_fetch_batches_and_collects_join_row() {
    let post = PostWithAuthorAndComment {
        post_id: 99,
        author_name: "Mio",
        comment_body: "first",
    };
    let row_post = post.clone();
    let (result, rounds) = run_fetch(
        Fetch::new(|cx| cx.fetch(PostWithAuthorAndCommentById(99))),
        move |_| Ok(vec![Row::PostWithAuthorAndComment(row_post.clone())]),
    );
    let result = result.unwrap();

    assert_eq!(rounds, vec![vec![Request::PostAuthorComments(vec![99])]]);
    assert_eq!(result, Some(post));
}

#[test]
fn self_join_fetch_batches_and_collects_join_row() {
    let user = UserWithManager {
        user_id: 7,
        user_name: "Mio",
        manager_email: "mio@example.test",
    };
    let row_user = user.clone();
    let (result, rounds) = run_fetch(
        Fetch::new(|cx| cx.fetch(UserWithManagerById(7))),
        move |_| Ok(vec![Row::UserWithManager(row_user.clone())]),
    );
    let result = result.unwrap();

    assert_eq!(rounds, vec![vec![Request::UserManagers(vec![7])]]);
    assert_eq!(result, Some(user));
}

#[test]
fn traverse_returns_results_in_input_order_after_deduping_requests() {
    let (users, rounds) = run_fetch(
        Fetch::new(|cx| cx.traverse([2, 1, 2], |id, cx| cx.fetch(UserById(id)))),
        |request| {
            Ok(match request {
                Request::Users(ids) => ids
                    .iter()
                    .map(|id| {
                        Row::User(User {
                            id: *id,
                            name: match id {
                                1 => "Mio",
                                2 => "Ritsu",
                                _ => "unknown",
                            },
                        })
                    })
                    .collect(),
                _ => Vec::new(),
            })
        },
    );
    let users = users.unwrap();

    assert_eq!(rounds, vec![vec![Request::Users(vec![2, 1])]]);
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
fn different_fetch_key_types_are_distinct_requests() {
    let (result, rounds) = run_fetch(
        Fetch::new(|cx| cx.fetch(UserById(7)).zip(cx.fetch(UserCardById(7)))),
        |request| {
            Ok(match request {
                Request::Users(ids) => ids
                    .into_iter()
                    .map(|id| Row::User(User { id, name: "Mio" }))
                    .collect(),
                Request::UserCards(ids) => ids
                    .into_iter()
                    .map(|id| {
                        Row::UserCard(UserCard {
                            id,
                            display_name: "Mio",
                        })
                    })
                    .collect(),
                _ => Vec::new(),
            })
        },
    );
    let result = result.unwrap();

    assert_eq!(
        rounds,
        vec![vec![Request::Users(vec![7]), Request::UserCards(vec![7])]]
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
fn nested_applicative_fetches_share_one_batch_round() {
    let fetch = Fetch::new(|cx| {
        cx.fetch(UserById(1)).zip(
            cx.fetch(PostWithAuthorById(10))
                .zip(cx.fetch(UserCardById(1)))
                .zip(cx.traverse([2, 3, 4], |id, cx| cx.fetch(UserById(id))))
                .zip(cx.fetch(PostsByAuthor(10))),
        )
    });
    let (result, rounds) = run_fetch(fetch, empty_rows);
    result.unwrap();

    assert_eq!(
        rounds,
        vec![vec![
            Request::Users(vec![1, 2, 3, 4]),
            Request::PostsWithAuthors(vec![10]),
            Request::UserCards(vec![1]),
            Request::Posts(vec![10]),
        ]]
    );
}

#[test]
fn dependent_fetch_to_same_key_reuses_cached_row() {
    let fetch = Fetch::new(|cx| {
        cx.fetch(UserById(7)).and_then(|user, cx| match user {
            Some(user) => cx.fetch(UserById(user.id)),
            None => Fetch::pure(None),
        })
    });
    let (result, rounds) = run_fetch(fetch, |_| Ok(vec![Row::User(User { id: 7, name: "Mio" })]));
    let result = result.unwrap();

    assert_eq!(rounds, vec![vec![Request::Users(vec![7])]]);
    assert_eq!(result, Some(User { id: 7, name: "Mio" }));
}

#[test]
fn dependent_requests_execute_in_later_round() {
    let fetch = Fetch::new(|cx| {
        cx.fetch(UserById(1)).and_then(|user, cx| {
            let author_id = user.unwrap().id;
            cx.fetch(PostWithAuthorById(author_id))
        })
    });
    let (result, rounds) = run_fetch(fetch, |request| {
        Ok(match request {
            Request::Users(ids) => ids
                .into_iter()
                .map(|id| Row::User(User { id, name: "Mio" }))
                .collect(),
            Request::PostsWithAuthors(ids) => ids
                .into_iter()
                .map(|id| {
                    Row::PostWithAuthor(PostWithAuthor {
                        post_id: id,
                        title: "Intro",
                        author_name: "Mio",
                    })
                })
                .collect(),
            _ => Vec::new(),
        })
    });
    let result = result.unwrap();

    assert_eq!(
        rounds,
        vec![
            vec![Request::Users(vec![1])],
            vec![Request::PostsWithAuthors(vec![1])]
        ]
    );
    assert_eq!(
        result,
        Some(PostWithAuthor {
            post_id: 1,
            title: "Intro",
            author_name: "Mio"
        })
    );
}

#[test]
fn deep_dependency_chain_runs_one_round_per_dependency_level() {
    let fetch = Fetch::new(|cx| {
        cx.fetch(UserById(1)).and_then(|user, cx| match user {
            Some(user) => {
                cx.fetch(PostWithAuthorById(user.id + 10))
                    .and_then(|post, cx| match post {
                        Some(post) => {
                            cx.fetch(PostsByAuthor(post.post_id))
                                .and_then(move |posts, cx| {
                                    let next_id = posts
                                        .first()
                                        .map(|post| post.author_id + 1)
                                        .unwrap_or(post.post_id + 1);

                                    cx.fetch(UserById(next_id))
                                })
                        }
                        None => Fetch::pure(None),
                    })
            }
            None => Fetch::pure(None),
        })
    });
    let (result, rounds) = run_fetch(fetch, |request| {
        Ok(match request {
            Request::Users(ids) => ids
                .into_iter()
                .map(|id| Row::User(User { id, name: "Mio" }))
                .collect(),
            Request::PostsWithAuthors(ids) => ids
                .into_iter()
                .map(|id| {
                    Row::PostWithAuthor(PostWithAuthor {
                        post_id: id,
                        title: "Intro",
                        author_name: "Mio",
                    })
                })
                .collect(),
            Request::Posts(ids) => ids
                .into_iter()
                .map(|id| {
                    Row::Post(Post {
                        id: 10,
                        author_id: id,
                    })
                })
                .collect(),
            _ => Vec::new(),
        })
    });
    result.unwrap();

    assert_eq!(
        rounds,
        vec![
            vec![Request::Users(vec![1])],
            vec![Request::PostsWithAuthors(vec![11])],
            vec![Request::Posts(vec![11])],
            vec![Request::Users(vec![12])],
        ]
    );
}

#[test]
fn diamond_dependency_batches_middle_layer_then_joins_again() {
    let fetch = Fetch::new(|cx| {
        cx.fetch(UserById(1)).and_then(|user, cx| match user {
            Some(user) => cx
                .fetch(UserById(user.id + 1))
                .zip(cx.fetch(PostWithAuthorById(user.id + 20)))
                .zip(cx.fetch(PostsByAuthor(user.id + 20)))
                .and_then(|((left, right), posts), cx| match (left, right) {
                    (Some(left), Some(right)) if !posts.is_empty() => cx.fetch(PostWithAuthorById(
                        left.id + right.post_id + posts[0].author_id,
                    )),
                    _ => Fetch::pure(None),
                }),
            None => Fetch::pure(None),
        })
    });
    let (result, rounds) = run_fetch(fetch, |request| {
        Ok(match request {
            Request::Users(ids) => ids
                .into_iter()
                .map(|id| Row::User(User { id, name: "Mio" }))
                .collect(),
            Request::PostsWithAuthors(ids) => ids
                .into_iter()
                .map(|id| {
                    Row::PostWithAuthor(PostWithAuthor {
                        post_id: id,
                        title: "Intro",
                        author_name: "Mio",
                    })
                })
                .collect(),
            Request::Posts(ids) => ids
                .into_iter()
                .map(|id| Row::Post(Post { id, author_id: id }))
                .collect(),
            _ => Vec::new(),
        })
    });
    result.unwrap();

    assert_eq!(
        rounds,
        vec![
            vec![Request::Users(vec![1])],
            vec![
                Request::Users(vec![2]),
                Request::PostsWithAuthors(vec![21]),
                Request::Posts(vec![21]),
            ],
            vec![Request::PostsWithAuthors(vec![44])],
        ]
    );
}

#[test]
fn fan_out_fan_in_batches_wide_independent_layer() {
    let fetch = Fetch::new(|cx| {
        cx.traverse(1..=5, |id, cx| cx.fetch(UserById(id)))
            .zip(cx.traverse([10, 20, 30, 40, 50], |id, cx| {
                cx.fetch(PostWithAuthorById(id))
            }))
            .zip(cx.traverse([10, 20, 30], |author_id, cx| {
                cx.fetch(PostsByAuthor(author_id))
            }))
            .and_then(|((users, posts), comments), cx| {
                let next_id = users.into_iter().flatten().map(|user| user.id).sum::<i32>()
                    + posts
                        .into_iter()
                        .flatten()
                        .map(|post| post.post_id)
                        .sum::<i32>()
                    + comments
                        .into_iter()
                        .flatten()
                        .map(|post| post.author_id)
                        .sum::<i32>();

                cx.fetch(UserById(next_id))
            })
    });
    let (result, rounds) = run_fetch(fetch, |request| {
        Ok(match request {
            Request::Users(ids) => ids
                .into_iter()
                .map(|id| Row::User(User { id, name: "Mio" }))
                .collect(),
            Request::PostsWithAuthors(ids) => ids
                .into_iter()
                .map(|id| {
                    Row::PostWithAuthor(PostWithAuthor {
                        post_id: id,
                        title: "Intro",
                        author_name: "Mio",
                    })
                })
                .collect(),
            Request::Posts(ids) => ids
                .into_iter()
                .map(|id| Row::Post(Post { id, author_id: id }))
                .collect(),
            _ => Vec::new(),
        })
    });
    result.unwrap();

    assert_eq!(
        rounds,
        vec![
            vec![
                Request::Users(vec![1, 2, 3, 4, 5]),
                Request::PostsWithAuthors(vec![10, 20, 30, 40, 50]),
                Request::Posts(vec![10, 20, 30]),
            ],
            vec![Request::Users(vec![225])],
        ]
    );
}

#[test]
fn missing_outer_result_does_not_schedule_dependent_fetch() {
    let fetch = Fetch::new(|cx| {
        cx.fetch(UserCardById(1)).and_then(|card, cx| match card {
            Some(card) => cx.fetch(UserById(card.id)),
            None => Fetch::pure(None),
        })
    });
    let (result, rounds) = run_fetch(fetch, empty_rows);
    let result = result.unwrap();

    assert_eq!(rounds, vec![vec![Request::UserCards(vec![1])]]);
    assert_eq!(result, None);
}

#[test]
fn request_error_stops_the_run() {
    let (result, _) = run_fetch(Fetch::new(|cx| cx.fetch(UserById(1))), |_| {
        Err(RequestError)
    });

    assert_eq!(result, Err(RequestError));
}

#[test]
fn per_key_failure_is_part_of_the_key_output() {
    let (result, _) = run_fetch(Fetch::new(|cx| cx.fetch(RequiredUserById(3))), empty_rows);
    let result = result.unwrap();

    assert_eq!(result, Err(UserLookupError::NotFound));
}
