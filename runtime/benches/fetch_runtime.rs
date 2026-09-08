use std::{
    collections::HashMap,
    convert::Infallible,
    future::{Future, ready},
    hint::black_box,
    task::{Context, Poll, Waker},
};

use criterion::{Criterion, criterion_group, criterion_main};
use placeholder_query_runtime::{DataSource, Fetch, FetchEnv, Request, fetch, traverse};

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserById(usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CardById(usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PostsByAuthor(usize);

struct BenchBackend;

impl FetchEnv for BenchBackend {
    type Error = Infallible;
}

impl Request for UserById {
    type Output = Option<User>;
}

impl DataSource<UserById> for BenchBackend {
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a UserById>,
    ) -> impl Future<Output = Result<HashMap<UserById, Option<User>>, Infallible>> {
        ready(Ok(reqs
            .into_iter()
            .map(|req| (req.clone(), Some(User { id: req.0 })))
            .collect()))
    }
}

impl Request for CardById {
    type Output = Option<Card>;
}

impl DataSource<CardById> for BenchBackend {
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a CardById>,
    ) -> impl Future<Output = Result<HashMap<CardById, Option<Card>>, Infallible>> {
        ready(Ok(reqs
            .into_iter()
            .map(|req| (req.clone(), Some(Card { id: req.0 })))
            .collect()))
    }
}

impl Request for PostsByAuthor {
    type Output = Vec<Post>;
}

impl DataSource<PostsByAuthor> for BenchBackend {
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a PostsByAuthor>,
    ) -> impl Future<Output = Result<HashMap<PostsByAuthor, Vec<Post>>, Infallible>> {
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

fn run_fetch<A>(fetch: Fetch<BenchBackend, A>) -> A
where
    A: 'static,
{
    let mut cx = Context::from_waker(Waker::noop());
    let mut future = std::pin::pin!(BenchBackend.run(fetch));

    // All bench sources are immediately ready; only the runtime yields between polls.
    loop {
        if let Poll::Ready(result) = future.as_mut().poll(&mut cx) {
            return result.unwrap();
        }
    }
}

fn bench_common_single_lookup(c: &mut Criterion) {
    c.bench_function("fetch/common_single_lookup_4096_requests", |b| {
        b.iter(|| {
            for id in 0..4096 {
                black_box(run_fetch(fetch(UserById(black_box(id)))));
            }
        });
    });
}

fn bench_common_list_page(c: &mut Criterion) {
    c.bench_function("fetch/common_list_page_64_items_16_unique_512_pages", |b| {
        b.iter(|| {
            for page in 0..512 {
                let base = black_box(page * 16);
                black_box(run_fetch(traverse(0..64, |id| {
                    fetch(UserById(base + id % 16))
                })));
            }
        });
    });
}

fn bench_common_parallel_batches(c: &mut Criterion) {
    c.bench_function("fetch/common_parallel_batches_32_each_512_pages", |b| {
        b.iter(|| {
            for page in 0..512 {
                let base = black_box(page * 32);
                black_box(run_fetch(
                    traverse(0..32, |id| fetch(UserById(base + id)))
                        .zip(traverse(0..32, |id| fetch(CardById(base + id)))),
                ));
            }
        });
    });
}

fn bench_common_detail_page(c: &mut Criterion) {
    c.bench_function("fetch/common_detail_page_3_rounds_2048_pages", |b| {
        b.iter(|| {
            for id in 0..2048 {
                black_box(run_fetch(
                    fetch(UserById(black_box(id)))
                        .and_then(|user| {
                            let id = user.unwrap().id;
                            fetch(CardById(id)).zip(fetch(PostsByAuthor(id)))
                        })
                        .and_then(|(card, posts)| fetch(CardById(card.unwrap().id + posts.len()))),
                ));
            }
        });
    });
}

fn bench_common_feed_page(c: &mut Criterion) {
    c.bench_function("fetch/common_feed_page_fan_out_fan_in_256_pages", |b| {
        b.iter(|| {
            for page in 0..256 {
                let base = black_box(page * 64);
                black_box(run_fetch(
                    traverse(0..32, |id| fetch(UserById(base + id)))
                        .zip(traverse(0..16, |id| fetch(PostsByAuthor(base + id))))
                        .and_then(|(users, posts)| {
                            let user_sum = users
                                .into_iter()
                                .flatten()
                                .map(|user| user.id)
                                .sum::<usize>();
                            let post_sum = posts
                                .into_iter()
                                .flatten()
                                .map(|post| post.id)
                                .sum::<usize>();

                            fetch(CardById(user_sum + post_sum))
                        }),
                ));
            }
        });
    });
}

fn bench_common_layered_page(c: &mut Criterion) {
    c.bench_function("fetch/common_layered_page_48_items_128_pages", |b| {
        b.iter(|| {
            for page in 0..128 {
                let base = black_box(page * 48);
                black_box(run_fetch(
                    traverse(0..48, |id| fetch(UserById(base + id)))
                        .and_then(|users| {
                            let card_ids = users.into_iter().flatten().map(|user| user.id + 10_000);

                            traverse(card_ids, |id| fetch(CardById(id)))
                        })
                        .and_then(|cards| {
                            let author_ids = cards.into_iter().flatten().map(|card| card.id % 128);

                            traverse(author_ids, |id| fetch(PostsByAuthor(id)))
                        }),
                ));
            }
        });
    });
}

criterion_group!(
    fetch_runtime,
    bench_common_single_lookup,
    bench_common_list_page,
    bench_common_parallel_batches,
    bench_common_detail_page,
    bench_common_feed_page,
    bench_common_layered_page
);
criterion_main!(fetch_runtime);
