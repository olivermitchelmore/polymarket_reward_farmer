use std::io;
use std::io::Write;
use crate::engine::bot_manager::BotManager;
use crate::infra::{ConfigParams, SigningUtils};

mod engine;
mod infra;
mod market_logic;
mod types;
mod websockets;

#[tokio::main]
async fn main() {
    println!("This project is under active development and not suitable for use with real funds. \nPress enter to continue anyway");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");


    let config_params = ConfigParams::new().expect("Failed to get config params");
    let signing_utils = SigningUtils::new_client(&config_params.private_key, &config_params.funder_address)
                .await
                .expect("Failed to create signing utils");

    let bot_manager = BotManager::new(config_params, signing_utils).await;
    bot_manager.run();
}