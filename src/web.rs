use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};

use crate::ssh;

const HTML: &str = include_str!("index.html");

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

fn api_error(msg: impl Into<String>) -> (StatusCode, Json<ApiResponse<()>>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse {
            success: false,
            data: None,
            error: Some(msg.into()),
        }),
    )
}

fn api_bad_request(msg: impl Into<String>) -> (StatusCode, Json<ApiResponse<()>>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse {
            success: false,
            data: None,
            error: Some(msg.into()),
        }),
    )
}

/// GET / – serve the HTML UI
async fn index() -> Html<&'static str> {
    Html(HTML)
}

/// GET /api/repos – list all bare repositories
async fn list_repos() -> impl IntoResponse {
    match ssh::list_repos() {
        Ok(repos) => (StatusCode::OK, Json(ApiResponse::ok(repos))).into_response(),
        Err(e) => api_error(e).into_response(),
    }
}

#[derive(Deserialize)]
struct CreateRepoRequest {
    name: String,
}

/// POST /api/repos – create a new bare repository
async fn create_repo(Json(body): Json<CreateRepoRequest>) -> impl IntoResponse {
    if let Err(e) = ssh::validate_repo_name(&body.name) {
        return api_bad_request(e).into_response();
    }
    match ssh::create_repo(&body.name) {
        Ok(()) => (StatusCode::CREATED, Json(ApiResponse::ok(body.name))).into_response(),
        Err(e) => api_error(e).into_response(),
    }
}

/// DELETE /api/repos/:name – delete a bare repository
async fn delete_repo(Path(name): Path<String>) -> impl IntoResponse {
    if let Err(e) = ssh::validate_repo_name(&name) {
        return api_bad_request(e).into_response();
    }
    match ssh::delete_repo(&name) {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::ok(name))).into_response(),
        Err(e) => api_error(e).into_response(),
    }
}

#[derive(Deserialize)]
struct RenameRepoRequest {
    new_name: String,
}

/// PUT /api/repos/:name – rename a bare repository
async fn rename_repo(
    Path(name): Path<String>,
    Json(body): Json<RenameRepoRequest>,
) -> impl IntoResponse {
    if let Err(e) = ssh::validate_repo_name(&name) {
        return api_bad_request(e).into_response();
    }
    if let Err(e) = ssh::validate_repo_name(&body.new_name) {
        return api_bad_request(e).into_response();
    }
    match ssh::rename_repo(&name, &body.new_name) {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::ok(body.new_name))).into_response(),
        Err(e) => api_error(e).into_response(),
    }
}

/// Builds the router.
pub fn build_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/repos", get(list_repos))
        .route("/api/repos", post(create_repo))
        .route("/api/repos/{name}", delete(delete_repo))
        .route("/api/repos/{name}", put(rename_repo))
}

/// Starts the web server on port 7411.
pub async fn start() {
    let app = build_router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:7411")
        .await
        .expect("Failed to bind port 7411");

    println!("git-retail web UI running at http://localhost:7411");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
