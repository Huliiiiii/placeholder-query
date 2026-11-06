mod support;

use criterion::{Criterion, criterion_group, criterion_main};
use support::{diesel, fixture::Fixture, rel9, sea_orm, sqlx, tokio_postgres};

fn comparison(c: &mut Criterion) {
    diesel::construction(c);
    sqlx::construction(c);
    sea_orm::construction(c);
    rel9::construction(c);

    let url = std::env::var("BENCH_DATABASE_URL")
        .expect("set BENCH_DATABASE_URL to a dedicated, disposable PostgreSQL database");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let client = runtime.block_on(support::fixture::connect(&url));
    runtime.block_on(support::fixture::create_tables(&client));

    let mut group = c.benchmark_group("load/trivial_query");
    for size in [1, 10, 100, 1_000, 10_000] {
        let fixture = Fixture::trivial(size);
        runtime.block_on(fixture.seed(&client));
        diesel::trivial(&mut group, &runtime, &url, &fixture);
        sqlx::trivial(&mut group, &runtime, &url, &fixture);
        sea_orm::trivial(&mut group, &runtime, &url, &fixture);
        tokio_postgres::trivial(&mut group, &runtime, &url, &fixture);
        rel9::trivial(&mut group, &runtime, &url, &fixture);
    }
    group.finish();

    let mut group = c.benchmark_group("load/medium_complex_query");
    for size in [1, 10, 100, 1_000, 10_000] {
        let fixture = Fixture::medium(size);
        runtime.block_on(fixture.seed(&client));
        diesel::medium(&mut group, &runtime, &url, &fixture);
        sqlx::medium(&mut group, &runtime, &url, &fixture);
        sea_orm::medium(&mut group, &runtime, &url, &fixture);
        tokio_postgres::medium(&mut group, &runtime, &url, &fixture);
        rel9::medium(&mut group, &runtime, &url, &fixture);
    }
    group.finish();

    let fixture = Fixture::associations();
    runtime.block_on(fixture.seed(&client));
    let mut group = c.benchmark_group("load/loading_associations_sequentially");
    diesel::associations(&mut group, &runtime, &url, &fixture);
    sqlx::associations(&mut group, &runtime, &url, &fixture);
    sea_orm::associations(&mut group, &runtime, &url, &fixture);
    tokio_postgres::associations(&mut group, &runtime, &url, &fixture);
    rel9::associations(&mut group, &runtime, &url, &fixture);
    group.finish();
}

criterion_group!(benches, comparison);
criterion_main!(benches);
