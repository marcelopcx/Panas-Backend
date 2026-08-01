use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use backend::config::AppConfig;
use backend::services::chat::ChatHub;
use backend::{db, routes};
use std::net::UdpSocket;
use std::sync::Arc;

fn local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env();
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("No se pudo conectar a la base de datos");

    let hub = Arc::new(ChatHub::new());

    let host = config.host.clone();
    let port = config.port;

    let printed_host = if host == "0.0.0.0" {
        local_ip().unwrap_or_else(|| host.clone())
    } else {
        host.clone()
    };

    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::from(hub.clone()))
            .configure(routes::configure)
    })
    .bind((host.as_str(), port))?;

    println!("Servidor listo en http://{}:{}", printed_host, port);
    println!("WebSocket chat: ws://{}:{}/ws/chats/{{id}}?token=<JWT>", printed_host, port);

    server.run().await
}
