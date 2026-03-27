use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use crate::config::application::LoggingConfig;

pub fn config(config: &LoggingConfig) {
    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%dT%H:%M:%S"),
                     record.level(),
                     record.args()
            )
        })
        .filter(None, config.level.parse::<LevelFilter>().unwrap())
        .filter(Some("sqlx"), LevelFilter::Debug)
        .init();
}