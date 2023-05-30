pub mod acceptor;
mod handler;
pub mod server;

use crate::server::acceptor::password_to_sha2_hex;
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
    pub secrets: DashMap<String, bool>,
}

pub type UserHolderArc = Arc<UserHolder>;

impl UserHolder {
    pub fn add_secrets(&mut self, arr: &Vec<String>) {
        for pass in arr {
            let hex = password_to_sha2_hex(pass);
            self.secrets.insert(hex, true);
        }
    }

    pub fn add_secret(&mut self, pass: &str) {
            let hex = password_to_sha2_hex(pass);
            self.secrets.insert(hex, true);
    }
}
