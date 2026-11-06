mod support;

use futures_util::FutureExt;
use ruxl::{Env, Fetch};

use support::executor::run;

struct Context;

impl Env for Context {
    type Error = ();
}

#[test]
fn nested_computations_preserve_dependency_order() {
    let fetch = Fetch::pure(0)
        .map(|count| count * 10 + 1)
        .and_then(|count| {
            Fetch::pure(count * 10 + 2)
                .map(|count| count * 10 + 3)
                .and_then(|count| Fetch::pure(count * 10 + 4))
                .map(|count| count * 10 + 5)
        })
        .map(|count| count * 10 + 6);

    let result = run(Context.run(fetch));

    assert_eq!(result.unwrap(), 123456);
}

#[test]
fn nested_map_chains_preserve_the_computed_value_after_yielding() {
    const INNER_DEPTH: usize = 4096;

    let chain = Fetch::pure(2)
        .and_then(|value| {
            let mut inner = Fetch::pure(value * 10);
            for _ in 0..INNER_DEPTH {
                inner = inner.map(|value| value + 1);
            }

            inner
        })
        .map(|value| value * 3);
    let mut future = std::pin::pin!(Context.run(chain));

    assert!(future.as_mut().now_or_never().is_none());

    let result = run(future);

    assert_eq!(result.unwrap(), (20 + INNER_DEPTH) * 3);
}
