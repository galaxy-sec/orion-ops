mod app;
mod args;
mod mcp_protocol;
// src/main.rs
use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web::Data};
use app::{AppState, get_manifest, handle_mcp_request};
use orion_syspec::infra::configure_dfx_logging;
use tracing::info;

use crate::args::TempArgs;

lazy_static::lazy_static! {
    static ref VERSION: String = env!("CARGO_PKG_VERSION").to_string();
}

// 健康检查端点
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志
    configure_dfx_logging(&TempArgs::default());

    // 创建应用状态
    let state = AppState::new();

    info!("MCP服务已启动，监听地址: 0.0.0.0:3000");

    HttpServer::new(move || {
        // 配置CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(Data::new(state.clone()))
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .service(handle_mcp_request)
            .service(get_manifest)
            .service(health_check)
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
