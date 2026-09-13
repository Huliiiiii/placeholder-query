use criterion::{BenchmarkGroup, BenchmarkId, Criterion, measurement::WallTime};
use futures_util::TryStreamExt;
use sqlx::{Connection, PgConnection};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;

use super::fixture::{Associations, Comment, Fixture, Post, User, group_associations};

pub fn trivial(
    group: &mut BenchmarkGroup<'_, WallTime>,
    runtime: &Runtime,
    url: &str,
    fixture: &Fixture,
) {
    let mut connection = runtime.block_on(PgConnection::connect(url)).unwrap();
    fixture.assert_users(runtime.block_on(load_users(&mut connection)));
    group.bench_function(BenchmarkId::new("sqlx", fixture.users.len()), |b| {
        b.iter_custom(|iterations| {
            runtime.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let query = users_query();
                    let start = Instant::now();
                    let users = query.fetch_all(&mut connection).await.unwrap();
                    black_box(users);
                    elapsed += start.elapsed();
                }
                elapsed
            })
        });
    });
}

type QueryAs<O> = sqlx::query::QueryAs<'static, sqlx::Postgres, O, sqlx::postgres::PgArguments>;

fn users_query() -> QueryAs<User> {
    sqlx::query_as("SELECT id, name, hair_color FROM users")
}

async fn load_users(connection: &mut PgConnection) -> Vec<User> {
    users_query().fetch_all(connection).await.unwrap()
}

#[derive(sqlx::FromRow)]
struct UserPost {
    id: i32,
    name: String,
    hair_color: Option<String>,
    post_id: Option<i32>,
    user_id: Option<i32>,
    title: Option<String>,
    body: Option<String>,
}

fn medium_query() -> QueryAs<UserPost> {
    sqlx::query_as(
        "SELECT u.id, u.name, u.hair_color, p.id AS post_id, p.user_id, p.title, p.body
         FROM users u LEFT JOIN posts p ON u.id = p.user_id WHERE u.hair_color = $1",
    )
    .bind("black")
}

async fn load_medium(connection: &mut PgConnection) -> Vec<(User, Option<Post>)> {
    load_medium_query(connection, medium_query()).await
}

async fn load_medium_query(
    connection: &mut PgConnection,
    query: QueryAs<UserPost>,
) -> Vec<(User, Option<Post>)> {
    query
        .fetch(connection)
        .map_ok(|row| {
            let post = row.post_id.map(|id| Post {
                id,
                user_id: row.user_id.unwrap(),
                title: row.title.unwrap(),
                body: row.body,
            });
            (
                User {
                    id: row.id,
                    name: row.name,
                    hair_color: row.hair_color,
                },
                post,
            )
        })
        .try_collect()
        .await
        .unwrap()
}

pub fn medium(
    group: &mut BenchmarkGroup<'_, WallTime>,
    runtime: &Runtime,
    url: &str,
    fixture: &Fixture,
) {
    let mut connection = runtime.block_on(PgConnection::connect(url)).unwrap();
    fixture.assert_medium(runtime.block_on(load_medium(&mut connection)));
    group.bench_function(BenchmarkId::new("sqlx", fixture.users.len()), |b| {
        b.iter_custom(|iterations| {
            runtime.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let query = medium_query();
                    let start = Instant::now();
                    let rows = load_medium_query(&mut connection, query).await;
                    black_box(rows);
                    elapsed += start.elapsed();
                }
                elapsed
            })
        });
    });
}

pub fn construction(c: &mut Criterion) {
    let user_ids: Vec<i32> = (1..=100).collect();
    let post_ids: Vec<i32> = (1..=1_000).collect();

    let mut group = c.benchmark_group("query_build/trivial");
    group.bench_function("sqlx", |b| {
        b.iter(|| {
            let sql = black_box("SELECT id, name, hair_color FROM users");
            black_box(sqlx::query_as::<sqlx::Postgres, User>(sql))
        });
    });
    group.finish();

    let mut group = c.benchmark_group("query_build/medium");
    group.bench_function("sqlx", |b| {
        b.iter(|| {
            let sql = black_box(
                "SELECT u.id, u.name, u.hair_color, p.id AS post_id, p.user_id, p.title, p.body
                 FROM users u LEFT JOIN posts p ON u.id = p.user_id WHERE u.hair_color = $1",
            );
            let query = sqlx::query_as::<sqlx::Postgres, UserPost>(sql).bind(black_box("black"));
            black_box(query)
        });
    });
    group.finish();

    let mut group = c.benchmark_group("query_build/associations");
    group.bench_function("sqlx", |b| {
        b.iter(|| {
            let users_sql = black_box("SELECT id, name, hair_color FROM users");
            let posts_sql =
                black_box("SELECT id, user_id, title, body FROM posts WHERE user_id = ANY($1)");
            let comments_sql =
                black_box("SELECT id, post_id, text FROM comments WHERE post_id = ANY($1)");
            let users = sqlx::query_as::<sqlx::Postgres, User>(users_sql);
            let posts =
                sqlx::query_as::<sqlx::Postgres, Post>(posts_sql).bind(black_box(&user_ids));
            let comments =
                sqlx::query_as::<sqlx::Postgres, Comment>(comments_sql).bind(black_box(&post_ids));
            black_box((users, posts, comments))
        });
    });
    group.finish();
}

async fn load_associations(connection: &mut PgConnection) -> Associations {
    let users = load_users(connection).await;
    let user_ids: Vec<_> = users.iter().map(|user| user.id).collect();
    let posts: Vec<Post> =
        sqlx::query_as("SELECT id, user_id, title, body FROM posts WHERE user_id = ANY($1)")
            .bind(&user_ids)
            .fetch_all(&mut *connection)
            .await
            .unwrap();
    let post_ids: Vec<_> = posts.iter().map(|post| post.id).collect();
    let comments: Vec<Comment> =
        sqlx::query_as("SELECT id, post_id, text FROM comments WHERE post_id = ANY($1)")
            .bind(&post_ids)
            .fetch_all(connection)
            .await
            .unwrap();
    group_associations(users, posts, comments)
}

pub fn associations(
    group: &mut BenchmarkGroup<'_, WallTime>,
    runtime: &Runtime,
    url: &str,
    fixture: &Fixture,
) {
    let mut connection = runtime.block_on(PgConnection::connect(url)).unwrap();
    fixture.assert_associations(runtime.block_on(load_associations(&mut connection)));
    group.bench_function("sqlx", |b| {
        b.iter_custom(|iterations| {
            runtime.block_on(async {
                let start = Instant::now();
                for _ in 0..iterations {
                    black_box(load_associations(&mut connection).await);
                }
                start.elapsed()
            })
        });
    });
}
