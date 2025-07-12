use actix_web::{web, HttpResponse, Result};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct NixConfig {
    pub content: String,
    pub device_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub hash: String,
    pub content: String,
    pub device_type: String,
    pub description: Option<String>,
}

pub async fn get_config(
    session: Session,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    if let Ok(Some(user_id)) = session.get::<u64>("user_id") {
        match data.db.get_config(user_id) {
            Ok(Some(config)) => Ok(HttpResponse::Ok().json(config)),
            Ok(None) => Ok(HttpResponse::NotFound().json("No configuration found")),
            Err(_) => Ok(HttpResponse::InternalServerError().json("Database error")),
        }
    } else {
        Ok(HttpResponse::Unauthorized().json("Not logged in"))
    }
}

pub async fn update_config(
    session: Session,
    data: web::Data<AppState>,
    config: web::Json<NixConfig>,
) -> Result<HttpResponse> {
    if let Ok(Some(user_id)) = session.get::<u64>("user_id") {
        // Generate hash for the config
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", user_id, config.content));
        let hash = hex::encode(hasher.finalize());
        
        let config_response = ConfigResponse {
            hash: hash.clone(),
            content: config.content.clone(),
            device_type: config.device_type.clone(),
            description: config.description.clone(),
        };
        
        match data.db.store_config(user_id, &hash, &config_response) {
            Ok(_) => Ok(HttpResponse::Ok().json(config_response)),
            Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to store config")),
        }
    } else {
        Ok(HttpResponse::Unauthorized().json("Not logged in"))
    }
}

pub async fn serve_config(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let hash = path.into_inner();
    
    match data.db.get_config_by_hash(&hash) {
        Ok(Some(config)) => Ok(HttpResponse::Ok()
            .content_type("text/plain")
            .body(config.content)),
        Ok(None) => Ok(HttpResponse::NotFound().body("Configuration not found")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Database error")),
    }
}

