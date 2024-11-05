use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::env;

// Lazy-initialized static variables for configuration
pub static PORT: Lazy<u16> = Lazy::new(|| {
    env::var("PORT")
        .expect("PORT must be set")
        .parse()
        .expect("PORT must be a number")
});

pub static DB_NAME: Lazy<String> = Lazy::new(|| env::var("DB_NAME").expect("DB_NAME must be set"));

pub static ROOT_URL: Lazy<String> =
    Lazy::new(|| env::var("ROOT_URL").expect("ROOT_URL must be set"));

pub static EXTENSIONS: Lazy<Vec<String>> = Lazy::new(|| {
    env::var("EXTENSIONS")
        .expect("EXTENSIONS must be set")
        .split(',')
        .map(|ext| ext.trim().to_string())
        .collect()
});

/// Initializes the configuration by loading environment variables
pub fn init() {
    dotenv().ok();

    Lazy::force(&PORT);
    Lazy::force(&DB_NAME);
    Lazy::force(&ROOT_URL);
    Lazy::force(&EXTENSIONS);
}
