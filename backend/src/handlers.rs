use axum::{
    extract::{Path, Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::Html;
use axum::http::header;
use crate::couchdb::CouchDb;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use qrcode::QrCode;
use image::{Luma, ImageFormat};
use std::io::Cursor;
use base64::{engine::general_purpose, Engine as _};
use std::env;
use url::Url;

#[derive(Serialize, Deserialize, Clone)]
pub struct Herb {
    pub id: String,
    pub name: String,
    pub farmer: String,
    pub location: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HerbWithQr {
    #[serde(flatten)]
    pub herb: Herb,
    pub qr_code: String, // Base64 PNG
}

#[derive(Deserialize)]
pub struct AddHerbRequest {
    pub name: String,
    pub farmer: String,
    pub location: String,
}

impl AddHerbRequest {
    pub fn validate(&self) -> Result<(), String> {
        fn non_empty_trimmed(s: &str) -> bool { !s.trim().is_empty() }
        if !non_empty_trimmed(&self.name) { return Err("name is required".to_string()); }
        if !non_empty_trimmed(&self.farmer) { return Err("farmer is required".to_string()); }
        if !non_empty_trimmed(&self.location) { return Err("location is required".to_string()); }
        if self.name.len() > 100 { return Err("name too long (max 100)".to_string()); }
        if self.farmer.len() > 100 { return Err("farmer too long (max 100)".to_string()); }
        if self.location.len() > 200 { return Err("location too long (max 200)".to_string()); }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct ScanRequest {
    pub data: String,
}

#[derive(Deserialize)]
pub struct UpdateHerbRequest {
    pub name: Option<String>,
    pub farmer: Option<String>,
    pub location: Option<String>,
}

// AppState to hold CouchDB client and database name
#[derive(Clone)]
pub struct AppState {
    pub couch: CouchDb,
    pub db_name: String,
}

// Generate deterministic hash-based ID from name + farmer
fn generate_id(name: &str, farmer: &str) -> String {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    farmer.hash(&mut hasher);
    let hash = hasher.finish();
    format!("herb_{:x}", hash)
}

// Generate Base64 PNG QR code from full herb details (JSON string)
fn generate_qr_base64(herb: &Herb) -> String {
    // If PUBLIC_BASE_URL is set, encode a URL to the public product page; otherwise embed JSON
    let payload = if let Ok(base) = env::var("PUBLIC_BASE_URL") {
        format!("{}/p/{}", base.trim_end_matches('/'), herb.id)
    } else {
        serde_json::to_string(herb).unwrap()
    };
    let code = QrCode::new(payload).unwrap();
    let image = code.render::<Luma<u8>>().build();
    let mut buffer: Vec<u8> = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buffer);
        image::DynamicImage::ImageLuma8(image)
            .write_to(&mut cursor, ImageFormat::Png)
            .unwrap();
    }
    general_purpose::STANDARD.encode(&buffer)
}

fn generate_qr_png_bytes(herb: &Herb) -> Vec<u8> {
    let payload = if let Ok(base) = env::var("PUBLIC_BASE_URL") {
        format!("{}/p/{}", base.trim_end_matches('/'), herb.id)
    } else {
        serde_json::to_string(herb).unwrap()
    };
    let code = QrCode::new(payload).unwrap();
    let image = code.render::<Luma<u8>>().build();
    let mut buffer: Vec<u8> = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buffer);
        image::DynamicImage::ImageLuma8(image)
            .write_to(&mut cursor, ImageFormat::Png)
            .unwrap();
    }
    buffer
}

// Handlers

// GET /
pub async fn root() -> &'static str {
    "Hello from Backend!"
}

// GET /health
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

// POST /resetDb
pub async fn reset_db(State(state): State<AppState>) -> impl IntoResponse {
    match state.couch.reset_db(&state.db_name).await {
        Ok(_) => (StatusCode::OK, "Database reset successfully").into_response(),
        Err(e) => {
            eprintln!("reset_db failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to reset database").into_response()
        },
    }
}

