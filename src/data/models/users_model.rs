use crate::data::schema::tbl_users;
use nanoid::nanoid;

#[derive(Queryable, Debug, serde::Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub hashed_password: String,
    pub title: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub hashed_password: &'a str,
    pub title: Option<&'a str>,
}
#[derive(Insertable)]
#[table_name = "tbl_users"]
pub struct InsertableNewUser {
    pub id: String,
    pub username: String,
    pub hashed_password: String,
    pub title: Option<String>,
}

impl<'a> From<NewUser<'a>> for InsertableNewUser {
    fn from(new_user: NewUser) -> InsertableNewUser {
        let id = nanoid!();

        InsertableNewUser {
            id,
            username: new_user.username.to_owned(),
            hashed_password: new_user.hashed_password.to_owned(),
            title: Some(new_user.username.to_owned()),
        }
    }
}
