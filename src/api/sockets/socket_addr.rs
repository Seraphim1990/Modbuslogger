use std::fs;
use std::net::{IpAddr, SocketAddr};
use serde::Deserialize;
use std::str::FromStr;


#[derive(Debug, Deserialize)]
struct SocketConfig{
    ip: String,
    port: u16,
}

pub fn init_socket() -> SocketAddr {
    let config_contents = fs::read_to_string("../../../configs/server.toml")
        .expect("Не вдалося прочитати файл server.toml");

    let config: SocketConfig = toml::from_str(&config_contents)
        .expect("Не вдалося десеріалізувати TOML");

    // Парсимо рядок IP у тип IpAddr
    let ip = IpAddr::from_str(&config.ip)
        .expect("Некоректний формат IP адреси в конфігурації");

    // Створюємо SocketAddr
    SocketAddr::new(ip, config.port)
}