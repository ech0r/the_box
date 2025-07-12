use actix_web::{web, App, HttpServer, middleware::Logger, Result, HttpResponse, HttpRequest};
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use sha2::{Sha256, Digest};

mod auth;
mod config;
mod database;

use auth::*;
use config::*;
use database::*;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub github_client_id: String,
    pub github_client_secret: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    
    let github_client_id = std::env::var("GITHUB_CLIENT_ID")
        .expect("GITHUB_CLIENT_ID must be set");
    let github_client_secret = std::env::var("GITHUB_CLIENT_SECRET")
        .expect("GITHUB_CLIENT_SECRET must be set");
    
    let db = Database::new("./data").expect("Failed to open database");
    let state = AppState {
        db,
        github_client_id,
        github_client_secret,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(Logger::default())
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(&[0; 64])
                )
                .build()
            )
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
            )
            .service(
                web::scope("/api")
                    .route("/auth/github", web::get().to(github_auth))
                    .route("/auth/callback", web::get().to(github_callback))
                    .route("/auth/logout", web::post().to(logout))
                    .route("/auth/user", web::get().to(get_user))
                    .route("/config", web::get().to(get_config))
                    .route("/config", web::post().to(update_config))
            )
            .route("/config/{hash}", web::get().to(serve_config))
            .route("/", web::get().to(serve_frontend))
            .route("/{path:.*}", web::get().to(serve_frontend))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn serve_frontend() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../frontend/index.html")))
}
