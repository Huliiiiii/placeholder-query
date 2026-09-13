use std::hint::black_box;

use futures_util::TryStreamExt;
use tokio_postgres::{Client, Statement, types::ToSql};

use super::fixture::{self, Comment, Post, User};

async fn load_users(client: &Client, statement: &Statement) -> Vec<User> {
    client
        .query_raw(statement, std::iter::empty::<&(dyn ToSql + Sync)>())
        .await
        .unwrap()
        .map_ok(|row| User {
            id: row.get(0),
            name: row.get(1),
            hair_color: row.get(2),
        })
        .try_collect()
        .await
        .unwrap()
}

async fn load_medium(client: &Client, statement: &Statement) -> Vec<(User, Option<Post>)> {
    client
        .query_raw(statement, [&"black"])
        .await
        .unwrap()
        .map_ok(|row| {
            let post = row.get::<_, Option<i32>>(3).map(|id| Post {
                id,
                user_id: row.get(4),
                title: row.get(5),
                body: row.get(6),
            });
            (
                User {
                    id: row.get(0),
                    name: row.get(1),
                    hair_color: row.get(2),
                },
                post,
            )
        })
        .try_collect()
        .await
        .unwrap()
}

async fn load_posts(client: &Client, statement: &Statement, user_ids: &[i32]) -> Vec<Post> {
    client
        .query_raw(statement, [&user_ids])
        .await
        .unwrap()
        .map_ok(|row| Post {
            id: row.get(0),
            user_id: row.get(1),
            title: row.get(2),
            body: row.get(3),
        })
        .try_collect()
        .await
        .unwrap()
}

async fn load_comments(client: &Client, statement: &Statement, post_ids: &[i32]) -> Vec<Comment> {
    client
        .query_raw(statement, [&post_ids])
        .await
        .unwrap()
        .map_ok(|row| Comment {
            id: row.get(0),
            post_id: row.get(1),
            text: row.get(2),
        })
        .try_collect()
        .await
        .unwrap()
}

async fn load_associations(
    client: &Client,
    users_statement: &Statement,
    posts_statement: &Statement,
    comments_statement: &Statement,
) -> fixture::Associations {
    let users = load_users(client, users_statement).await;
    let user_ids = users.iter().map(|user| user.id).collect::<Vec<_>>();
    let posts = load_posts(client, posts_statement, &user_ids).await;
    let post_ids = posts.iter().map(|post| post.id).collect::<Vec<_>>();
    let comments = load_comments(client, comments_statement, &post_ids).await;

    fixture::group_associations(users, posts, comments)
}

pub fn trivial(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    runtime: &tokio::runtime::Runtime,
    url: &str,
    fixture: &fixture::Fixture,
) {
    let client = runtime.block_on(fixture::connect(url));
    let statement = runtime
        .block_on(client.prepare("SELECT id, name, hair_color FROM users"))
        .unwrap();
    let warmup = runtime.block_on(load_users(&client, &statement));
    fixture.assert_users(warmup);

    group.bench_function(
        criterion::BenchmarkId::new("tokio_postgres", fixture.users.len()),
        |b| {
            b.to_async(runtime).iter(|| async {
                let users = load_users(&client, &statement).await;
                black_box(users);
            });
        },
    );
}

pub fn medium(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    runtime: &tokio::runtime::Runtime,
    url: &str,
    fixture: &fixture::Fixture,
) {
    let client = runtime.block_on(fixture::connect(url));
    let statement = runtime
        .block_on(client.prepare(
            "SELECT u.id, u.name, u.hair_color, p.id, p.user_id, p.title, p.body \
         FROM users AS u LEFT JOIN posts AS p ON u.id = p.user_id \
         WHERE u.hair_color = $1",
        ))
        .unwrap();
    let warmup = runtime.block_on(load_medium(&client, &statement));
    fixture.assert_medium(warmup);

    group.bench_function(
        criterion::BenchmarkId::new("tokio_postgres", fixture.users.len()),
        |b| {
            b.to_async(runtime).iter(|| async {
                let rows = load_medium(&client, &statement).await;
                black_box(rows);
            });
        },
    );
}

pub fn associations(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    runtime: &tokio::runtime::Runtime,
    url: &str,
    fixture: &fixture::Fixture,
) {
    let client = runtime.block_on(fixture::connect(url));
    let users_statement = runtime
        .block_on(client.prepare("SELECT id, name, hair_color FROM users"))
        .unwrap();
    let posts_statement = runtime
        .block_on(
            client.prepare("SELECT id, user_id, title, body FROM posts WHERE user_id = ANY($1)"),
        )
        .unwrap();
    let comments_statement = runtime
        .block_on(client.prepare("SELECT id, post_id, text FROM comments WHERE post_id = ANY($1)"))
        .unwrap();
    let warmup = runtime.block_on(load_associations(
        &client,
        &users_statement,
        &posts_statement,
        &comments_statement,
    ));
    fixture.assert_associations(warmup);

    group.bench_function("tokio_postgres", |b| {
        b.to_async(runtime).iter(|| async {
            let associations = load_associations(
                &client,
                &users_statement,
                &posts_statement,
                &comments_statement,
            )
            .await;
            black_box(associations);
        });
    });
}
