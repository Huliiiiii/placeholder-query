mod support;

use placeholder_query_macro::{QueryInput, table};
use placeholder_query_postgres::query::select as pg;
use placeholder_query_tokio_postgres::ExecError;

use support::{schema::User, seeded_executor};

#[table(name = "user_labels", derive(Clone, Debug, PartialEq, Eq))]
struct UserLabel {
    user_id: i32,
    label: Option<String>,
}

#[tokio::test]
async fn optional_map_preserves_null_and_missing() -> Result<(), ExecError> {
    let executor = seeded_executor().await?;
    executor
        .batch_execute(
            r#"
CREATE TEMP TABLE user_labels (
    user_id integer NOT NULL,
    label text
);

INSERT INTO user_labels (user_id, label) VALUES
    (1, NULL),
    (2, 'primary');
"#,
        )
        .await?;

    let query = pg::from(User::table())
        .left_join(UserLabel::table())
        .on(|user, label| user.id().eq(label.user_id()))
        .order_by(|(user, _)| user.id().asc())
        .limit(3)
        .map(|(_, labels)| labels.map(|label| label.label));

    assert_eq!(
        executor.run(query).await?,
        [Some(None), Some(Some("primary".to_owned())), None]
    );
    Ok(())
}

#[tokio::test]
async fn nested_optional_mapping_preserves_column_alignment() -> Result<(), ExecError> {
    let executor = seeded_executor().await?;
    let query = pg::from(User::table())
        .left_join_lateral(|outer| {
            pg::from(
                pg::from(User::table())
                    .filter(|middle| middle.id().gt(outer.id()))
                    .order_by(|middle| middle.id().asc())
                    .limit(1),
            )
            .left_join_lateral(|middle| {
                pg::from(User::table())
                    .filter(|inner| inner.id().gt(middle.id()))
                    .order_by(|inner| inner.id().asc())
                    .limit(1)
            })
            .on_true()
        })
        .on_true()
        .order_by(|(outer, _)| outer.id().desc())
        .limit(3)
        .map(|(outer, middle)| {
            (
                middle.map(|(_, inner)| inner.map(|inner| inner.id)),
                outer.id,
            )
        });

    assert_eq!(
        executor.run(query).await?,
        [(None, 7), (Some(None), 3), (Some(Some(7)), 2)]
    );
    Ok(())
}
