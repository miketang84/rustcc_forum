#![allow(unused)]

use axum::{
    extract::State,
    http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
use std::net::SocketAddr;
// use tower_http::services::{ServeDir, ServeFile};

use hyper::{client::HttpConnector, Body};
type Client = hyper::client::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // println!("listening on {}", addr);
    // let serve_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));

    // let app = Router::new()
    //     .route("/foo", get(|| async { "Hi from /foo" }))
    //     .nest_service("/assets", serve_dir.clone())
    //     .fallback_service(serve_dir);

    // axum::Server::bind(&addr)
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();

    let client = Client::new();

    let app = Router::new().route("/", get(handler)).with_state(client);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    println!("reverse proxy listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(State(client): State<Client>, mut req: Request<Body>) -> Response<Body> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("http://127.0.0.1:3000{}", path_query);

    *req.uri_mut() = Uri::try_from(uri).unwrap();

    client.request(req).await.unwrap()
}
