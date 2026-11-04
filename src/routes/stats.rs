use axum::Json;
use axum::extract::State;
use serde::Serialize;

use super::ApiError;
use crate::AppState;

#[derive(Serialize)]
pub struct Stats {
    buttons_link: u64,
    buttons_no_link: u64,
    unique_buttons: u64,
}

pub async fn stats_api_route(State(state): State<AppState>) -> Result<Json<Stats>, ApiError> {
    let stats = state
        .db
        .run(|db| {
            db.query_row(
                "SELECT count(link), count(*) - count(link), count(DISTINCT sha256) FROM pages",
                [],
                |r| {
                    Ok(Stats {
                        buttons_link: r.get(0)?,
                        buttons_no_link: r.get(1)?,
                        unique_buttons: r.get(2)?,
                    })
                },
            )
        })
        .await?;

    Ok(Json(stats))
}