// POST /addHerb
pub async fn add_herb(
    State(state): State<AppState>,
    Json(payload): Json<AddHerbRequest>,
) -> impl IntoResponse {
    if let Err(msg) = payload.validate() {
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }
    let id = generate_id(&payload.name, &payload.farmer);
    let created_at = Utc::now();

    let herb = Herb {
        id: id.clone(),
        name: payload.name,
        farmer: payload.farmer,
        location: payload.location,
        created_at,
    };

    // Save to CouchDB; if exists, fetch and return existing plain herb instead of erroring
    if let Err(e) = state.couch.add_doc(&state.db_name, &id, &herb).await {
        eprintln!("add_doc failed for id {}: {}", id, e);
        match state.couch.get_doc::<Herb>(&state.db_name, &id).await {
            Ok(existing) => {
                return (StatusCode::OK, Json(existing)).into_response();
            }
            Err(fetch_err) => {
                eprintln!("Failed to add and then fetch herb {}: {} / fetch err: {}", id, e, fetch_err);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add herb").into_response();
            }
        }
    }

    // Return plain herb; QR generated only on demand via get
    (StatusCode::CREATED, Json(herb)).into_response()
}

// GET /getHerb/{id}
pub async fn get_herb(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.couch.get_doc::<Herb>(&state.db_name, &id).await {
        Ok(herb) => {
            let qr_base64 = generate_qr_base64(&herb);
            let herb_with_qr = HerbWithQr {
                herb,
                qr_code: format!("data:image/png;base64,{}", qr_base64),
            };
            (StatusCode::OK, Json(herb_with_qr)).into_response()
        },
        Err(err) => {
            eprintln!("get_herb failed for id {}: {}", id, err);
            (StatusCode::NOT_FOUND, "Herb not found").into_response()
        },
    }
}

// GET /p/{id} - Public product endpoint (no QR data), suitable for QR landing page
pub async fn get_public_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.couch.get_doc::<Herb>(&state.db_name, &id).await {
        Ok(herb) => (StatusCode::OK, Json(herb)).into_response(),
        Err(err) => {
            eprintln!("get_public_product failed for id {}: {}", id, err);
            (StatusCode::NOT_FOUND, "Product not found").into_response()
        },
    }
}

// GET /p/{id}/html - Simple HTML landing page for a product
pub async fn get_public_product_html(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.couch.get_doc::<Herb>(&state.db_name, &id).await {
        Ok(herb) => {
            let html = format!(
                "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{name}</title><style>body{{font-family:sans-serif;margin:24px;}}.card{{max-width:640px;border:1px solid #eee;border-radius:12px;padding:20px;box-shadow:0 2px 8px rgba(0,0,0,0.06);}}.row{{margin:6px 0}}code,a{{color:#0a6;word-break:break-all}}</style></head><body><div class=\"card\"><h1>{name}</h1><div class=\"row\"><strong>Farmer:</strong> {farmer}</div><div class=\"row\"><strong>Location:</strong> {location}</div><div class=\"row\"><strong>ID:</strong> <code>{id}</code></div><div class=\"row\"><img alt=\"QR\" src=\"/qr/{id}\" style=\"margin-top:12px;max-width:240px\"/></div><hr/><div class=\"row\"><a href=\"/p/{id}\">View JSON</a></div></div></body></html>",
                name = herb.name,
                farmer = herb.farmer,
                location = herb.location,
                id = herb.id
            );
            (StatusCode::OK, Html(html)).into_response()
        },
        Err(err) => {
            eprintln!("get_public_product_html failed for id {}: {}", id, err);
            (StatusCode::NOT_FOUND, "Product not found").into_response()
        }
    }
}

// GET /qr/{id} - Return QR as PNG image bytes
pub async fn get_qr_png(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.couch.get_doc::<Herb>(&state.db_name, &id).await {
        Ok(herb) => {
            let bytes = generate_qr_png_bytes(&herb);
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/png")],
                bytes,
            )
        }.into_response(),
        Err(err) => {
            eprintln!("get_qr_png failed for id {}: {}", id, err);
            (StatusCode::NOT_FOUND, "Herb not found").into_response()
        },
    }
}

// GET /listHerbs
pub async fn list_herbs(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.couch.list_docs::<Herb>(&state.db_name).await {
        Ok(herbs) => (StatusCode::OK, Json(herbs)).into_response(),
        Err(err) => {
            eprintln!("list_herbs failed: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch herbs").into_response()
        },
    }
}

// DELETE /deleteHerb/{id}
pub async fn delete_herb(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.couch.delete_doc(&state.db_name, &id).await {
        Ok(_) => (StatusCode::OK, format!("Herb {} deleted successfully", id)).into_response(),
        Err(err) => {
            eprintln!("delete_herb failed for id {}: {}", id, err);
            (StatusCode::NOT_FOUND, "Herb not found or deletion failed").into_response()
        },
    }
}

