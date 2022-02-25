use crate::data::schema::tbl_posts;

#[derive(Queryable, Debug)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Insertable)]
#[table_name="tbl_posts"]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
}
