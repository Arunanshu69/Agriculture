mod handlers;
mod couchdb;

use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use handler::*;
use couchdb::CouchDb;

#[tokio::main]
async fn main() {
    // CouchDB client
    let couch = CouchDb::new("http://127.0.0.1:5984", "admin", "d**4");
    let db_name = "herbs".to_string();

    // Build app routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/addHerb", post({
            let couch = couch.clone();
            let db_name = db_name.clone();
            move |Json(payload)| add_herb(couch.clone(), db_name.clone(), payload)
        }))
        .route("/getHerb/{id}", get({
            let couch = couch.clone();
            let db_name = db_name.clone();
            move |Path(id)| get_herb(couch.clone(), db_name.clone(), id)
        }))
        .route("/listHerbs", get({
            let couch = couch.clone();
            let db_name = db_name.clone();
            move || list_herbs(couch.clone(), db_name.clone())
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running at http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}