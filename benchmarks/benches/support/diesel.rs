use criterion::{BatchSize, BenchmarkId, Criterion};
use diesel::{Connection, PgConnection, associations::GroupedBy, prelude::*};

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        hair_color -> Nullable<Text>,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        user_id -> Integer,
        title -> Text,
        body -> Nullable<Text>,
    }
}

diesel::table! {
    comments (id) {
        id -> Integer,
        post_id -> Integer,
        text -> Text,
    }
}

diesel::joinable!(posts -> users (user_id));
diesel::joinable!(comments -> posts (post_id));
diesel::allow_tables_to_appear_in_same_query!(users, posts, comments);

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Identifiable)]
#[diesel(table_name = users)]
struct DieselUser {
    id: i32,
    name: String,
    hair_color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(DieselUser, foreign_key = user_id))]
#[diesel(table_name = posts)]
struct DieselPost {
    id: i32,
    user_id: i32,
    title: String,
    body: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(DieselPost, foreign_key = post_id))]
#[diesel(table_name = comments)]
struct DieselComment {
    id: i32,
    post_id: i32,
    text: String,
}

pub fn construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_build/trivial");
    group.bench_function("diesel", |b| {
        b.iter(|| std::hint::black_box(users::table));
    });
    group.bench_function("diesel_boxed", |b| {
        b.iter(|| std::hint::black_box(users::table.into_boxed::<diesel::pg::Pg>()));
    });
    group.finish();

    let mut group = c.benchmark_group("query_build/medium");
    group.bench_function("diesel", |b| {
        b.iter(|| {
            std::hint::black_box(
                users::table
                    .left_join(posts::table)
                    .filter(users::hair_color.eq("black")),
            )
        });
    });
    group.bench_function("diesel_boxed", |b| {
        b.iter(|| {
            std::hint::black_box(
                users::table
                    .left_join(posts::table)
                    .filter(users::hair_color.eq("black"))
                    .into_boxed::<diesel::pg::Pg>(),
            )
        });
    });
    group.finish();

    let user_ids = (1..=100).collect::<Vec<_>>();
    let post_ids = (1..=1_000).collect::<Vec<_>>();
    let mut group = c.benchmark_group("query_build/associations");
    group.bench_function("diesel", |b| {
        b.iter(|| {
            let user_ids = std::hint::black_box(&user_ids);
            let post_ids = std::hint::black_box(&post_ids);
            std::hint::black_box((
                users::table,
                posts::table.filter(posts::user_id.eq_any(user_ids)),
                comments::table.filter(comments::post_id.eq_any(post_ids)),
            ))
        });
    });
    group.finish();
}

