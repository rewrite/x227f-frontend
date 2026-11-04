use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};

use super::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct QueryParams {
    amount: usize,
}

#[derive(Serialize)]
pub struct Hashes {
    hashes: Vec<String>,
}

pub async fn random_api_route(
    State(state): State<AppState>,
    Query(QueryParams { amount }): Query<QueryParams>,
) -> Result<Json<Hashes>, ApiError> {
    // safe to format into the query since it's just a number
    let amount = amount.clamp(1, 1000);
    let hashes = state
        .db
        .run(move |db| {
            db.prepare(&format!(
                "SELECT sha256 FROM (SELECT DISTINCT sha256 FROM pages) USING SAMPLE {amount} ROWS"
            ))?
            .query_map([], |r| r.get(0))?
            .collect()
        })
        .await?;

    Ok(Json(Hashes { hashes }))
}
