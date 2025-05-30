use std::{sync::Arc, thread};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    buttons::{DeckButton, DeckButtonUpdate},
    config::DeckConfig,
    deck::{Deck, DeckScreen},
};

#[derive(Clone)]
struct AxumState {
    deck: Arc<Deck>,
}

async fn get_config(State(state): State<AxumState>) -> Json<DeckConfig> {
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

async fn build_and_run<S>(deck_ref: Arc<Deck>, host: S, port: u32)
where
    S: AsRef<str>,
{
    let state = AxumState { deck: deck_ref };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST]);

    let app = Router::new()
        .route("/api/client/config", get(get_config))
        .route("/api/client/buttons", get(get_buttons))
        .route("/api/client/click/{y}/{x}", post(handle_click))
        .route("/api/client/icon/{id}", get(get_icon))
        .route(
            "/api/config/button/{y}/{x}",
            get(get_button).put(update_button),
        )
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
