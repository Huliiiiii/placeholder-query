mod comment;
mod post;
mod user;

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkGroup, BenchmarkId, Criterion, measurement::WallTime};
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait,
    LoaderTrait, QueryFilter, QueryTrait,
};
use tokio::runtime::Runtime;

use super::fixture::{self, Fixture};

type AssociationModels = Vec<(user::Model, Vec<(post::Model, Vec<comment::Model>)>)>;

fn connect(runtime: &Runtime, url: &str) -> DatabaseConnection {
    runtime.block_on(async {
        let mut options = ConnectOptions::new(url);
        options.max_connections(1).min_connections(1);
        Database::connect(options)
            .await
            .expect("SeaORM benchmark database connection")
    })
}

async fn load_users(db: &DatabaseConnection) -> Result<Vec<user::Model>, DbErr> {
    user::Entity::find().all(db).await
}

async fn load_associations(db: &DatabaseConnection) -> Result<AssociationModels, DbErr> {
    let users = load_users(db).await?;
    let posts_by_user = users.load_many(post::Entity, db).await?;
    let post_counts = posts_by_user.iter().map(Vec::len).collect::<Vec<_>>();
    let posts = posts_by_user.into_iter().flatten().collect::<Vec<_>>();
    let comments_by_post = posts.load_many(comment::Entity, db).await?;
    let mut posts = posts.into_iter().zip(comments_by_post);

    Ok(users
        .into_iter()
        .zip(post_counts)
        .map(|(user, post_count)| {
            let posts = posts.by_ref().take(post_count).collect();
            (user, posts)
        })
        .collect())
}

fn fixture_user(model: user::Model) -> fixture::User {
    fixture::User {
        id: model.id,
        name: model.name,
        hair_color: model.hair_color,
    }
}

fn fixture_post(model: post::Model) -> fixture::Post {
    fixture::Post {
        id: model.id,
        user_id: model.user_id,
        title: model.title,
        body: model.body,
    }
}

fn fixture_comment(model: comment::Model) -> fixture::Comment {
    fixture::Comment {
        id: model.id,
        post_id: model.post_id,
        text: model.text,
    }
}

fn assert_users(fixture: &Fixture, models: Vec<user::Model>) {
    fixture.assert_users(models.into_iter().map(fixture_user).collect());
}

fn assert_medium(fixture: &Fixture, models: Vec<(user::Model, Option<post::Model>)>) {
    fixture.assert_medium(
        models
            .into_iter()
            .map(|(user, post)| (fixture_user(user), post.map(fixture_post)))
            .collect(),
    );
}

fn assert_associations(fixture: &Fixture, models: AssociationModels) {
    fixture.assert_associations(
        models
            .into_iter()
            .map(|(user, posts)| {
                (
                    fixture_user(user),
                    posts
                        .into_iter()
                        .map(|(post, comments)| {
                            (
                                fixture_post(post),
                                comments.into_iter().map(fixture_comment).collect(),
                            )
                        })
                        .collect(),
                )
            })
            .collect(),
    );
}

pub fn construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_build/trivial");
    group.bench_function("sea_orm", |b| {
        b.iter(|| black_box(user::Entity::find()));
    });
    group.finish();

    let mut group = c.benchmark_group("query_build/medium");
    group.bench_function("sea_orm", |b| {
        b.iter(|| {
            black_box(
                user::Entity::find()
                    .filter(user::Column::HairColor.eq("black"))
                    .left_join(post::Entity)
                    .select_also(post::Entity),
            )
        });
    });
    group.finish();

    let user_ids = (1..=100).collect::<Vec<_>>();
    let post_ids = (1..=1_000).collect::<Vec<_>>();
    let mut group = c.benchmark_group("query_build/associations");
    group.bench_function("sea_orm", |b| {
        b.iter(|| {
            let user_ids = black_box(&user_ids);
            let post_ids = black_box(&post_ids);
            black_box((
                user::Entity::find(),
                post::Entity::find().filter(post::Column::UserId.is_in(user_ids.iter().copied())),
                comment::Entity::find()
                    .filter(comment::Column::PostId.is_in(post_ids.iter().copied())),
            ))
        });
    });
    group.finish();

    let mut group = c.benchmark_group("template_compile/trivial");
    group.bench_function("sea_orm", |b| {
        b.iter(|| black_box(user::Entity::find().build(DbBackend::Postgres)));
    });
    group.finish();

    let mut group = c.benchmark_group("template_compile/medium");
    group.bench_function("sea_orm", |b| {
        b.iter(|| {
            black_box(
                user::Entity::find()
                    .filter(user::Column::HairColor.eq("black"))
                    .left_join(post::Entity)
                    .select_also(post::Entity)
                    .build(DbBackend::Postgres),
            )
        });
    });
    group.finish();

    let mut group = c.benchmark_group("template_compile/associations");
    group.bench_function("sea_orm", |b| {
        b.iter(|| {
            let user_ids = black_box(&user_ids);
            let post_ids = black_box(&post_ids);
            black_box((
                user::Entity::find().build(DbBackend::Postgres),
                post::Entity::find()
                    .filter(post::Column::UserId.is_in(user_ids.iter().copied()))
                    .build(DbBackend::Postgres),
                comment::Entity::find()
                    .filter(comment::Column::PostId.is_in(post_ids.iter().copied()))
                    .build(DbBackend::Postgres),
            ))
        });
    });
    group.finish();
}

pub fn trivial(
    group: &mut BenchmarkGroup<'_, WallTime>,
    runtime: &Runtime,
    url: &str,
    fixture: &Fixture,
) {
    let db = connect(runtime, url);
    let query = user::Entity::find();
    let warmup = runtime
        .block_on(query.clone().all(&db))
        .expect("SeaORM trivial query");
    assert_users(fixture, warmup);

    group.bench_function(BenchmarkId::new("sea_orm", fixture.users.len()), |b| {
        b.to_async(runtime).iter_batched(
            || query.clone(),
            |query| async {
                let users = query.all(&db).await.expect("SeaORM trivial query");
                black_box(users);
            },
            BatchSize::SmallInput,
        );
    });
}

pub fn medium(
    group: &mut BenchmarkGroup<'_, WallTime>,
    runtime: &Runtime,
    url: &str,
    fixture: &Fixture,
) {
    let db = connect(runtime, url);
    let query = user::Entity::find()
        .filter(user::Column::HairColor.eq("black"))
        .left_join(post::Entity)
        .select_also(post::Entity);
    let warmup = runtime
        .block_on(query.clone().all(&db))
        .expect("SeaORM medium query");
    assert_medium(fixture, warmup);

    group.bench_function(BenchmarkId::new("sea_orm", fixture.users.len()), |b| {
        b.to_async(runtime).iter_batched(
            || query.clone(),
            |query| async {
                let users_with_posts = query.all(&db).await.expect("SeaORM medium query");
                black_box(users_with_posts);
            },
            BatchSize::SmallInput,
        );
    });
}

pub fn associations(
    group: &mut BenchmarkGroup<'_, WallTime>,
    runtime: &Runtime,
    url: &str,
    fixture: &Fixture,
) {
    let db = connect(runtime, url);
    let warmup = runtime
        .block_on(load_associations(&db))
        .expect("SeaORM associations query");
    assert_associations(fixture, warmup);

    group.bench_function("sea_orm", |b| {
        b.to_async(runtime).iter(|| async {
            let associations = load_associations(&db)
                .await
                .expect("SeaORM associations query");
            black_box(associations);
        });
    });
}
