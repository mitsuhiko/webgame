mod game;

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();
    game::serve().await;
}
