use axum::{
    extract::{Path, Json},
    http::StatusCode,
    response::IntoResponse,
};
use crate::couchdb::CouchDb;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Clone)]
pub struct Herb {
    pub id: String,
    pub name: String,
    pub farmer: String,
    pub location: String,
    pub timestamp: u128,
}

#[derive(Deserialize)]
pub struct AddHerbRequest {
    pub name: String,
    pub farmer: String,
    pub location: String,
}

// Generate deterministic hash-based ID from name + farmer
fn generate_id(name: &str, farmer: &str) -> String {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    farmer.hash(&mut hasher);
    let hash = hasher.finish();
    format!("herb_{:x}", hash)
}

pub async fn root() -> &'static str {
    "Hello from Backend!"
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub async fn add_herb(
    couch: CouchDb,
    db_name: String,
    Json(payload): Json<AddHerbRequest>,
) -> impl IntoResponse {
    let id = generate_id(&payload.name, &payload.farmer);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let herb = Herb {
        id: id.clone(),
        name: payload.name,
        farmer: payload.farmer,
        location: payload.location,
        timestamp,
    };

    match couch.add_doc(&db_name, &herb).await {
        Ok(_) => (StatusCode::CREATED, Json(herb)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add herb").into_response(),
    }
}

pub async fn get_herb(
    couch: CouchDb,
    db_name: String,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match couch.get_doc::<Herb>(&db_name, &id).await {
        Ok(herb) => (StatusCode::OK, Json(herb)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Herb not found").into_response(),
    }
}

pub async fn list_herbs(
    couch: CouchDb,
    db_name: String,
) -> impl IntoResponse {
    match couch.list_docs::<Herb>(&db_name).await {
        Ok(herbs) => (StatusCode::OK, Json(herbs)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch herbs").into_response(),
    }
}