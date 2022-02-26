use crate::data::schema::tbl_posts;

#[derive(Queryable, Debug, serde::Serialize)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(serde::Deserialize)]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
}
#[derive(serde::Deserialize)]
pub struct UpdatedPost<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    pub published: bool,
}
#[derive(Insertable)]
#[table_name = "tbl_posts"]
pub struct InsertableNewPost {
    pub id: String,
    pub title: String,
    pub body: String,
}

use nanoid::nanoid;
impl<'a> From<NewPost<'a>> for InsertableNewPost {
    fn from(new_post: NewPost) -> InsertableNewPost {
        let id = nanoid!();

        InsertableNewPost {
            id,
            title: new_post.title.to_owned(),
            body: new_post.body.to_owned(),
        }
    }
}
