mod support;

use placeholder_query_macro::QueryInput;
use placeholder_query_postgres::query::select as pg;
use placeholder_query_tokio_postgres::ExecError;

use support::{schema::User, seeded_executor};

#[tokio::test]
async fn plain_selects_are_executable() -> Result<(), ExecError> {
    let executor = seeded_executor().await?;
    let query = pg::from(User::table())
        .order_by(|user| user.id().asc())
        .map(|user| user.id);

    assert_eq!(executor.run(query).await?, [1, 2, 3, 7]);
    Ok(())
}
