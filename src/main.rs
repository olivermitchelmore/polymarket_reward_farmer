use crate::engine::bot_manager::BotManager;
mod engine;
mod infra;
mod market;
mod types;
mod websockets;

#[tokio::main]
async fn main() {
    let bot_manager = BotManager::new().await;
    bot_manager.run();
}