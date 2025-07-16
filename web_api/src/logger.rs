use std::{fs, path::{Path, PathBuf}};

use fern::Dispatch;
use log::LevelFilter;

use crate::config::LogConfig;

pub fn setup_logger(log_cfg: LogConfig) -> Result<(), fern::InitError> {
      // 自动创建日志目录
    let log_dir = Path::new(&log_cfg.path);
    fs::create_dir_all(log_dir).expect("无法创建日志目录");

    // 构建日志文件路径
    let log_path = |filename: &str| -> PathBuf {
        log_dir.join(filename)
    };

    let base_config = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {}-{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.file().unwrap_or("unknown file"),
                record.line().unwrap_or(0),
                record.level(),
                message
            ))
        })
        .level(parse_log_level(&log_cfg.level)); // 默认日志级别

    let file_dispatch = Dispatch::new()
        .level(LevelFilter::Trace)
        .chain(Dispatch::new()
            .level(LevelFilter::Error)
            .filter(|m| m.level() == log::Level::Error)
            .chain(fern::log_file(log_path("error.log"))?))
        .chain(Dispatch::new()
            .level(LevelFilter::Warn)
            .filter(|m| m.level() == log::Level::Warn)
            .chain(fern::log_file(log_path("warn.log"))?))
        .chain(Dispatch::new()
            .level(LevelFilter::Info)
            .filter(|m| m.level() == log::Level::Info)
            .chain(fern::log_file(log_path("info.log"))?))
        .chain(Dispatch::new()
            .level(LevelFilter::Debug)
            .filter(|m| m.level() == log::Level::Debug)
            .chain(fern::log_file(log_path("debug.log"))?))
        .chain(Dispatch::new()
            .level(LevelFilter::Trace)
            .filter(|m| m.level() == log::Level::Trace)
            .chain(fern::log_file(log_path("trace.log"))?));

    let logger_config = base_config
                                .chain(file_dispatch) 
                                .chain(fern::log_file("out.log")?) 
                                .chain(std::io::stdout()); // 输出到控制台
    logger_config.apply()?; // 应用日志配置
    Ok(())
}

fn parse_log_level(level: &str) -> LevelFilter{
    match level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info, // 默认值
    }
}
