mod chart;
mod error;

use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, fs};

use axum::http::StatusCode;
use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chart::LineColor;
use serde::Deserialize;

use crate::chart::{Chart, ChartType, Charts};

#[tokio::main]
async fn main() {
    let charts = Arc::new(Charts::new());

    let app = Router::new()
        .route("/:index", get(index))
        .route("/new_chart", get(new_plot))
        .route("/new_label/:index", get(new_label))
        .route("/plot/:index", get(get_chart_data))
        .route("/plot_info/:index", get(get_chart_info))
        .route("/insert/:index", get(insert_data))
        .route("/www/pkg/:file_name", get(serve_file))
        .layer(Extension(charts));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index(
    Extension(charts): Extension<Arc<Charts>>,
    Path(index): Path<usize>,
) -> Result<Response<Body>, StatusCode> {
    if charts.contains(index) {
        let bytes = include_bytes!("../www/index.html").to_vec();
        let len = bytes.len();

        let body = Body::from(bytes);
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("CONTENT-LENGTH", len)
            .header("CONTENT-TYPE", "text/html")
            .body(body)
            .unwrap())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn new_plot(
    Extension(charts): Extension<Arc<Charts>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response<String>, StatusCode> {
    let caption = params.get("caption").ok_or(StatusCode::BAD_REQUEST)?;
    let chart_type = params.get("type").ok_or(StatusCode::BAD_REQUEST)?;

    let y_start = params
        .get("y_start")
        .ok_or(StatusCode::BAD_REQUEST)?
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let y_end = params
        .get("y_end")
        .ok_or(StatusCode::BAD_REQUEST)?
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let interval = params
        .get("interval")
        .ok_or(StatusCode::BAD_REQUEST)?
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let index = params
        .get("index")
        .ok_or(StatusCode::BAD_REQUEST)?
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let tti = params
        .get("tti")
        .ok_or(StatusCode::BAD_REQUEST)?
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let chart_type = if chart_type == "stack" {
        ChartType::Stack
    } else if chart_type == "pass-thru" {
        let viewport_size = params
            .get("viewport_size")
            .unwrap_or(&"40".to_string())
            .parse()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        ChartType::PassThru(viewport_size)
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let chart = Chart::new(
        caption.clone(),
        chart_type,
        Duration::from_millis(interval),
        y_start..y_end,
    );

    charts
        .insert_chart(index, chart, Duration::from_millis(tti))
        .map_err(|_| StatusCode::CONFLICT)?;

    Ok(Response::builder()
        .body(format!("Success to make new chart in {index}"))
        .unwrap())
}

#[derive(Deserialize)]
struct Label {
    name: String,
    r: u8,
    g: u8,
    b: u8,
}

async fn new_label(
    Extension(charts): Extension<Arc<Charts>>,
    Path(index): Path<usize>,
    Query(label): Query<Label>,
) -> Result<Response<String>, StatusCode> {
    charts
        .new_label(
            index,
            label.name,
            LineColor::init(label.r, label.g, label.b),
        )
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("Success".to_string())
        .unwrap())
}

async fn get_chart_data(
    Extension(charts): Extension<Arc<Charts>>,
    Path(index): Path<usize>,
) -> Result<Response<Body>, StatusCode> {
    let serialized_lines = charts
        .get_lines_as_json_string(index)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let len = serialized_lines.len();

    let body = Body::from(serialized_lines);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("CONTENT-LENGTH", len)
        .header("CONTENT-TYPE", "text/json")
        .body(body)
        .unwrap())
}

async fn get_chart_info(
    Extension(charts): Extension<Arc<Charts>>,
    Path(index): Path<usize>,
) -> Result<Response<Body>, StatusCode> {
    let serialized_info = charts
        .get_info_as_json_string(index)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let len = serialized_info.len();

    let body = Body::from(serialized_info);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("CONTENT-LENGTH", len)
        .header("CONTENT-TYPE", "text/json")
        .body(body)
        .unwrap())
}

#[derive(Deserialize)]
struct Data {
    label: String,
    value: f64,
}

async fn insert_data(
    Extension(charts): Extension<Arc<Charts>>,
    Path(index): Path<usize>,
    Query(data): Query<Data>,
) -> Result<Response<String>, StatusCode> {
    charts
        .insert_data(index, data.label, data.value)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body("Success".to_string())
        .unwrap())
}

async fn serve_file(Path(file_name): Path<String>) -> Response<Body> {
    let file = fs::read(format!("./www/pkg/{}", file_name)).unwrap();

    let mime = mime_guess::from_path(file_name)
        .first()
        .unwrap()
        .to_string();

    let len = file.len();

    let body = Body::from(file);
    Response::builder()
        .status(StatusCode::OK)
        .header("CONTENT-LENGTH", len)
        .header("Content-type", mime)
        .body(body)
        .unwrap()
}
