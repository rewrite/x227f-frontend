mod buttons;
mod db;
mod graph;
mod routes;

use std::sync::Arc;

use crate::buttons::Buttons;
use crate::db::Db;
use crate::graph::Graph;

const BUTTONS_DIR: &str = "buttons";
const DATABASE_PATH: &str = "88x31.ddb";
const LISTEN_ADDR: &str = "0.0.0.0:8831";

#[derive(Clone)]
pub struct AppState {
    pub buttons: Arc<Buttons>,
    pub db: Db,
    pub graph: Arc<Graph>,
}

#[tokio::main]
async fn main() {
    let db = Db::open(DATABASE_PATH)
        .unwrap_or_else(|e| panic!("couldn't open {DATABASE_PATH}: {e}"));
    let buttons = tokio::task::spawn_blocking(|| Buttons::load(BUTTONS_DIR))
        .await
        .unwrap()
        .unwrap_or_else(|e| panic!("couldn't load buttons from {BUTTONS_DIR}/: {e}"));
    println!("loaded {} buttons", buttons.len());
    let graph = db
        .run(|db| Graph::load(db))
        .await
        .unwrap_or_else(|e| panic!("couldn't load the link graph: {e:?}"));
    println!("loaded link graph: {} pages, {} links", graph.page_count(), graph.edge_count());

    let app = routes::router(AppState {
        buttons: Arc::new(buttons),
        db,
        graph: Arc::new(graph),
    });

    let listener = tokio::net::TcpListener::bind(LISTEN_ADDR).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
