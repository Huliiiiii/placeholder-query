mod support;

use std::collections::HashMap;

use futures_util::future::try_join_all;
use rel9::{
    ProjectionExt,
    query::{any, select as pg},
};
use rel9_macro::QueryInput;
use rel9_tokio_postgres::{Error, ExecError, Executor};
use ruxl::{DataSource, Request};

use support::{schema::User, seeded_executor};

#[derive(QueryInput, Clone, Debug, PartialEq, Eq, Hash)]
struct UserByName {
    name: String,
}

impl Request for UserByName {
    type Output = Option<User>;
}

impl DataSource<UserByName> for Executor {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a UserByName>,
    ) -> Result<HashMap<UserByName, Option<User>>, Error> {
        let query = pg::query::<[UserByName], _>(|reqs| {
            pg::from(User::table())
                .filter(|user| user.name().eq(any(reqs.name)))
                .map(|user| {
                    user.map(|user: User| {
                        (
                            UserByName {
                                name: user.name.clone(),
                            },
                            user,
                        )
                    })
                })
        })
        .compile();

        self.fetch_batch(&query, reqs).await
    }
}

#[test]
fn run_future_is_send() {
    fn assert_send(_: impl Send) {}

    let _ = |executor: &Executor| {
        assert_send(ruxl::run(executor, async |scope| {
            scope
                .fetch(UserByName {
                    name: String::from("Mio"),
                })
                .await
        }));
    };
}

#[tokio::test]
async fn batch_fetch_preserves_request_order() -> Result<(), ExecError> {
    let executor = seeded_executor().await?;
    let users = ruxl::run(&executor, async |scope| {
        try_join_all(["Mio", "Ada"].into_iter().map(|name| {
            scope.fetch(UserByName {
                name: name.to_owned(),
            })
        }))
        .await
    })
    .await?;

    assert_eq!(
        users
            .into_iter()
            .map(|user| user.map(|user| user.id))
            .collect::<Vec<_>>(),
        [Some(7), Some(1)]
    );

    Ok(())
}
