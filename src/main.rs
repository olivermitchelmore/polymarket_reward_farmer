use std::io;
use std::io::Write;
use crate::engine::bot_manager::BotManager;
mod engine;
mod infra;
mod market;
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

    let bot_manager = BotManager::new().await;
    bot_manager.run();
}