use axum::Json;
use axum::extract::{Path, State};
use duckdb::{Connection, Row};
use serde::Serialize;

use super::{ApiError, Info};
use crate::AppState;

#[derive(Serialize)]
pub struct PageInfo {
    /// Buttons on other pages that link to this page.
    backlinks: Vec<PageButton>,
    /// Buttons on this page, with the page each one links to.
    buttons: Vec<PageButton>,
}

#[derive(Serialize)]
struct PageButton {
    /// The page on the other end: the linking page for a backlink, the linked
    /// page for a button (if the button links anywhere).
    page: Option<String>,
    button: String,
    alt: Option<String>,
    title: Option<String>,
    filename: Option<String>,
}

impl PageButton {
    fn from_row(r: &Row) -> duckdb::Result<Self> {
        Ok(Self {
            page: r.get(0)?,
            button: r.get(1)?,
            alt: r.get(2)?,
            title: r.get(3)?,
            filename: r.get(4)?,
        })
    }
}

fn query(db: &Connection, sql: &str, page: &str) -> duckdb::Result<Vec<PageButton>> {
    db.prepare(sql)?.query_map([page], PageButton::from_row)?.collect()
}

pub async fn page_api_route(
    State(state): State<AppState>,
    Path(page): Path<String>,
) -> Result<Json<Info<PageInfo>>, ApiError> {
    let info = state
        .db
        .run(move |db| {
            Ok(PageInfo {
                backlinks: query(
                    db,
                    "SELECT page, sha256, alt, title, filename FROM pages WHERE link = ?",
                    &page,
                )?,
                buttons: query(
                    db,
                    "SELECT link, sha256, alt, title, filename FROM pages WHERE page = ?",
                    &page,
                )?,
            })
        })
        .await?;

    Ok(Json(Info { info }))
}
