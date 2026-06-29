use std::{fs, net::IpAddr, str::FromStr};
use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[derive(Debug, Deserialize, Serialize)]
struct SocketConfig {
    ip: String,
    port: u16,
}

fn config_exists() -> bool {
    fs::metadata("configs/server.toml").is_ok()
}

fn write_config(cfg: &SocketConfig) {
    let toml_str = toml::to_string(cfg)
        .expect("Не вдалося серіалізувати SocketConfig");

    fs::create_dir_all("configs")
        .expect("Не вдалося створити папку configs");

    fs::write("configs/server.toml", toml_str)
        .expect("Не вдалося записати server.toml");
}

async fn read_line(prompt: &str) -> String {
    println!("{}", prompt);

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();

    reader.read_line(&mut input)
        .await
        .expect("Помилка читання вводу");

    input.trim().to_string()
}

pub async fn init_server_config_first_run() {
    if config_exists() {
        return;
    }


    let ip = loop {
        let input = read_line("Введіть IP цього серверу\nнаприклад 127.0.0.1 (це сервер для локального компютера)\nабо 0.0.0.0 (щоб можна було достукатись по Ip цього компютера):").await;

        if IpAddr::from_str(&input).is_ok() {
            break input;
        } else {
            println!("❌ Некоректний IP, спробуйте ще раз");
        }
    };

    let port = loop {
        let input = read_line("Введіть порт (0-65535):").await;

        match input.parse::<u16>() {
            Ok(p) => break p,
            Err(_) => println!("❌ Некоректний порт"),
        }
    };

    let cfg = SocketConfig { ip, port };

    write_config(&cfg);

    println!("✔ Конфіг server.toml створено");
}