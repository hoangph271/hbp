pub mod lib;
pub mod models;

use async_std::task;
use log::*;
use std::thread::{sleep, spawn};
use std::time::Duration;

pub mod user_orm;

pub fn init_db() {
    spawn(|| {
        info!("---@ init_db()");

        loop {
            match task::block_on(user_orm::user_orm::init_users_table()) {
                Ok(_) => break,
                Err(e) => {
                    error!("{:?}", e);
                    sleep(Duration::from_secs(10))
                }
            }
        }

        loop {
            match task::block_on(lib::post_orm::init_posts_table()) {
                Ok(_) => break,
                Err(e) => {
                    error!("{:?}", e);
                    sleep(Duration::from_secs(10))
                }
            }
        }

        info!("---# init_db()");
    });
}
