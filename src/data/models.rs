// pub trait IdedEntity {
//     fn get_id (&self) -> String;
// }

#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

// impl IdedEntity for Post {
//     fn get_id(&self) -> String {
//         self.id.clone()
//     }
// }
