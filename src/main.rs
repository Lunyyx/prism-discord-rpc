mod discord;
mod monitor;
mod models;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .init();

    monitor::monitor()
}