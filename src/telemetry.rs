use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt};

pub fn init_subscriber(default_level: LevelFilter) {
    let subscriber = tracing_subscriber::registry()
        .with(fmt::layer().without_time())
        .with(default_level);

    tracing::subscriber::set_global_default(subscriber).unwrap();
}