// PUT /updateHerb/{id}
pub async fn update_herb(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateHerbRequest>,
) -> impl IntoResponse {
    // Fetch current doc with revision
    let (mut herb, rev) = match state.couch.get_doc_with_rev::<Herb>(&state.db_name, &id).await {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!("update_herb get failed for id {}: {}", id, err);
            return (StatusCode::NOT_FOUND, "Herb not found").into_response();
        }
    };

    // Apply partial updates with basic validation
    if let Some(name) = payload.name {
        if name.trim().is_empty() || name.len() > 100 {
            return (StatusCode::BAD_REQUEST, "invalid name").into_response();
        }
        herb.name = name;
    }
    if let Some(farmer) = payload.farmer {
        if farmer.trim().is_empty() || farmer.len() > 100 {
            return (StatusCode::BAD_REQUEST, "invalid farmer").into_response();
        }
        herb.farmer = farmer;
    }
    if let Some(location) = payload.location {
        if location.trim().is_empty() || location.len() > 200 {
            return (StatusCode::BAD_REQUEST, "invalid location").into_response();
        }
        herb.location = location;
    }

    // Persist update with _rev
    if let Err(err) = state.couch.update_doc(&state.db_name, &id, &rev, &herb).await {
        eprintln!("update_herb save failed for id {}: {}", id, err);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update herb").into_response();
    }

    (StatusCode::OK, Json(herb)).into_response()
}

fn extract_id_from_scanned_text(input: &str) -> Option<String> {
    // Try as URL like https://.../p/{id}
    if let Ok(url) = Url::parse(input) {
        let path = url.path();
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.len() >= 2 && segments[0] == "p" {
            return Some(segments[1].to_string());
        }
        // Fallback: last segment
        if let Some(last) = segments.last() {
            return Some((*last).to_string());
        }
    }

    // Try as JSON containing { "id": "..." } or full Herb
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(input) {
        if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
            return Some(id.to_string());
        }
    }

    // Assume plain id
    let trimmed = input.trim();
    if !trimmed.is_empty() {
        return Some(trimmed.to_string());
    }
    None
}

// POST /scan - Accepts scanned QR text and resolves to product info
pub async fn scan_product(
    State(state): State<AppState>,
    Json(payload): Json<ScanRequest>,
) -> impl IntoResponse {
    let Some(id) = extract_id_from_scanned_text(&payload.data) else {
        return (StatusCode::BAD_REQUEST, "Unable to extract product id").into_response();
    };

    match state.couch.get_doc::<Herb>(&state.db_name, &id).await {
        Ok(herb) => (StatusCode::OK, Json(herb)).into_response(),
        Err(err) => {
            eprintln!("scan_product could not fetch id {}: {}", id, err);
            (StatusCode::NOT_FOUND, "Product not found").into_response()
        },
    }
}

// GET /scan-page - Minimal HTML scanner page (paste/scan input)
pub async fn scan_page() -> impl IntoResponse {
    let html = r#"<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Scan Product</title><style>body{font-family:sans-serif;margin:24px;} .card{max-width:640px;border:1px solid #eee;border-radius:12px;padding:20px;box-shadow:0 2px 8px rgba(0,0,0,0.06);} textarea{width:100%;height:120px} pre{background:#f7f7f7;padding:12px;border-radius:8px;white-space:pre-wrap;word-break:break-all}</style></head><body><div class=\"card\"><h1>Scan Product</h1><p>Paste scanned QR text (URL/JSON/id) below. The page will POST to /scan and show the product.</p><textarea id=\"scan\" placeholder=\"Paste scanned content here...\"></textarea><br/><button id=\"btn\">Submit</button><pre id=\"out\"></pre></div><script>const btn=document.getElementById('btn');const area=document.getElementById('scan');const out=document.getElementById('out');btn.onclick=async()=>{out.textContent='Loading...';try{const res=await fetch('/scan',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({data:area.value})});const text=await res.text();try{const json=JSON.parse(text);out.textContent=JSON.stringify(json,null,2);}catch(e){out.textContent=text}}catch(e){out.textContent=String(e)}};</script></body></html>"#;
    (StatusCode::OK, Html(html)).into_response()
}