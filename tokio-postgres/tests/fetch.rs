mod support;

use std::collections::HashMap;

use placeholder_query_macro::QueryInput;
use placeholder_query_postgres::{
    ProjectionExt,
    query::{any, select as pg},
};
use placeholder_query_runtime::{DataSource, Request, fetch, traverse};
use placeholder_query_tokio_postgres::{Error, ExecError, Executor};

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

#[tokio::test]
async fn batch_fetch_preserves_request_order() -> Result<(), ExecError> {
    let executor = seeded_executor().await?;
    let users = executor
        .run(traverse(["Mio", "Ada"], |name| {
            fetch(UserByName {
                name: name.to_owned(),
            })
        }))
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
