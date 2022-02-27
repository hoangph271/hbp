// use crate::data::schema::tbl_users;

#[derive(Queryable, Debug, serde::Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub hashed_password: String,
    pub title: Option<String>,
}
