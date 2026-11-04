mod button;
mod page;
mod random;
mod search;
mod stats;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{MethodRouter, get};
use axum::{Json, Router};
use serde::Serialize;

use crate::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", html(include_str!("../../static/index.html")))
        .route("/button", html(include_str!("../../static/button.html")))
        .route("/random", html(include_str!("../../static/random.html")))
        .route("/page", html(include_str!("../../static/page.html")))
        .route("/search", html(include_str!("../../static/search.html")))
        .route(
            "/style.css",
            get(|| async { ([(CONTENT_TYPE, "text/css")], include_str!("../../static/style.css")) }),
        )
        .route("/api/page/{page}", get(page::page_api_route))
        .route("/api/button/{hash}", get(button::button_api_route))
        .route("/api/button/{hash}/img", get(button_img_route))
        .route("/api/search", get(search::search_api_route))
        .route("/api/random", get(random::random_api_route))
        .route("/api/stats", get(stats::stats_api_route))
        .with_state(state)
}

fn html(page: &'static str) -> MethodRouter<AppState> {
    get(move || async move { Html(page) })
}

async fn button_img_route(State(state): State<AppState>, Path(hash): Path<String>) -> Response {
    if let Some(button) = state.buttons.get(&hash) {
        // buttons are named by their hash, so they never change
        let headers = [
            (CONTENT_TYPE, button.content_type),
            (CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ];
        return (headers, button.data.clone()).into_response();
    }
    match state.buttons.get("404") {
        Some(button) => (
            StatusCode::NOT_FOUND,
            [(CONTENT_TYPE, button.content_type)],
            button.data.clone(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// A successful response for routes that wrap their result in `{"info": ...}`.
#[derive(Serialize)]
pub struct Info<T> {
    info: T,
}

/// Responds with `{"error": "..."}`.
#[derive(Debug)]
pub enum ApiError {
    BadRequest(&'static str),
    Internal(String),
}

impl From<duckdb::Error> for ApiError {
    fn from(e: duckdb::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct Body {
            error: String,
        }
        let (status, error) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.to_owned()),
            Self::Internal(msg) => {
                eprintln!("internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_owned())
            }
        };
        (status, Json(Body { error })).into_response()
    }
}
