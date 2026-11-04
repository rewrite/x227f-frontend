use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;

use super::{ApiError, Info};
use crate::AppState;

#[derive(Serialize)]
pub struct ButtonInfo {
    pages: Vec<PageInfo>,
}

/// One place the button was found on.
#[derive(Serialize)]
struct PageInfo {
    backlink: String,
    link: Option<String>,
    alt: Option<String>,
    title: Option<String>,
    filename: Option<String>,
}

pub async fn button_api_route(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<Info<ButtonInfo>>, ApiError> {
    if hash.len() > 32 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::BadRequest("Invalid hash"));
    }

    let pages = state
        .db
        .run(move |db| {
            db.prepare("SELECT page, link, alt, title, filename FROM pages WHERE sha256 = ?")?
                .query_map([hash], |r| {
                    Ok(PageInfo {
                        backlink: r.get(0)?,
                        link: r.get(1)?,
                        alt: r.get(2)?,
                        title: r.get(3)?,
                        filename: r.get(4)?,
                    })
                })?
                .collect()
        })
        .await?;

    Ok(Json(Info {
        info: ButtonInfo { pages },
    }))
}