pub fn trivial(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    _runtime: &tokio::runtime::Runtime,
    database_url: &str,
    fixture: &super::fixture::Fixture,
) {
    let mut connection = PgConnection::establish(database_url).unwrap();
    let to_fixture_users = |users: Vec<DieselUser>| {
        users
            .into_iter()
            .map(|user| super::fixture::User {
                id: user.id,
                name: user.name,
                hair_color: user.hair_color,
            })
            .collect::<Vec<_>>()
    };
    let users = users::table.load::<DieselUser>(&mut connection).unwrap();
    fixture.assert_users(to_fixture_users(users));

    let boxed_users = users::table
        .into_boxed::<diesel::pg::Pg>()
        .load::<DieselUser>(&mut connection)
        .unwrap();
    fixture.assert_users(to_fixture_users(boxed_users));

    group.bench_function(BenchmarkId::new("diesel", fixture.users.len()), |b| {
        b.iter_batched(
            || users::table,
            |query| {
                let users = query.load::<DieselUser>(&mut connection).unwrap();
                std::hint::black_box(users);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function(BenchmarkId::new("diesel_boxed", fixture.users.len()), |b| {
        b.iter_batched(
            || users::table.into_boxed::<diesel::pg::Pg>(),
            |query| {
                let users = query.load::<DieselUser>(&mut connection).unwrap();
                std::hint::black_box(users);
            },
            BatchSize::SmallInput,
        );
    });
}

pub fn medium(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    _runtime: &tokio::runtime::Runtime,
    database_url: &str,
    fixture: &super::fixture::Fixture,
) {
    let mut connection = PgConnection::establish(database_url).unwrap();
    let to_fixture_medium = |rows: Vec<(DieselUser, Option<DieselPost>)>| {
        rows.into_iter()
            .map(|(user, post)| {
                (
                    super::fixture::User {
                        id: user.id,
                        name: user.name,
                        hair_color: user.hair_color,
                    },
                    post.map(|post| super::fixture::Post {
                        id: post.id,
                        user_id: post.user_id,
                        title: post.title,
                        body: post.body,
                    }),
                )
            })
            .collect::<Vec<_>>()
    };
    let users_and_posts = users::table
        .left_join(posts::table)
        .filter(users::hair_color.eq("black"))
        .load::<(DieselUser, Option<DieselPost>)>(&mut connection)
        .unwrap();
    fixture.assert_medium(to_fixture_medium(users_and_posts));

    let boxed_users_and_posts = users::table
        .left_join(posts::table)
        .filter(users::hair_color.eq("black"))
        .into_boxed::<diesel::pg::Pg>()
        .load::<(DieselUser, Option<DieselPost>)>(&mut connection)
        .unwrap();
    fixture.assert_medium(to_fixture_medium(boxed_users_and_posts));

    group.bench_function(BenchmarkId::new("diesel", fixture.users.len()), |b| {
        b.iter_batched(
            || {
                users::table
                    .left_join(posts::table)
                    .filter(users::hair_color.eq("black"))
            },
            |query| {
                let rows = query
                    .load::<(DieselUser, Option<DieselPost>)>(&mut connection)
                    .unwrap();
                std::hint::black_box(rows);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function(BenchmarkId::new("diesel_boxed", fixture.users.len()), |b| {
        b.iter_batched(
            || {
                users::table
                    .left_join(posts::table)
                    .filter(users::hair_color.eq("black"))
                    .into_boxed::<diesel::pg::Pg>()
            },
            |query| {
                let rows = query
                    .load::<(DieselUser, Option<DieselPost>)>(&mut connection)
                    .unwrap();
                std::hint::black_box(rows);
            },
            BatchSize::SmallInput,
        );
    });
}

pub fn associations(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    _runtime: &tokio::runtime::Runtime,
    database_url: &str,
    fixture: &super::fixture::Fixture,
) {
    let mut connection = PgConnection::establish(database_url).unwrap();
    let associations = load_associations(&mut connection);
    fixture.assert_associations(
        associations
            .into_iter()
            .map(|(user, posts)| {
                (
                    super::fixture::User {
                        id: user.id,
                        name: user.name,
                        hair_color: user.hair_color,
                    },
                    posts
                        .into_iter()
                        .map(|(post, comments)| {
                            (
                                super::fixture::Post {
                                    id: post.id,
                                    user_id: post.user_id,
                                    title: post.title,
                                    body: post.body,
                                },
                                comments
                                    .into_iter()
                                    .map(|comment| super::fixture::Comment {
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

    group.bench_function("diesel", |b| {
        b.iter(|| std::hint::black_box(load_associations(&mut connection)));
    });
}

fn load_associations(
    connection: &mut PgConnection,
) -> Vec<(DieselUser, Vec<(DieselPost, Vec<DieselComment>)>)> {
    let users = users::table.load::<DieselUser>(connection).unwrap();
    let posts = DieselPost::belonging_to(&users)
        .load::<DieselPost>(connection)
        .unwrap();
    let comments = DieselComment::belonging_to(&posts)
        .load::<DieselComment>(connection)
        .unwrap();
    let comments = comments.grouped_by(&posts);
    let posts_and_comments = posts.into_iter().zip(comments).grouped_by(&users);

    users
        .into_iter()
        .zip(posts_and_comments)
        .collect::<Vec<_>>()
}
