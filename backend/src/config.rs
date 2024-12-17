use dotenvy::dotenv;
use env_logger::Builder;
use log::LevelFilter;
use once_cell::sync::Lazy;
use std::io::Write;
use std::{
    env,
    fs::{self, OpenOptions},
    path::Path,
    sync::Once,
};

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

static INIT: Once = Once::new();

pub fn init_log() {
    INIT.call_once(|| {
        // Укажите путь к файлу логов
        let log_file_path = "logs/server.log";

        // Если файла нет, создайте его и необходимые директории
        if !Path::new(log_file_path).exists() {
            if let Some(parent) = Path::new(log_file_path).parent() {
                fs::create_dir_all(parent).expect("Failed to create log directory");
            }
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(log_file_path)
                .expect("Failed to create log file");
        }

        // Настройка `env_logger` для записи логов в файл
        let file = OpenOptions::new()
            .append(true)
            .open(log_file_path)
            .expect("Failed to open log file");

        Builder::new()
            .format(move |buf, record| {
                writeln!(
                    buf,
                    "{} [{}] - {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .filter(None, LevelFilter::Info)
            .target(env_logger::Target::Pipe(Box::new(file)))
            .init();

        log::info!("Logger initialized and writing to {}", log_file_path);
    });
}
