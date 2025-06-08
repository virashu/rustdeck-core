use std::{collections::HashMap, sync::Arc, thread};

use axum::{
    extract::{Path, State},
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    buttons::{DeckButton, DeckButtonUpdate},
    config::DeckDimensionConfig,
    deck::{Deck, DeckScreen},
};

#[derive(Clone)]
struct AxumState {
    deck: Arc<Deck>,
}

async fn get_config(State(state): State<AxumState>) -> Json<DeckDimensionConfig> {
    Json(state.deck.get_config())
}

async fn get_buttons(State(state): State<AxumState>) -> Json<DeckScreen> {
    Json(state.deck.get_rendered_screen())
}

async fn handle_click(State(state): State<AxumState>, Path(pos): Path<(u32, u32)>) -> StatusCode {
    match state.deck.handle_click_at(pos) {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

async fn get_icon(
    State(state): State<AxumState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let icon_path = state.deck.get_icon(id);

    icon_path.map_or_else(
        || Err(StatusCode::NOT_FOUND),
        |path| {
            Ok((
                axum::response::AppendHeaders([(header::CONTENT_TYPE, "image/png")]),
                std::fs::read(path).unwrap(),
            ))
        },
    )
}

async fn get_button(
    State(state): State<AxumState>,
    Path(pos): Path<(u32, u32)>,
) -> Json<DeckButton> {
    Json(state.deck.get_raw_button(pos))
}

async fn update_button(
    State(state): State<AxumState>,
    Path(pos): Path<(u32, u32)>,
    Json(update): Json<DeckButtonUpdate>,
) -> StatusCode {
    state.deck.update_button(pos, update);

    StatusCode::OK
}

async fn list_variables(State(state): State<AxumState>) -> Json<HashMap<String, String>> {
    Json(state.deck.get_all_variables())
}

async fn list_actions(State(state): State<AxumState>) -> Json<Vec<String>> {
    Json(state.deck.get_all_actions_names())
}

async fn list_screens(State(state): State<AxumState>) -> Json<Vec<String>> {
    Json(state.deck.get_available_screens())
}

async fn build_and_run<S>(deck_ref: Arc<Deck>, host: S, port: u32)
where
    S: AsRef<str>,
{
    let state = AxumState { deck: deck_ref };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/client/config", get(get_config))
        .route("/api/client/buttons", get(get_buttons))
        .route("/api/client/click/{y}/{x}", post(handle_click))
        .route("/api/client/icon/{id}", get(get_icon))
        .route(
            "/api/config/button/{y}/{x}",
            get(get_button).patch(update_button),
        )
        .route("/api/config/list/actions", get(list_actions))
        .route("/api/config/list/variables", get(list_variables))
        .route("/api/config/list/screens", get(list_screens))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(format!("{}:{port}", host.as_ref()))
        .await
        .unwrap();

    tracing::info!("Axum serving");
    axum::serve(listener, app).await.unwrap();
}

pub fn build_and_run_thread<S>(
    deck_ref: &Arc<Deck>,
    host: S,
    port: u32,
) -> std::thread::JoinHandle<()>
where
    S: AsRef<str>,
{
    let host = host.as_ref().to_owned();
    let deck = deck_ref.clone();

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();

        rt.block_on(build_and_run(deck, host, port));
    })
}
