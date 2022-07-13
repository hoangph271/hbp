use nanoid::nanoid;
use stargate_grpc_derive::TryFromRow;

#[derive(Debug, serde::Serialize, TryFromRow)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(serde::Deserialize)]
pub struct NewPost {
    pub title: String,
    pub body: String,
}
#[derive(serde::Deserialize)]
pub struct UpdatedPost {
    pub id: String,
    pub title: String,
    pub body: String,
    pub published: bool,
}
pub struct InsertableNewPost {
    pub id: String,
    pub title: String,
    pub body: String,
}

impl From<NewPost> for InsertableNewPost {
    fn from(new_post: NewPost) -> InsertableNewPost {
        let id = nanoid!();

        InsertableNewPost {
            id,
            title: new_post.title,
            body: new_post.body,
        }
    }
}
