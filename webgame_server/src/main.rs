mod board;
mod game;
mod server;
mod universe;
mod utils;

pub(crate) use webgame_protocol as protocol;

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();
    server::serve().await;
}
