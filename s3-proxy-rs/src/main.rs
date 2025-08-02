use std::sync::Arc;

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{config::Region, Client};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Response, StatusCode},
    response::Html,
    routing::get,
    Router,
};
use dotenv::dotenv;
use mime_guess::mime;

use crate::errors::Error;

type Result<T, E = Error> = std::result::Result<T, E>;

mod errors;

#[derive(Debug)]
struct AppState {
    client: Client,
    bucket_name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let listener = tokio::net::TcpListener::bind(format!(
        "0.0.0.0:{}",
        dotenv::var("SERVER_PORT").map_err(|e| Error::InternalError(e.to_string()))?
    ))
    .await
    .map_err(|e| Error::ConnectionError(e))?;

    let region_provider = RegionProviderChain::first_try(Region::new(
        dotenv::var("AWS_REGION").map_err(|e| Error::InternalError(e.to_string()))?,
    ));

    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let client = Client::new(&shared_config);
    let bucket_name =
        dotenv::var("BUCKET_NAME").map_err(|e| Error::InternalError(e.to_string()))?;

    let state = Arc::new(AppState {
        client,
        bucket_name,
    });

    let app = Router::new()
        .route("/{*path}", get(generic_handler))
        .route("/", get(root))
        .with_state(state);

    Ok(axum::serve(listener, app).await.unwrap())
}

async fn root(State(state): State<Arc<AppState>>) -> Html<String> {
    let path = "index.html";
    let client = &state.client;
    let bucket_name = &state.bucket_name;

    let object = client
        .get_object()
        .bucket(bucket_name)
        .key(path)
        .send()
        .await
        .map_err(|e| Error::InternalError(e.to_string()))
        .expect("Expected to get object");

    let buf = object.body.collect().await.unwrap().into_bytes().to_vec();

    let content = String::from_utf8(buf).unwrap();

    Html(content)
}

async fn generic_handler(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Response<Body> {
    let mut path = match path_clean::clean(&path).to_str() {
        Some(path) => path,
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Path not found"))
                .unwrap()
        }
    }
    .to_string();

    let content_type = match mime_guess::from_path(&path).first() {
        Some(mime) => mime,
        None => mime::TEXT_HTML,
    };

    if content_type == mime::TEXT_HTML {
        path = path + "/index.html";
    }

    let client = &state.client;
    let bucket_name = &state.bucket_name;

    let object = match client
        .get_object()
        .bucket(bucket_name)
        .key(path)
        .send()
        .await
    {
        Ok(object) => object,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(e.to_string()))
                .expect("Expected to build a response");
        }
    };

    let buf = match object.body.collect().await {
        Ok(buf) => buf,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(e.to_string()))
                .expect("Expected to build a response");
        }
    }
    .into_bytes()
    .to_vec();

    let body = Body::from(buf);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type.to_string())
        .body(body)
        .expect("Expected to build a response")
}
