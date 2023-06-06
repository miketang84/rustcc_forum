#![allow(unused)]

use axum::{
    extract::State,
    http::{uri::Uri, Request, Response},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
// use tower_http::services::{ServeDir, ServeFile};

use hyper::{client::HttpConnector, Body};
type Client = hyper::client::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let app = Router::new()
        .route("/", get(handler))
        .route("/latest_articles", get(handler))
        .route("/latest_replied_articles", get(handler))
        .route("/latest_articles_by_tag", get(handler))
        .route("/latest_replied_articles_by_tag", get(handler))
        .route("/sp", get(handler))
        .route("/sp/create", get(handler).post(handler))
        .route("/sp/edit", get(handler).post(handler))
        .route("/article", get(handler))
        .route("/article/create", get(handler).post(handler))
        .route("/article/edit", get(handler).post(handler))
        .route("/article/delete", get(handler).post(handler))
        .route("/comment/create", get(handler).post(handler))
        .route("/comment/edit", get(handler).post(handler))
        .route("/comment/delete", get(handler).post(handler))
        .route("/tag/create", get(handler).post(handler))
        .route("/tag/edit", get(handler).post(handler))
        .route("/tag/delete", get(handler).post(handler))
        .route("/manage/my_articles", get(handler))
        .route("/manage/my_tags", get(handler))
        .route("/manage/pubprofile", get(handler).post(handler))
        .route("/user/account", get(handler))
        .route("/user/signout", get(handler))
        .route("/user/register", post(handler))
        .route("/user/login", post(handler))
        .route("/user/login_with3rd", get(handler))
        .route("/user/login_with_github", get(handler))
        .with_state(client);

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
