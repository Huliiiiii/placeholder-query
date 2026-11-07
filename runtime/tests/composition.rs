use ruxl::Env;

struct Context;
impl Env for Context {
    type Error = ();
}

#[tokio::test]
async fn asynchronous_computations_can_borrow_external_values() {
    let value = String::from("value");
    let result = ruxl::run(&Context, async |_| value.as_str()).await;
    assert_eq!(result, "value");
}

#[tokio::test]
async fn computation_can_be_awaited_in_a_send_future() {
    let result = tokio::spawn(async {
        ruxl::run(&Context, async |_| {
            ruxl::traverse([42], async |value| Ok::<_, ()>(value)).await
        })
        .await
    })
    .await
    .unwrap();
    assert_eq!(result.unwrap(), [42]);
}
