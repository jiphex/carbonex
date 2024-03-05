use std::net::SocketAddr;

#[derive(serde::Deserialize)]
pub struct Config {
    pub postcodes: Vec<String>,
    pub listen_addr: SocketAddr,
}

pub mod types;