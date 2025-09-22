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

#[tokio::main]
async fn main() {
    let couch = CouchDb::new("http://127.0.0.1:5984", "admin", "d**4");
    let db_name = "herbs".to_string();

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

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running at http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}