pub mod lib;
pub mod models;
pub mod schema;
pub mod sqlite;

pub async fn init_db() {
    info!("--- init_db()");
    lib::user_orm::init_users_table().await;
    lib::post_orm::init_posts_table().await;
    info!("--- @init_db");
}
