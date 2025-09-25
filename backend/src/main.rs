mod handlers;
mod couchdb;

use axum::{
    Router,
    routing::{get, post, delete},
};
use std::net::SocketAddr;
use handlers::*;
use couchdb::CouchDb;
use tower_http::cors::{CorsLayer, Any};
use http::Method;
use std::env;

#[tokio::main]
async fn main() {
    // Load .env if present
    let _ = dotenvy::dotenv();

    let couch_url = env::var("COUCHDB_URL").unwrap_or_else(|_| "http://127.0.0.1:5984".to_string());
    let couch_user = env::var("COUCHDB_USER").unwrap_or_else(|_| "admin".to_string());
    let couch_pass = env::var("COUCHDB_PASS").unwrap_or_else(|_| "d**4".to_string());
    let db_name = env::var("COUCHDB_DB").unwrap_or_else(|_| "herbs".to_string());

    let couch = CouchDb::new(&couch_url, &couch_user, &couch_pass);

    let state = handlers::AppState { couch: couch.clone(), db_name: db_name.clone() };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/addHerb", post(add_herb))
        .route("/getHerb/{id}", get(get_herb))
        .route("/p/{id}", get(get_public_product))
        .route("/p/{id}/html", get(get_public_product_html))
        .route("/qr/{id}", get(get_qr_png))
        .route("/scan", post(scan_product))
        .route("/scan-page", get(scan_page))
        .route("/listHerbs", get(list_herbs))
        .route("/deleteHerb/{id}", delete(delete_herb))
        .route("/updateHerb/{id}", axum::routing::put(update_herb))
        .route("/resetDb", post(reset_db))
        .with_state(state)
        .layer(cors);

    let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(3000);
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let ip: std::net::IpAddr = host.parse().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127,0,0,1)));
    let addr = SocketAddr::from((ip, port));
    println!("🚀 Server running at http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}