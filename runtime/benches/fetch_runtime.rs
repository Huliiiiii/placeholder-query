use std::{
    convert::Infallible,
    future::{Future, ready},
    hint::black_box,
};

use criterion::{Criterion, criterion_group, criterion_main};
use futures_util::FutureExt;
use indexmap::IndexMap;
use placeholder_query_runtime::{DataSource, Fetch, FetchEnv, FetchKey};

#[derive(Clone, Debug, PartialEq, Eq)]
struct User {
    id: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Card {
    id: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Post {
    id: usize,
    author_id: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UserById(usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CardById(usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PostsByAuthor(usize);

enum Request {
    Users(Vec<usize>),
    Cards(Vec<usize>),
    Posts(Vec<usize>),
}

enum Row {
    User(User),
    Card(Card),
    Post(Post),
}

struct BenchBackend;

impl FetchEnv for BenchBackend {
    type Error = Infallible;
}

impl FetchKey for UserById {
    type Output = Option<User>;
}

impl DataSource<UserById> for BenchBackend {
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [UserById],
    ) -> impl Future<Output = Result<IndexMap<UserById, Option<User>>, Infallible>> + 'a {
        let request = Request::Users(keys.iter().map(|key| key.0).collect());
        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, None))
            .collect::<IndexMap<_, _>>();

        ready(serve_request(request).map(|rows| {
            for row in rows {
                let Row::User(user) = row else {
                    continue;
                };
                values.insert(UserById(user.id), Some(user));
            }

            values
        }))
    }
}

impl FetchKey for CardById {
    type Output = Option<Card>;
}

impl DataSource<CardById> for BenchBackend {
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [CardById],
    ) -> impl Future<Output = Result<IndexMap<CardById, Option<Card>>, Infallible>> + 'a {
        let request = Request::Cards(keys.iter().map(|key| key.0).collect());
        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, None))
            .collect::<IndexMap<_, _>>();

        ready(serve_request(request).map(|rows| {
            for row in rows {
                let Row::Card(card) = row else {
                    continue;
                };
                values.insert(CardById(card.id), Some(card));
            }

            values
        }))
    }
}

impl FetchKey for PostsByAuthor {
    type Output = Vec<Post>;
}

impl DataSource<PostsByAuthor> for BenchBackend {
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [PostsByAuthor],
    ) -> impl Future<Output = Result<IndexMap<PostsByAuthor, Vec<Post>>, Infallible>> + 'a {
        let request = Request::Posts(keys.iter().map(|key| key.0).collect());
        let mut values = keys
            .iter()
            .cloned()
            .map(|key| (key, Vec::new()))
            .collect::<IndexMap<_, _>>();

        ready(serve_request(request).map(|rows| {
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
        }))
    }
}

fn serve_request(request: Request) -> Result<Vec<Row>, Infallible> {
    Ok(match request {
        Request::Users(ids) => ids.into_iter().map(|id| Row::User(User { id })).collect(),
        Request::Cards(ids) => ids.into_iter().map(|id| Row::Card(Card { id })).collect(),
        Request::Posts(author_ids) => author_ids
            .into_iter()
            .flat_map(|author_id| {
                [
                    Row::Post(Post {
                        id: author_id * 2,
                        author_id,
                    }),
                    Row::Post(Post {
                        id: author_id * 2 + 1,
                        author_id,
                    }),
                ]
            })
            .collect(),
    })
}

fn run_fetch<A>(fetch: Fetch<BenchBackend, A>) -> A
where
    A: 'static,
{
    fetch
        .run(&BenchBackend)
        .now_or_never()
        .expect("bench backend should return ready futures")
        .unwrap()
}

fn bench_common_single_lookup(c: &mut Criterion) {
    c.bench_function("fetch/common_single_lookup_4096_requests", |b| {
        b.iter(|| {
            let checksum = (0..4096)
                .map(|id| {
                    run_fetch(Fetch::new(|cx| cx.fetch(UserById(black_box(id)))))
                        .unwrap()
                        .id
                })
                .sum::<usize>();
            black_box(checksum);
        });
    });
}

