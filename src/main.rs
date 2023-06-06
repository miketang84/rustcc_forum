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

async fn index(State(client): State<Client>, mut req: Request<Body>) -> Response<Body> {
    let path = req.uri().path();
    let query = req.uri().query();

    // check the user login status

    // get subspace tags
    let res_bytes = make_get(client, "/v1/tag/list_by_subspace", query).await;
    let obj_vec: Vec<Model> = Model::from(res_bytes);

    // get the latest articles
    let res_bytes = make_get(client, "/v1/post/list_by_subspace", query).await;
    let obj_vec: Vec<Model> = Model::from(res_bytes);

    // get the latest replied articles
    let res_bytes = make_get(client, "/v1/post/list_by_subspace_by_latest_replied", query).await;
    let obj_vec: Vec<Model> = Model::from(res_bytes);

    // get other extensive links (items)
    let res_bytes = make_get(client, "/v1/extobj/list_by_subspace", query).await;
    let obj_vec: Vec<Model> = Model::from(res_bytes);

    // render the page

    // construct the response
}

/// helper function
async fn make_get(
    client: Client,
    path: &str,
    query: Option<&str>,
) -> anyhow::Result<hyper::body::Bytes> {
    let pq = if let Some(query) = query {
        format!("{}?{}", path, query)
    } else {
        format!("{}", path)
    };
    let uri = Uri::builder()
        .scheme("http")
        .authority("127.0.0.1:3000")
        .path_and_query(&pq)
        .build()
        .unwrap();

    let response = client.get(uri).await.unwrap();
    let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
    println!("body: {:?}", body_bytes);
    Ok(body_bytes)
}

async fn make_post(
    client: Client,
    path: &str,
    body: Option<&str>,
) -> anyhow::Result<hyper::body::Bytes> {
    let pq = if let Some(query) = query {
        format!("{}?{}", path, query)
    } else {
        format!("{}", path)
    };
    let uri = Uri::builder()
        .scheme("http")
        .authority("127.0.0.1:3000")
        .path_and_query(&pq)
        .build()
        .unwrap();

    let response = client.get(uri).await.unwrap();
    let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
    println!("body: {:?}", body_bytes);
    Ok(body_bytes)
}
