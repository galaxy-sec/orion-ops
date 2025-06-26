
use lazy_static::lazy_static;

use std::{
    path::Path,
};
use tracing::{info, error, Level};
use tracing_subscriber::{
    fmt::{self, format::Writer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
// src/logger.rs

use anyhow::Result;
// 添加这行导入
use tracing_subscriber::filter::EnvFilter;
lazy_static! {
    static ref VERSION: String = env!("CARGO_PKG_VERSION").to_string();
    static ref LOG_DIR: String = "logs".to_string();
}
// 初始化日志系统
pub fn init_logging() -> Result<()> {
    // 创建日志目录
    if !Path::new(&*LOG_DIR).exists() {
        std::fs::create_dir_all(&*LOG_DIR)?;
    }
    
    // 创建每日滚动日志文件
    let file_appender = tracing_appender::rolling::daily(&*LOG_DIR, "mcp_server.log");
    
    // 控制台输出层
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        //.with_timer(time_format())
        .with_ansi(true)
        .with_level(true)
        .with_target(true);
    
    // 文件输出层
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        //.with_timer(time_format())
        .with_ansi(false)
        .with_level(true)
        .with_target(true);
    
    // 设置日志过滤器
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))?;
    
    // 注册全局订阅者
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(console_layer)
        .with(file_layer)
        .init();
    
    Ok(())
}