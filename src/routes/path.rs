use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};

use super::{ApiError, Info};
use crate::AppState;

#[derive(Deserialize)]
pub struct PathQuery {
    from: String,
    to: String,
}

#[derive(Serialize)]
pub struct PathInfo {
    /// The pages from `from` to `to` (inclusive), or None if there's no path.
    path: Option<Vec<PathStep>>,
}

#[derive(Serialize)]
struct PathStep {
    page: String,
    /// The button on this page that links to the next one.
    button: Option<String>,
}

pub async fn path_api_route(
    State(state): State<AppState>,
    Query(q): Query<PathQuery>,
) -> Result<Json<Info<PathInfo>>, ApiError> {
    let graph = &state.graph;
    let from = graph.find_page(&q.from).ok_or(ApiError::BadRequest("unknown start page"))?;
    let to = graph.find_page(&q.to).ok_or(ApiError::BadRequest("unknown target page"))?;
    let path = graph.shortest_path(from, to).map(|steps| {
        steps
            .into_iter()
            .map(|s| PathStep {
                page: s.page.to_owned(),
                button: s.button.map(str::to_owned),
            })
            .collect()
    });
    Ok(Json(Info { info: PathInfo { path } }))
}
