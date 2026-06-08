#[path = "api_example/backend.rs"]
mod backend;
#[path = "api_example/model.rs"]
mod model;

use placeholder_query_core::value::Value;
use placeholder_query_postgres::{Pg, PgStatement};
use placeholder_query_runtime::Fetch;

use backend::{TestPostgresBackend, TestRow};
use model::{
    PostComment, PostCommentsByPostId, PostWithAuthor, PostWithAuthorById, User, UserById,
    UserCard, UserCardById, post_comments, posts, users,
};

fn round_statements<A>(fetch: Fetch<TestPostgresBackend, A>) -> Vec<Vec<PgStatement>> {
    round_statements_with_rows(fetch, std::iter::empty())
}

fn round_statements_with_rows<A>(
    fetch: Fetch<TestPostgresBackend, A>,
    rows_by_round: impl IntoIterator<Item = Vec<Vec<TestRow>>>,
) -> Vec<Vec<PgStatement>> {
    let mut rounds = Vec::new();
    let mut rows_by_round = rows_by_round.into_iter();

    fetch
        .run_with(|statements| {
            let rows = rows_by_round
                .next()
                .unwrap_or_else(|| statements.iter().map(|_| Vec::new()).collect());

            assert_eq!(
                rows.len(),
                statements.len(),
                "fake rows should match statement count in the round"
            );

            rounds.push(statements);
            Ok::<_, std::convert::Infallible>(rows)
        })
        .expect("fake rows should be collectable in statement planning tests");

    assert!(
        rows_by_round.next().is_none(),
        "all explicit fake row rounds should be consumed"
    );

    rounds
}

#[test]
fn table_projection() {
    let statement = Pg.select(|q| q.from(users::table())).build();

    assert_eq!(
        statement.sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0"
    );
    assert_eq!(statement.params, []);
}

#[test]
fn partial_table_projection() {
    let statement = Pg
        .select(|q| q.from(users::table()).project(UserCard::project))
        .build();

    assert_eq!(statement.sql, "SELECT t0.id, t0.name FROM users AS t0");
    assert_eq!(statement.params, []);
}

