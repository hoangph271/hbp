pub mod post_orm {
    use crate::data::models::*;
    use diesel::prelude::*;
    use diesel::sqlite::SqliteConnection;

    pub fn get_posts(conn: &SqliteConnection) -> Vec<Post> {
        use crate::data::schema::tbl_posts::dsl::*;
        tbl_posts.load(conn).expect("Error loading posts")
    }

    pub fn create_post<'a>(conn: &SqliteConnection, title: &'a str, body: &'a str) -> Post {
        use crate::data::schema::tbl_posts;
        use diesel::*;

        diesel::insert_into(tbl_posts::table)
            .values(&NewPost { title, body })
            .execute(conn)
            .expect("insert new_post failed");

        // FIXME: This is a shame, I know
        let new_post: Post = tbl_posts::table
            .order(tbl_posts::id.desc())
            .first(conn)
            .unwrap();
        println!("{:?}", new_post);
        new_post
    }
}
