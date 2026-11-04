use axum::Json;
use axum::extract::{Query, State};
use duckdb::params_from_iter;
use serde::{Deserialize, Serialize};

use super::ApiError;
use crate::AppState;

const MAX_PERCENT_WILDCARDS: usize = 3;
const MAX_UNDERSCORE_WILDCARDS: usize = 10;
const MAX_RESULTS: usize = 1000;

/// What each bit of `search_in` searches, from the highest bit to the lowest:
/// filenames, names (alt and title), links to, backlinks.
const SEARCH_COLUMNS: [(u8, &[&str]); 4] = [
    (1 << 3, &["filename"]),
    (1 << 2, &["alt", "title"]),
    (1 << 1, &["link"]),
    (1 << 0, &["page"]),
];

#[derive(Deserialize)]
pub struct QueryParams {
    /// A LIKE pattern.
    query: String,
    search_in: u8,
}

#[derive(Serialize)]
pub struct SearchResults {
    results: Vec<String>,
}

pub async fn search_api_route(
    State(state): State<AppState>,
    Query(QueryParams { query, search_in }): Query<QueryParams>,
) -> Result<Json<SearchResults>, ApiError> {
    if query.matches('%').count() > MAX_PERCENT_WILDCARDS
        || query.matches('_').count() > MAX_UNDERSCORE_WILDCARDS
    {
        return Err(ApiError::BadRequest(
            "Too many wildcards. Max. 3 of % and 10 of _",
        ));
    }

    let columns: Vec<&str> = SEARCH_COLUMNS
        .iter()
        .filter(|(bit, _)| search_in & bit != 0)
        .flat_map(|(_, columns)| columns.iter().copied())
        .collect();
    if columns.is_empty() || search_in > 0b1111 {
        return Err(ApiError::BadRequest(
            "Invalid bitflags. Did you check at least one search?",
        ));
    }

    let conditions = columns
        .iter()
        .map(|column| format!("{column} LIKE ?"))
        .collect::<Vec<_>>()
        .join(" OR ");
    let sql = format!("SELECT DISTINCT sha256 FROM pages WHERE {conditions} LIMIT {MAX_RESULTS}");

    let results = state
        .db
        .run(move |db| {
            db.prepare(&sql)?
                .query_map(params_from_iter(vec![query; columns.len()]), |r| r.get(0))?
                .collect()
        })
        .await?;

    Ok(Json(SearchResults { results }))
}