#[test]
fn full_table_fetch_batch_builds_statement_and_collects_full_row() {
    let user = User {
        id: 7,
        name: "Mio".to_owned(),
        email: "mio@example.test".to_owned(),
    };
    let mut rounds = Vec::new();

    let result = Fetch::new(|cx| cx.fetch(UserById { id: 7 }))
        .run_with(|statements| {
            rounds.push(statements);

            Ok::<_, std::convert::Infallible>(vec![vec![TestRow::User(user.clone())]])
        })
        .unwrap();

    let statements = &rounds[0];

    assert_eq!(rounds.len(), 1);
    assert_eq!(statements.len(), 1);
    assert_eq!(
        statements[0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(statements[0].params, vec![Value::Int(7)]);
    assert_eq!(result, Some(user));
}

#[test]
fn partial_table_fetch_batch_builds_statement_and_collects_projected_row() {
    let user = User {
        id: 7,
        name: "Mio".to_owned(),
        email: "mio@example.test".to_owned(),
    };
    let mut rounds = Vec::new();

    let result = Fetch::new(|cx| cx.fetch(UserCardById { id: 7 }))
        .run_with(|statements| {
            rounds.push(statements);

            Ok::<_, std::convert::Infallible>(vec![vec![TestRow::User(user.clone())]])
        })
        .unwrap();

    let statements = &rounds[0];

    assert_eq!(rounds.len(), 1);
    assert_eq!(statements.len(), 1);
    assert_eq!(
        statements[0].sql,
        "SELECT t0.id, t0.name FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(statements[0].params, vec![Value::Int(7)]);
    assert_eq!(
        result,
        Some(UserCard {
            id: 7,
            display_name: "Mio".to_owned()
        })
    );
}

#[test]
fn merge_and_dedupe_key_in_same_fetch_batch() {
    let rounds = round_statements(Fetch::new(|cx| {
        cx.fetch(UserById { id: 7 })
            .zip(cx.traverse([7, 8, 9], |id, cx| cx.fetch(UserById { id })))
    }));
    let statements = &rounds[0];

    assert_eq!(rounds.len(), 1);
    assert_eq!(statements.len(), 1);
    assert_eq!(
        statements[0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1, $2, $3)"
    );
    assert_eq!(
        statements[0].params,
        vec![Value::Int(7), Value::Int(8), Value::Int(9)]
    );
}

#[test]
fn different_keys_are_distinct_requests() {
    let rounds = round_statements(Fetch::new(|cx| {
        cx.fetch(UserById { id: 7 })
            .zip(cx.fetch(UserCardById { id: 7 }))
    }));
    let statements = &rounds[0];

    assert_eq!(rounds.len(), 1);
    assert_eq!(statements.len(), 2);
    assert_eq!(
        statements[0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(
        statements[1].sql,
        "SELECT t0.id, t0.name FROM users AS t0 WHERE t0.id IN ($1)"
    );
}

#[test]
fn join_fetch_batch_builds_statement_and_collects_join_projection_row() {
    let post = PostWithAuthor {
        post_id: 99,
        title: "Intro".to_owned(),
        author_name: "Mio".to_owned(),
    };
    let mut rounds = Vec::new();

    let result = Fetch::new(|cx| cx.fetch(PostWithAuthorById { id: 99 }))
        .run_with(|statements| {
            rounds.push(statements);

            Ok::<_, std::convert::Infallible>(vec![vec![TestRow::PostWithAuthor(post.clone())]])
        })
        .unwrap();

    let statements = &rounds[0];

    assert_eq!(rounds.len(), 1);
    assert_eq!(statements.len(), 1);
    assert_eq!(
        statements[0].sql,
        "SELECT t0.id, t0.title, t1.name FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id WHERE t0.id IN ($1)"
    );
    assert_eq!(statements[0].params, vec![Value::Int(99)]);
    assert_eq!(result, Some(post));
}

#[test]
fn nested_join_projection() {
    let statement = Pg
        .select(|q| {
            q.from(posts::table())
                .join(users::table(), |(post, author)| {
                    post.author_id().eq(author.id())
                })
                .join(post_comments::table(), |((post, _), comment)| {
                    post.id().eq(comment.post_id())
                })
                .filter(|((post, author), comment)| {
                    [
                        post.id().eq(99),
                        author.name().like("M%"),
                        comment.body().like("%first%"),
                    ]
                })
                .project(|((post, author), comment)| (post.id(), author.name(), comment.body()))
        })
        .build();

    assert_eq!(
        statement.sql,
        "SELECT t0.id, t1.name, t2.body FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id JOIN post_comments AS t2 ON t0.id = t2.post_id WHERE t0.id = $1 AND t1.name LIKE $2 AND t2.body LIKE $3"
    );
    assert_eq!(
        statement.params,
        vec![
            Value::Int(99),
            Value::Text("M%".to_owned()),
            Value::Text("%first%".to_owned()),
        ]
    );
}

#[test]
fn self_join_projection_uses_distinct_aliases() {
    let statement = Pg
        .select(|q| {
            q.from(users::table())
                .join(users::table(), |(user, manager)| user.id().eq(manager.id()))
                .filter(|(_, manager)| [manager.email().like("%@example.test")])
                .project(|(user, manager)| (user.id(), user.name(), manager.email()))
        })
        .build();

    assert_eq!(
        statement.sql,
        "SELECT t0.id, t0.name, t1.email FROM users AS t0 JOIN users AS t1 ON t0.id = t1.id WHERE t1.email LIKE $1"
    );
    assert_eq!(
        statement.params,
        vec![Value::Text("%@example.test".to_owned())]
    );
}
#[test]
fn dependent_fetch_to_same_key_reuses_cached_row() {
    let fetch: Fetch<TestPostgresBackend, Option<User>> = Fetch::new(|cx| {
        cx.fetch(UserById { id: 7 })
            .and_then(|user, cx| match user {
                Some(user) => cx.fetch(UserById { id: user.id }),
                None => Fetch::pure(None),
            })
    });

    let rounds = round_statements_with_rows(
        fetch,
        [vec![vec![TestRow::User(User {
            id: 7,
            ..Default::default()
        })]]],
    );

    assert_eq!(rounds.len(), 1);
    assert_eq!(rounds[0].len(), 1);
    assert_eq!(
        rounds[0][0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[0][0].params, vec![Value::Int(7)]);
}

#[test]
fn then_right_side_is_not_in_first_round() {
    let fetch: Fetch<TestPostgresBackend, Option<PostWithAuthor>> = Fetch::new(|cx| {
        cx.fetch(UserById { id: 7 })
            .and_then(|user, cx| match user {
                Some(user) => cx.fetch(PostWithAuthorById { id: user.id }),
                None => Fetch::pure(None),
            })
    });

    let rounds = round_statements_with_rows(
        fetch,
        [vec![vec![TestRow::User(User {
            id: 7,
            ..Default::default()
        })]]],
    );

    assert_eq!(rounds.len(), 2);
    assert_eq!(rounds[0].len(), 1);
    assert_eq!(
        rounds[0][0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[0][0].params, vec![Value::Int(7)]);
    assert_eq!(rounds[1].len(), 1);
    assert_eq!(
        rounds[1][0].sql,
        "SELECT t0.id, t0.title, t1.name FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[1][0].params, vec![Value::Int(7)]);
}

#[test]
fn deep_dependency_chain_runs_one_round_per_dependency_level() {
    let fetch: Fetch<TestPostgresBackend, Option<User>> = Fetch::new(|cx| {
        cx.fetch(UserById { id: 1 })
            .and_then(|user, cx| match user {
                Some(user) => cx.fetch(PostWithAuthorById { id: user.id + 10 }).and_then(
                    |post, cx| match post {
                        Some(post) => cx
                            .fetch(PostCommentsByPostId {
                                post_id: post.post_id,
                            })
                            .and_then(move |comments, cx| {
                                let next_id = comments
                                    .first()
                                    .map(|comment| comment.post_id + 1)
                                    .unwrap_or(post.post_id + 1);

                                cx.fetch(UserById { id: next_id })
                            }),
                        None => Fetch::pure(None),
                    },
                ),
                None => Fetch::pure(None),
            })
    });

    let rounds = round_statements_with_rows(
        fetch,
        [
            vec![vec![TestRow::User(User {
                id: 1,
                ..Default::default()
            })]],
            vec![vec![TestRow::PostWithAuthor(PostWithAuthor {
                post_id: 11,
                ..Default::default()
            })]],
            vec![vec![TestRow::PostComment(PostComment {
                id: 1011,
                post_id: 11,
                ..Default::default()
            })]],
        ],
    );

    assert_eq!(rounds.len(), 4);
    assert_eq!(
        rounds[0][0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[0][0].params, vec![Value::Int(1)]);
    assert_eq!(
        rounds[1][0].sql,
        "SELECT t0.id, t0.title, t1.name FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[1][0].params, vec![Value::Int(11)]);
    assert_eq!(
        rounds[2][0].sql,
        "SELECT t0.id, t0.post_id, t0.body FROM post_comments AS t0 WHERE t0.post_id IN ($1)"
    );
    assert_eq!(rounds[2][0].params, vec![Value::Int(11)]);
    assert_eq!(
        rounds[3][0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[3][0].params, vec![Value::Int(12)]);
}

#[test]
fn diamond_dependency_batches_middle_layer_then_joins_again() {
    let fetch: Fetch<TestPostgresBackend, Option<PostWithAuthor>> = Fetch::new(|cx| {
        cx.fetch(UserById { id: 1 })
            .and_then(|user, cx| match user {
                Some(user) => cx
                    .fetch(UserById { id: user.id + 1 })
                    .zip(cx.fetch(PostWithAuthorById { id: user.id + 20 }))
                    .zip(cx.fetch(PostCommentsByPostId {
                        post_id: user.id + 20,
                    }))
                    .and_then(|((left, right), comments), cx| match (left, right) {
                        (Some(left), Some(right)) if !comments.is_empty() => {
                            cx.fetch(PostWithAuthorById {
                                id: left.id + right.post_id + comments[0].post_id,
                            })
                        }
                        _ => Fetch::pure(None),
                    }),
                None => Fetch::pure(None),
            })
    });

    let rounds = round_statements_with_rows(
        fetch,
        [
            vec![vec![TestRow::User(User {
                id: 1,
                ..Default::default()
            })]],
            vec![
                vec![TestRow::User(User {
                    id: 2,
                    ..Default::default()
                })],
                vec![TestRow::PostWithAuthor(PostWithAuthor {
                    post_id: 21,
                    ..Default::default()
                })],
                vec![TestRow::PostComment(PostComment {
                    id: 1021,
                    post_id: 21,
                    ..Default::default()
                })],
            ],
        ],
    );

    assert_eq!(rounds.len(), 3);
    assert_eq!(rounds[0].len(), 1);
    assert_eq!(
        rounds[0][0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[0][0].params, vec![Value::Int(1)]);
    assert_eq!(rounds[1].len(), 3);
    assert_eq!(
        rounds[1][0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[1][0].params, vec![Value::Int(2)]);
    assert_eq!(
        rounds[1][1].sql,
        "SELECT t0.id, t0.title, t1.name FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[1][1].params, vec![Value::Int(21)]);
    assert_eq!(
        rounds[1][2].sql,
        "SELECT t0.id, t0.post_id, t0.body FROM post_comments AS t0 WHERE t0.post_id IN ($1)"
    );
    assert_eq!(rounds[1][2].params, vec![Value::Int(21)]);
    assert_eq!(rounds[2].len(), 1);
    assert_eq!(
        rounds[2][0].sql,
        "SELECT t0.id, t0.title, t1.name FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[2][0].params, vec![Value::Int(44)]);
}

#[test]
fn fan_out_fan_in_batches_wide_independent_layer() {
    let fetch: Fetch<TestPostgresBackend, Option<User>> = Fetch::new(|cx| {
        cx.traverse(1..=5, |id, cx| cx.fetch(UserById { id }))
            .zip(cx.traverse([10, 20, 30, 40, 50], |id, cx| {
                cx.fetch(PostWithAuthorById { id })
            }))
            .zip(cx.traverse([10, 20, 30], |post_id, cx| {
                cx.fetch(PostCommentsByPostId { post_id })
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
                        .map(|comment| comment.post_id)
                        .sum::<i32>();

                cx.fetch(UserById { id: next_id })
            })
    });

    let rounds = round_statements_with_rows(
        fetch,
        [vec![
            (1..=5)
                .map(|id| {
                    TestRow::User(User {
                        id,
                        ..Default::default()
                    })
                })
                .collect(),
            [10, 20, 30, 40, 50]
                .into_iter()
                .map(|post_id| {
                    TestRow::PostWithAuthor(PostWithAuthor {
                        post_id,
                        ..Default::default()
                    })
                })
                .collect(),
            [10, 20, 30]
                .into_iter()
                .map(|post_id| {
                    TestRow::PostComment(PostComment {
                        id: post_id + 1000,
                        post_id,
                        ..Default::default()
                    })
                })
                .collect(),
        ]],
    );

    assert_eq!(rounds.len(), 2);
    assert_eq!(rounds[0].len(), 3);
    assert_eq!(
        rounds[0][0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1, $2, $3, $4, $5)"
    );
    assert_eq!(
        rounds[0][0].params,
        (1..=5).map(Value::Int).collect::<Vec<_>>()
    );
    assert_eq!(
        rounds[0][1].sql,
        "SELECT t0.id, t0.title, t1.name FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id WHERE t0.id IN ($1, $2, $3, $4, $5)"
    );
    assert_eq!(
        rounds[0][1].params,
        [10, 20, 30, 40, 50]
            .into_iter()
            .map(Value::Int)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        rounds[0][2].sql,
        "SELECT t0.id, t0.post_id, t0.body FROM post_comments AS t0 WHERE t0.post_id IN ($1, $2, $3)"
    );
    assert_eq!(
        rounds[0][2].params,
        [10, 20, 30].into_iter().map(Value::Int).collect::<Vec<_>>()
    );
    assert_eq!(rounds[1].len(), 1);
    assert_eq!(
        rounds[1][0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[1][0].params, vec![Value::Int(225)]);
}

#[test]
fn nested_applicative_fetches_share_one_batch_round() {
    let fetch = Fetch::new(|cx| {
        cx.fetch(UserById { id: 1 }).zip(
            cx.fetch(PostWithAuthorById { id: 10 })
                .zip(cx.fetch(UserCardById { id: 1 }))
                .zip(cx.traverse([2, 3, 4], |id, cx| cx.fetch(UserById { id })))
                .zip(cx.fetch(PostCommentsByPostId { post_id: 10 })),
        )
    });

    let rounds = round_statements(fetch);

    assert_eq!(rounds.len(), 1);
    assert_eq!(rounds[0].len(), 4);
    assert_eq!(
        rounds[0][0].sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1, $2, $3, $4)"
    );
    assert_eq!(
        rounds[0][0].params,
        vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]
    );
    assert_eq!(
        rounds[0][1].sql,
        "SELECT t0.id, t0.title, t1.name FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[0][1].params, vec![Value::Int(10)]);
    assert_eq!(
        rounds[0][2].sql,
        "SELECT t0.id, t0.name FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[0][2].params, vec![Value::Int(1)]);
    assert_eq!(
        rounds[0][3].sql,
        "SELECT t0.id, t0.post_id, t0.body FROM post_comments AS t0 WHERE t0.post_id IN ($1)"
    );
    assert_eq!(rounds[0][3].params, vec![Value::Int(10)]);
}

#[test]
fn missing_outer_result_does_not_schedule_dependent_fetch() {
    let fetch: Fetch<TestPostgresBackend, Option<User>> = Fetch::new(|cx| {
        cx.fetch(UserCardById { id: 1 })
            .and_then(|card, cx| match card {
                Some(card) => cx.fetch(UserById { id: card.id }),
                None => Fetch::pure(None),
            })
    });

    let rounds = round_statements(fetch);

    assert_eq!(rounds.len(), 1);
    assert_eq!(rounds[0].len(), 1);
    assert_eq!(
        rounds[0][0].sql,
        "SELECT t0.id, t0.name FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(rounds[0][0].params, vec![Value::Int(1)]);
}
