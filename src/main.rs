mod config;
mod discord;
mod monitor;
mod models;
mod template;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .init();

    config::init()?;
    
    config::CONFIG.set(config::load()?).expect("Config already initialized!");

    monitor::monitor()
}
