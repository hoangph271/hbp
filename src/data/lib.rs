use crate::data::models::{NewPost, Post};
use diesel::sqlite::SqliteConnection;

pub fn create_post<'a>(conn: &SqliteConnection, title: &'a str, body: &'a str) -> Post {
    use crate::data::schema::tbl_posts;
    use diesel::*;

    diesel::insert_into(tbl_posts::table)
        .values(&NewPost { title, body })
        .execute(conn)
        .expect("insert new_post failed");

    let new_post: Post = tbl_posts::table
        .order(tbl_posts::id.desc())
        .first(conn)
        .unwrap();

    println!("{:?}", new_post);

    new_post
}
