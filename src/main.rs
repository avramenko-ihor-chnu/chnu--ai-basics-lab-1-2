use std::collections::HashMap;

use askama::Template;
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use axum::routing::post;
use bmp_analyzer::maybe_load_reference;
use tokio::fs;
mod bmp_analyzer;
use axum::extract::{Multipart, State};
use image::DynamicImage;
use tower_http::services::ServeDir;

use crate::bmp_analyzer::guess_image;

#[derive(askama::Template)]
#[template(path = "image-analysis-table.html")]
struct Table {
    rows: Vec<Row>,
}

struct Row {
    guess: usize,
    cells: u32,
    diff: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let references: HashMap<i32, DynamicImage> = maybe_load_reference().await?;

    let app = axum::Router::new()
        .route("/", get(get_index))
        .route("/image-analyze", post(image_analyze))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/sugar", ServeDir::new("sugar"))
        .with_state(references);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_index() -> Result<Html<String>, StatusCode> {
    let index = fs::read_to_string("templates/index.html")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(index))
}

async fn image_analyze(
    State(references): State<HashMap<i32, DynamicImage>>,
    mut multipart: Multipart,
) -> Result<Html<String>, StatusCode> {
    let mut rows = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let dynamic_image =
            image::load_from_memory(&data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let guess_top = guess_image(&dynamic_image, &references)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        guess_top.iter().for_each(|item| {
            let row = Row {
                guess: item.index_of_reference,
                cells: item.number_of_cells_in_row,
                diff: item.difference,
            };
            rows.push(row);
        });
    }

    let table = Table { rows };
    let html = table
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}
