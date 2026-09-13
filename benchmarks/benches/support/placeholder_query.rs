use std::collections::HashMap;

use criterion::{BenchmarkGroup, BenchmarkId, Criterion, measurement::WallTime};
use placeholder_query_macro::{QueryInput, table};
use placeholder_query_postgres::query::{
    any, mode, select as pg, select::template::SelectTemplate,
};
use placeholder_query_tokio_postgres::Executor;
use tokio::runtime::Runtime;

use super::fixture::{self, Fixture};

type Associations = Vec<(User, Vec<(Post, Vec<Comment>)>)>;

#[table(name = "users")]
struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
}

#[table(name = "posts")]
struct Post {
    id: i32,
    user_id: i32,
    title: String,
    body: Option<String>,
}

#[table(name = "comments")]
struct Comment {
    id: i32,
    post_id: i32,
    text: String,
}

#[derive(QueryInput)]
struct Ids {
    ids: Vec<i32>,
}

fn users() -> SelectTemplate<User<mode::Expr>> {
    pg::from(User::table()).compile()
}

fn posts() -> SelectTemplate<Post<mode::Expr>, Ids> {
    pg::query::<Ids, _>(|input| {
        pg::from(Post::table()).filter(|post| post.user_id().eq(any(input.ids)))
    })
    .compile()
}

fn comments() -> SelectTemplate<Comment<mode::Expr>, Ids> {
    pg::query::<Ids, _>(|input| {
        pg::from(Comment::table()).filter(|comment| comment.post_id().eq(any(input.ids)))
    })
    .compile()
}

pub fn construction(c: &mut Criterion) {
    c.benchmark_group("query_build/trivial")
        .bench_function("placeholder_query", |b| {
            b.iter(|| pg::from(std::hint::black_box(User::table())));
        });
    c.benchmark_group("query_build/associations")
        .bench_function("placeholder_query", |b| {
            b.iter(|| {
                (
                    pg::from(std::hint::black_box(User::table())),
                    pg::query::<Ids, _>(|input| {
                        pg::from(std::hint::black_box(Post::table()))
                            .filter(|post| post.user_id().eq(any(input.ids)))
                    }),
                    pg::query::<Ids, _>(|input| {
                        pg::from(std::hint::black_box(Comment::table()))
                            .filter(|comment| comment.post_id().eq(any(input.ids)))
                    }),
                )
            });
        });
    c.benchmark_group("template_compile/trivial")
        .bench_function("placeholder_query", |b| b.iter(users));
    c.benchmark_group("template_compile/associations")
        .bench_function("placeholder_query", |b| {
            b.iter(|| (users(), posts(), comments()));
        });
}

pub fn trivial(
    group: &mut BenchmarkGroup<'_, WallTime>,
    runtime: &Runtime,
    url: &str,
    fixture: &Fixture,
) {
    let executor = Executor::new(runtime.block_on(fixture::connect(url)));
    let users = users();
    fixture.assert_users(
        runtime
            .block_on(executor.run(users.clone()))
            .unwrap()
            .into_iter()
            .map(fixture_user)
            .collect(),
    );
    group.bench_function(
        BenchmarkId::new("placeholder_query", fixture.users.len()),
        |b| {
            b.to_async(runtime)
                .iter(|| async { executor.run(users.clone()).await.unwrap() });
        },
    );
}

async fn load_associations(
    executor: &Executor,
    users: &SelectTemplate<User<mode::Expr>>,
    posts: &SelectTemplate<Post<mode::Expr>, Ids>,
    comments: &SelectTemplate<Comment<mode::Expr>, Ids>,
) -> Associations {
    let users = executor.run(users.clone()).await.unwrap();
    let ids = Ids {
        ids: users.iter().map(|user| user.id).collect(),
    };
    let posts = executor.run(posts.bind(&ids)).await.unwrap();
    let ids = Ids {
        ids: posts.iter().map(|post| post.id).collect(),
    };
    let comments = executor.run(comments.bind(&ids)).await.unwrap();
    let mut comments_by_post = HashMap::<_, Vec<_>>::with_capacity(posts.len());
    for comment in comments {
        comments_by_post
            .entry(comment.post_id)
            .or_default()
            .push(comment);
    }
    let mut posts_by_user = HashMap::<_, Vec<_>>::with_capacity(users.len());
    for post in posts {
        let comments = comments_by_post.remove(&post.id).unwrap_or_default();
        posts_by_user
            .entry(post.user_id)
            .or_default()
            .push((post, comments));
    }
    users
        .into_iter()
        .map(|user| {
            let posts = posts_by_user.remove(&user.id).unwrap_or_default();
            (user, posts)
        })
        .collect()
}

fn fixture_user(user: User) -> fixture::User {
    fixture::User {
        id: user.id,
        name: user.name,
        hair_color: user.hair_color,
    }
}

fn assert_associations(fixture: &Fixture, rows: Associations) {
    fixture.assert_associations(
        rows.into_iter()
            .map(|(user, posts)| {
                (
                    fixture_user(user),
                    posts
                        .into_iter()
                        .map(|(post, comments)| {
                            (
                                fixture::Post {
                                    id: post.id,
                                    user_id: post.user_id,
                                    title: post.title,
                                    body: post.body,
                                },
                                comments
                                    .into_iter()
                                    .map(|comment| fixture::Comment {
                                        id: comment.id,
                                        post_id: comment.post_id,
                                        text: comment.text,
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                )
            })
            .collect(),
    );
}

pub fn associations(
    group: &mut BenchmarkGroup<'_, WallTime>,
    runtime: &Runtime,
    url: &str,
    fixture: &Fixture,
) {
    let executor = Executor::new(runtime.block_on(fixture::connect(url)));
    let users = users();
    let posts = posts();
    let comments = comments();
    assert_associations(
        fixture,
        runtime.block_on(load_associations(&executor, &users, &posts, &comments)),
    );
    group.bench_function("placeholder_query", |b| {
        b.to_async(runtime)
            .iter(|| load_associations(&executor, &users, &posts, &comments));
    });
}
