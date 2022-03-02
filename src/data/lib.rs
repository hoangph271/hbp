use diesel::result::Error;

#[derive(Debug)]
pub enum OrmError {
    NotFound,
    DieselError(Error),
}

pub mod post_orm {
    use crate::data::lib::OrmError;
    use crate::data::models::posts_model::*;
    use crate::data::schema::tbl_posts;
    use crate::data::schema::tbl_posts::dsl::*;
    use diesel::prelude::*;
    use diesel::result::Error;

    pub fn get_one(conn: &SqliteConnection, post_id: &str) -> Result<Post, OrmError> {
        match tbl_posts::table
            .filter(tbl_posts::id.eq(post_id))
            .first(conn)
        {
            Ok(post) => Ok(post),
            Err(e) => Err(OrmError::DieselError(e)),
        }
    }

    pub fn get_posts(conn: &SqliteConnection) -> Vec<Post> {
        tbl_posts.load(conn).expect("Error loading posts")
    }

    pub fn delete_one(conn: &SqliteConnection, post_id: &str) -> usize {
        diesel::delete(tbl_posts.filter(id.eq(post_id)))
            .execute(conn)
            .expect(&*format!("Error deleting post {}", post_id))
    }

    pub fn create_post(conn: &SqliteConnection, new_post: NewPost) -> Result<Post, Error> {
        diesel::insert_into(tbl_posts::table)
            .values(InsertableNewPost::from(new_post))
            .execute(conn)
            .expect("insert new_post failed");

        // FIXME: This is a shame, I know
        // * It's SQLite, and I'm an idiot, I don't know how to return the just inserted record
        tbl_posts::table.order(tbl_posts::id.desc()).first(conn)
    }

    pub fn update_one(
        conn: &SqliteConnection,
        updated_post: UpdatedPost,
    ) -> Result<Post, OrmError> {
        let update_result =
            diesel::update(tbl_posts.filter(tbl_posts::id.eq(updated_post.id.clone())))
                .set((
                    tbl_posts::title.eq(updated_post.title),
                    tbl_posts::body.eq(updated_post.body),
                    tbl_posts::published.eq(updated_post.published),
                ))
                .execute(conn);

        match update_result {
            Ok(val) => {
                if val == 1 {
                    get_one(conn, &updated_post.id)
                } else {
                    Err(OrmError::NotFound)
                }
            }
            Err(e) => Err(OrmError::DieselError(e)),
        }
    }
}

pub mod user_orm {
    use crate::data::lib::OrmError;
    use crate::data::models::users_model::*;
    use crate::data::schema::tbl_users;
    use diesel::prelude::*;
    use diesel::result::Error;
    use diesel::SqliteConnection;

    pub fn find_one_by_username(conn: &SqliteConnection, username: &str) -> Result<User, OrmError> {
        match tbl_users::table
            .filter(tbl_users::username.eq(username))
            .first(conn)
        {
            Ok(post) => Ok(post),
            Err(e) => Err(OrmError::DieselError(e)),
        }
    }

    #[allow(dead_code)]
    pub fn create_user(conn: &SqliteConnection, new_user: NewUser) -> Result<User, Error> {
        diesel::insert_into(tbl_users::table)
            .values(InsertableNewUser::from(new_user))
            .execute(conn)
            .expect("insert new_user failed");

        // FIXME: This is a shame, I know
        // * It's SQLite, and I'm an idiot, I don't know how to return the just inserted record
        tbl_users::table.order(tbl_users::id.desc()).first(conn)
    }
}
