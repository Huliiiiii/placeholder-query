use placeholder_query_macro::table;

#[table(name = "users", derive(Clone, Debug, PartialEq, Eq))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}