fn bench_common_list_page(c: &mut Criterion) {
    c.bench_function("fetch/common_list_page_64_items_16_unique_512_pages", |b| {
        b.iter(|| {
            let checksum = (0..512)
                .map(|page| {
                    let base = black_box(page * 16);
                    run_fetch(Fetch::new(|cx| {
                        cx.traverse(0..64, |id, cx| cx.fetch(UserById(base + id % 16)))
                    }))
                    .into_iter()
                    .flatten()
                    .map(|user| user.id)
                    .sum::<usize>()
                })
                .sum::<usize>();
            black_box(checksum);
        });
    });
}

fn bench_common_parallel_batches(c: &mut Criterion) {
    c.bench_function("fetch/common_parallel_batches_32_each_512_pages", |b| {
        b.iter(|| {
            let checksum = (0..512)
                .map(|page| {
                    let base = black_box(page * 32);
                    let (users, cards) = run_fetch(Fetch::new(|cx| {
                        cx.traverse(0..32, |id, cx| cx.fetch(UserById(base + id)))
                            .zip(cx.traverse(0..32, |id, cx| cx.fetch(CardById(base + id))))
                    }));

                    users
                        .into_iter()
                        .flatten()
                        .map(|user| user.id)
                        .sum::<usize>()
                        + cards
                            .into_iter()
                            .flatten()
                            .map(|card| card.id)
                            .sum::<usize>()
                })
                .sum::<usize>();
            black_box(checksum);
        });
    });
}

fn bench_common_detail_page(c: &mut Criterion) {
    c.bench_function("fetch/common_detail_page_3_rounds_2048_pages", |b| {
        b.iter(|| {
            let checksum = (0..2048)
                .map(|id| {
                    run_fetch(Fetch::new(|cx| {
                        cx.fetch(UserById(black_box(id)))
                            .and_then(|user, cx| {
                                let id = user.unwrap().id;
                                cx.fetch(CardById(id)).zip(cx.fetch(PostsByAuthor(id)))
                            })
                            .and_then(|(card, posts), cx| {
                                cx.fetch(CardById(card.unwrap().id + posts.len()))
                            })
                    }))
                    .unwrap()
                    .id
                })
                .sum::<usize>();
            black_box(checksum);
        });
    });
}

fn bench_common_feed_page(c: &mut Criterion) {
    c.bench_function("fetch/common_feed_page_fan_out_fan_in_256_pages", |b| {
        b.iter(|| {
            let checksum = (0..256)
                .map(|page| {
                    let base = black_box(page * 64);
                    run_fetch(Fetch::new(|cx| {
                        cx.traverse(0..32, |id, cx| cx.fetch(UserById(base + id)))
                            .zip(cx.traverse(0..16, |id, cx| cx.fetch(PostsByAuthor(base + id))))
                            .and_then(|(users, posts), cx| {
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

                                cx.fetch(CardById(user_sum + post_sum))
                            })
                    }))
                    .unwrap()
                    .id
                })
                .sum::<usize>();
            black_box(checksum);
        });
    });
}

fn bench_common_layered_page(c: &mut Criterion) {
    c.bench_function("fetch/common_layered_page_48_items_128_pages", |b| {
        b.iter(|| {
            let checksum = (0..128)
                .map(|page| {
                    let base = black_box(page * 48);
                    run_fetch(Fetch::new(|cx| {
                        cx.traverse(0..48, |id, cx| cx.fetch(UserById(base + id)))
                            .and_then(|users, cx| {
                                let card_ids = users
                                    .into_iter()
                                    .flatten()
                                    .map(|user| user.id + 10_000)
                                    .collect::<Vec<_>>();

                                cx.traverse(card_ids, |id, cx| cx.fetch(CardById(id)))
                            })
                            .and_then(|cards, cx| {
                                let author_ids = cards
                                    .into_iter()
                                    .flatten()
                                    .map(|card| card.id % 128)
                                    .collect::<Vec<_>>();

                                cx.traverse(author_ids, |id, cx| cx.fetch(PostsByAuthor(id)))
                            })
                    }))
                    .into_iter()
                    .flatten()
                    .map(|post| post.id)
                    .sum::<usize>()
                })
                .sum::<usize>();
            black_box(checksum);
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
