mod bmp_analyzer;
mod server;
mod templates;

use bmp_analyzer::{guess_image, sugar};
use server::ServerState;

use askama::Template;
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{Html, Response},
    routing::{get, post},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

type SharedServerState = Arc<RwLock<ServerState>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_server_state = Arc::new(RwLock::new(ServerState::new()));

    let session_app = axum::Router::new()
        .route("/image_analyze", post(image_analyze))
        .route("/sugary_image/{number_of_cuts}", get(sugary_image))
        .with_state(shared_server_state);

    let app = axum::Router::new()
        .route("/", get(get_index))
        .nest_service("/static", ServeDir::new("static"))
        .nest("/session/{uuid}", session_app);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
async fn get_index() -> Result<Html<String>, StatusCode> {
    let uuid = &uuid::Uuid::new_v4().to_string();

    let index = templates::IndexHtml { uuid };
    let html = index
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

async fn image_analyze(
    Path(uuid): Path<String>,
    State(server_state): State<SharedServerState>,
    mut multipart: Multipart,
) -> Result<Html<String>, StatusCode> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let data = field
        .bytes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dynamic_image =
        image::load_from_memory(&data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let guess_top = guess_image(&dynamic_image, &server_state.read().await.references)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    server_state
        .write()
        .await
        .user_image
        .insert(uuid.clone(), dynamic_image);

    let mut rows = Vec::new();
    let mut unique_numbers = Vec::new();
    guess_top.iter().for_each(|item| {
        let row = templates::Row {
            guess: item.index_of_reference,
            cells: item.number_of_cells_in_row,
            diff: item.difference,
        };
        rows.push(row);
        match unique_numbers.contains(&item.number_of_cells_in_row) {
            true => (),
            false => unique_numbers.push(item.number_of_cells_in_row),
        }
    });
    let uuid = &uuid;

    let table = templates::Table {
        uuid,
        rows,
        unique_numbers,
    };
    let html = table
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

async fn sugary_image(
    Path((uuid, number_of_cuts)): Path<(String, u32)>,
    State(server_state): State<SharedServerState>,
) -> Result<Response, StatusCode> {
    let server_state_reader = server_state.read().await;
    let raw_image: &image::DynamicImage = server_state_reader
        .user_image
        .get(&uuid)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let processed_image = sugar(raw_image, number_of_cuts)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();
    let mut bytes = Vec::new();
    processed_image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "image/png")
        .header("cache-control", "no-store, no-cache, must-revalidate")
        .body(Body::from(bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}
