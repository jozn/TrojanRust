pub mod acceptor;
mod handler;
pub mod server;

use dashmap::DashMap;
use std::sync::Arc;

pub struct UserMem {
    pub password: String,
    pub last_ip: String,
    pub ips: String, // vec
    pub block_to: i64,
    pub ip_moving_avg: f64,
}

pub struct UserHolder {
    pub mp: DashMap<String, UserMem>,
}

pub type UserHolderArc = Arc<UserHolder>;
