use std::{sync::Arc, thread};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{Method, StatusCode, header},
    response::IntoResponse,
    routing::{get, patch, post, put},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    buttons::{RawDeckButton, DeckButtonPos, DeckButtonUpdate},
    config::DeckDimensionConfig,
    deck::{Deck, DeckScreen},
    models::{
        PluginActionsGroupedData, PluginActionsUngroupedData, PluginData,
        PluginVariablesGroupedData, PluginVariablesUngroupedData,
    },
};

#[derive(serde::Deserialize)]
struct PatchButtonsSwapRequest {
    a: DeckButtonPos,
    b: DeckButtonPos,
}

#[derive(Clone)]
struct AxumState {
    deck: Arc<Deck>,
}

async fn get_config(State(state): State<AxumState>) -> Json<DeckDimensionConfig> {
    Json(state.deck.get_dimensions_config())
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

async fn handle_switch_screen(State(state): State<AxumState>, Path(id): Path<String>) {
    state.deck.switch_screen(id);
}

async fn handle_new_screen(State(state): State<AxumState>, Path(id): Path<String>) -> StatusCode {
    match state.deck.new_screen(id) {
        Ok(()) => StatusCode::OK,
        Err(()) => StatusCode::CONFLICT,
    }
}

async fn delete_screen(State(state): State<AxumState>, Path(id): Path<String>) -> StatusCode {
    match state.deck.delete_screen(&id) {
        Ok(()) => StatusCode::OK,
        Err(()) => StatusCode::NOT_FOUND,
    }
}

async fn rename_screen(
    State(state): State<AxumState>,
    Path(id): Path<String>,
    Json(new_name): Json<String>,
) -> StatusCode {
    match state.deck.rename_screen(&id, new_name) {
        Ok(()) => StatusCode::OK,
        Err(()) => StatusCode::NOT_FOUND,
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
) -> Json<RawDeckButton> {
    Json(state.deck.get_raw_button(pos))
}

async fn update_config(State(state): State<AxumState>, Json(update): Json<DeckDimensionConfig>) {
    state.deck.update_config(update);
}

async fn update_button(
    State(state): State<AxumState>,
    Path(pos): Path<(u32, u32)>,
    Json(update): Json<DeckButtonUpdate>,
) -> StatusCode {
    state.deck.update_button(pos, update);

    StatusCode::OK
}

async fn swap_buttons(
    State(state): State<AxumState>,
    Json(buttons): Json<PatchButtonsSwapRequest>,
) -> StatusCode {
    state
        .deck
        .swap_buttons(buttons.a.as_yx(), buttons.b.as_yx());

    StatusCode::OK
}

async fn delete_button(State(state): State<AxumState>, Path(pos): Path<(u32, u32)>) -> StatusCode {
    if state.deck.delete_button(pos) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn list_variables_ungrouped(
    State(state): State<AxumState>,
) -> Json<Vec<PluginVariablesUngroupedData>> {
    Json(state.deck.get_all_variables_ungrouped())
}

async fn list_variables_grouped(
    State(state): State<AxumState>,
) -> Json<Vec<PluginVariablesGroupedData>> {
    Json(state.deck.get_all_variables_grouped())
}

async fn list_actions_ungrouped(
    State(state): State<AxumState>,
) -> Json<Vec<PluginActionsUngroupedData>> {
    Json(state.deck.get_all_actions_ungrouped())
}

async fn list_actions_grouped(
    State(state): State<AxumState>,
) -> Json<Vec<PluginActionsGroupedData>> {
    Json(state.deck.get_all_actions_grouped())
}

async fn list_plugins(State(state): State<AxumState>) -> Json<Vec<PluginData>> {
    Json(state.deck.get_all_plugins())
}

async fn list_screens(State(state): State<AxumState>) -> Json<Vec<String>> {
    Json(state.deck.get_available_screens())
}

async fn list_icons(State(state): State<AxumState>) -> Json<Vec<String>> {
    Json(state.deck.get_all_icons())
}

async fn build_and_run<S>(deck_ref: Arc<Deck>, host: S, port: u32)
where
    S: AsRef<str>,
{
    let state = AxumState { deck: deck_ref };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::OPTIONS,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/client/config", get(get_config))
        .route("/api/client/buttons", get(get_buttons))
        .route("/api/client/click/{y}/{x}", post(handle_click))
        .route("/api/client/icon/{id}", get(get_icon))
        .route("/api/config/config", patch(update_config))
        .route(
            "/api/config/button/{y}/{x}",
            get(get_button).patch(update_button).delete(delete_button),
        )
        .route("/api/config/buttons/swap", patch(swap_buttons))
        .route(
            "/api/config/list/actions/ungrouped",
            get(list_actions_ungrouped),
        )
        .route(
            "/api/config/list/actions/grouped",
            get(list_actions_grouped),
        )
        .route(
            "/api/config/list/variables/ungrouped",
            get(list_variables_ungrouped),
        )
        .route(
            "/api/config/list/variables/grouped",
            get(list_variables_grouped),
        )
        .route("/api/config/list/plugins", get(list_plugins))
        .route("/api/config/list/screens", get(list_screens))
        .route("/api/config/list/icons", get(list_icons))
        .route(
            "/api/config/screen/{id}",
            put(handle_new_screen)
                .post(handle_switch_screen)
                .patch(rename_screen)
                .delete(delete_screen),
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
