use std::{
    collections::HashMap,
    convert::Infallible,
    future::{Future, ready},
    hint::black_box,
    task::{Context, Poll, Waker},
};

use criterion::{Criterion, criterion_group, criterion_main};
use futures_util::{future::try_join_all, join, try_join};
use ruxl::{DataSource, Env, Request, Scope};

#[derive(Clone)]
struct User {
    id: usize,
}

#[derive(Clone)]
struct Card {
    id: usize,
}

#[derive(Clone)]
struct Post {
    id: usize,
}

struct BenchBackend;

impl Env for BenchBackend {
    type Error = Infallible;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserById(usize);

impl Request for UserById {
    type Output = Option<User>;
}

impl DataSource<UserById> for BenchBackend {
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a UserById>,
    ) -> impl Future<Output = Result<HashMap<UserById, Option<User>>, Infallible>> + Send {
        ready(Ok(reqs
            .into_iter()
            .map(|req| (req.clone(), Some(User { id: req.0 })))
            .collect()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CardById(usize);

impl Request for CardById {
    type Output = Option<Card>;
}

impl DataSource<CardById> for BenchBackend {
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a CardById>,
    ) -> impl Future<Output = Result<HashMap<CardById, Option<Card>>, Infallible>> + Send {
        ready(Ok(reqs
            .into_iter()
            .map(|req| (req.clone(), Some(Card { id: req.0 })))
            .collect()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PostsByAuthor(usize);

impl Request for PostsByAuthor {
    type Output = Vec<Post>;
}

impl DataSource<PostsByAuthor> for BenchBackend {
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a PostsByAuthor>,
    ) -> impl Future<Output = Result<HashMap<PostsByAuthor, Vec<Post>>, Infallible>> + Send {
        ready(Ok(reqs
            .into_iter()
            .map(|req| {
                (
                    req.clone(),
                    vec![Post { id: req.0 * 2 }, Post { id: req.0 * 2 + 1 }],
                )
            })
            .collect()))
    }
}

fn run_ruxl<A>(computation: impl for<'a> AsyncFnOnce(&'a Scope<BenchBackend>) -> A) -> A {
    let mut cx = Context::from_waker(Waker::noop());
    let mut future = std::pin::pin!(ruxl::run(&BenchBackend, computation));
    loop {
        if let Poll::Ready(result) = future.as_mut().poll(&mut cx) {
            return result;
        }
    }
}

fn bench_common_single_lookup(c: &mut Criterion) {
    c.bench_function("ruxl/common_single_lookup_4096_requests", |b| {
        b.iter(|| {
            for id in 0..4096 {
                black_box(run_ruxl(async |scope| {
                    scope.fetch(UserById(black_box(id))).await.unwrap()
                }));
            }
        });
    });
}

fn bench_common_list_page(c: &mut Criterion) {
    c.bench_function("ruxl/common_list_page_64_items_16_unique_512_pages", |b| {
        b.iter(|| {
            for page in 0..512 {
                let base = black_box(page * 16);
                black_box(run_ruxl(async |scope| {
                    try_join_all((0..64).map(|id| scope.fetch(UserById(base + id % 16))))
                        .await
                        .unwrap()
                }));
            }
        });
    });
}

fn bench_common_parallel_batches(c: &mut Criterion) {
    c.bench_function("ruxl/common_parallel_batches_32_each_512_pages", |b| {
        b.iter(|| {
            for page in 0..512 {
                let base = black_box(page * 32);
                black_box(run_ruxl(async |scope| {
                    try_join!(
                        try_join_all((0..32).map(|id| scope.fetch(UserById(base + id)))),
                        try_join_all((0..32).map(|id| scope.fetch(CardById(base + id)))),
                    )
                    .unwrap()
                }));
            }
        });
    });
}

fn bench_common_detail_page(c: &mut Criterion) {
    c.bench_function("ruxl/common_detail_page_3_rounds_2048_pages", |b| {
        b.iter(|| {
            for id in 0..2048 {
                black_box(run_ruxl(async |scope| {
                    let id = scope
                        .fetch(UserById(black_box(id)))
                        .await
                        .unwrap()
                        .unwrap()
                        .id;
                    let (card, posts) =
                        join!(scope.fetch(CardById(id)), scope.fetch(PostsByAuthor(id)));
                    scope
                        .fetch(CardById(card.unwrap().unwrap().id + posts.unwrap().len()))
                        .await
                        .unwrap()
                }));
            }
        });
    });
}

fn bench_common_feed_page(c: &mut Criterion) {
    c.bench_function("ruxl/common_feed_page_fan_out_fan_in_256_pages", |b| {
        b.iter(|| {
            for page in 0..256 {
                let base = black_box(page * 64);
                black_box(run_ruxl(async |scope| {
                    let (users, posts) = join!(
                        try_join_all((0..32).map(|id| scope.fetch(UserById(base + id)))),
                        try_join_all((0..16).map(|id| scope.fetch(PostsByAuthor(base + id)))),
                    );
                    let user_sum = users
                        .unwrap()
                        .into_iter()
                        .flatten()
                        .map(|user| user.id)
                        .sum::<usize>();
                    let post_sum = posts
                        .unwrap()
                        .into_iter()
                        .flatten()
                        .map(|post| post.id)
                        .sum::<usize>();
                    scope.fetch(CardById(user_sum + post_sum)).await.unwrap()
                }));
            }
        });
    });
}

fn bench_common_layered_page(c: &mut Criterion) {
    c.bench_function("ruxl/common_layered_page_48_items_128_pages", |b| {
        b.iter(|| {
            for page in 0..128 {
                let base = black_box(page * 48);
                black_box(run_ruxl(async |scope| {
                    let users = try_join_all((0..48).map(|id| scope.fetch(UserById(base + id))))
                        .await
                        .unwrap();
                    let cards = try_join_all(
                        users
                            .into_iter()
                            .flatten()
                            .map(|user| scope.fetch(CardById(user.id + 10_000))),
                    )
                    .await
                    .unwrap();
                    try_join_all(
                        cards
                            .into_iter()
                            .flatten()
                            .map(|card| scope.fetch(PostsByAuthor(card.id % 128))),
                    )
                    .await
                    .unwrap()
                }));
            }
        });
    });
}

criterion_group!(
    ruxl,
    bench_common_single_lookup,
    bench_common_list_page,
    bench_common_parallel_batches,
    bench_common_detail_page,
    bench_common_feed_page,
    bench_common_layered_page
);
criterion_main!(ruxl);
