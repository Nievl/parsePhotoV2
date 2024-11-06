use chrono::Utc;

pub fn get_now_time() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
