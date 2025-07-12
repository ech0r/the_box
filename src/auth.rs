use actix_web::{web, HttpResponse, Result, HttpRequest};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use reqwest;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

pub async fn github_auth(data: web::Data<AppState>) -> Result<HttpResponse> {
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&scope=user:email",
        data.github_client_id
    );
    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url))
        .finish())
}

pub async fn github_callback(
    req: HttpRequest,
    data: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    let query = req.query_string();
    let params: std::collections::HashMap<String, String> = 
        serde_urlencoded::from_str(query).unwrap_or_default();
    
    if let Some(code) = params.get("code") {
        let token_response = exchange_code_for_token(code, &data).await?;
        let user = get_github_user(&token_response.access_token).await?;
        
        session.insert("user_id", user.id)?;
        session.insert("username", &user.login)?;
        
        // Store user in database
        data.db.store_user(&user)?;
        
        Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish())
    } else {
        Ok(HttpResponse::BadRequest().json("Missing code parameter"))
    }
}

async fn exchange_code_for_token(
    code: &str,
    data: &AppState,
) -> Result<GitHubTokenResponse> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", data.github_client_id.as_str()),
        ("client_secret", data.github_client_secret.as_str()),
        ("code", code),
    ];
    
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to exchange token"))?;
    
    let token_response: GitHubTokenResponse = response
        .json()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to parse token response"))?;
    
    Ok(token_response)
}

async fn get_github_user(access_token: &str) -> Result<GitHubUser> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("token {}", access_token))
        .header("User-Agent", "the-box")
        .send()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to get user info"))?;
    
    let user: GitHubUser = response
        .json()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to parse user response"))?;
    
    Ok(user)
}

pub async fn logout(session: Session) -> Result<HttpResponse> {
    session.clear();
    Ok(HttpResponse::Ok().json("Logged out"))
}

pub async fn get_user(session: Session) -> Result<HttpResponse> {
    if let Ok(Some(user_id)) = session.get::<u64>("user_id") {
        if let Ok(Some(username)) = session.get::<String>("username") {
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "id": user_id,
                "username": username
            })));
        }
    }
    Ok(HttpResponse::Unauthorized().json("Not logged in"))
}

