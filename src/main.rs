#![allow(unused)]
use askama::Template;
use axum::{
    extract::{RawQuery, State},
    http::{response, uri::Uri, Request, Response},
    response::{Html, IntoResponse},
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
        .route("/", get(index))
        .route("/latest_articles", get(latest_articles))
        .route("/latest_replied_articles", get(handler))
        .route("/latest_articles_by_tag", get(handler))
        .route("/latest_replied_articles_by_tag", get(handler))
        .route("/sp", get(handler))
        .route("/sp/create", get(handler).post(handler))
        .route("/sp/edit", get(handler).post(handler))
        .route("/article", get(article))
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

use gutp_types::GutpPost;
use gutp_types::GutpTag;
use gutp_types::{GutpComment, GutpExtobj};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    tags: Vec<GutpTag>,
    posts: Vec<GutpPost>,
    replied_posts: Vec<GutpPost>,
    extobjs: Vec<GutpExtobj>,
}

async fn index(State(client): State<Client>, RawQuery(query): RawQuery) -> impl IntoResponse {
    // check the user login status

    // get subspace tags
    let res_bytes = make_get(client, "/v1/tag/list_by_subspace", query).await;
    let tags: Vec<GutpTag> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // get the latest articles
    let res_bytes = make_get(client, "/v1/post/list_by_subspace", query).await;
    let posts: Vec<GutpPost> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // get the latest replied articles
    let res_bytes = make_get(client, "/v1/post/list_by_subspace_by_latest_replied", query).await;
    let replied_posts: Vec<GutpPost> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // get other extensive links (items)
    let res_bytes = make_get(client, "/v1/extobj/list_by_subspace", query).await;
    let extobjs: Vec<GutpExtobj> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // render the page

    let template = IndexTemplate {
        tags,
        posts,
        replied_posts,
        extobjs,
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "articles.html")]
struct LatestArticlesTemplate {
    tags: Vec<GutpTag>,
    posts: Vec<GutpPost>,
    extobjs: Vec<GutpExtobj>,
}

async fn latest_articles(
    State(client): State<Client>,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    // check the user login status

    // query format: id=xxxxx, id is the id of subspace

    // get subspace tags
    let res_bytes = make_get(client, "/v1/tag/list_by_subspace", query).await;
    let tags: Vec<GutpTag> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // get the latest articles
    let res_bytes = make_get(client, "/v1/post/list_by_subspace", query).await;
    let posts: Vec<GutpPost> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // paging information

    // get other extensive links (items)
    let res_bytes = make_get(client, "/v1/extobj/list_by_subspace", query).await;
    let extobjs: Vec<GutpExtobj> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // render the page

    let template = LatestArticlesTemplate {
        tags,
        posts,
        extobjs,
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate {
    posts: Vec<GutpPost>,
    comments: Vec<GutpComment>,
    tags: Vec<GutpTag>,
}

async fn article(State(client): State<Client>, RawQuery(query): RawQuery) -> impl IntoResponse {
    // check the user login status

    // query is like: id=xxxxxxxxxx
    // id is the article id

    // get this article
    let res_bytes = make_get(client, "/v1/post", query).await;
    let posts: Vec<GutpPost> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // get comments of this article
    let res_bytes = make_get(client, "/v1/comment/list_by_post", query).await;
    let comments: Vec<GutpComment> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // get subspace tags
    let res_bytes = make_get(client, "/v1/tag/list_by_post", query).await;
    let tags: Vec<GutpTag> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // render the page
    let template = ArticleTemplate {
        posts,
        comments,
        tags,
    };
    HtmlTemplate(template)
}

/// helper function
async fn make_get(
    client: Client,
    path: &str,
    query: Option<String>,
) -> anyhow::Result<hyper::body::Bytes> {
    use hyper::{Body, Client, Method, Request};
    let uri = if let Some(query) = query {
        format!("http://127.0.0.1:3000{}?{}", path, query)
    } else {
        format!("http://127.0.0.1:3000{}", path)
    };

    let req = Request::builder()
        .method(Method::GET)
        .uri(&uri)
        .expect("request builder");

    let response = client.request(req).await.unwrap();
    let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
    println!("body: {:?}", body_bytes);
    Ok(body_bytes)
}

async fn make_post(client: Client, path: &str, body: String) -> anyhow::Result<hyper::body::Bytes> {
    use hyper::{Body, Client, Method, Request};

    let uri = format!("http://127.0.0.1:3000{}", path);

    let req = Request::builder()
        .method(Method::POST)
        .uri(&uri)
        .body(body)
        .expect("request builder");

    let response = client.request(req).await.unwrap();
    let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
    println!("body: {:?}", body_bytes);
    Ok(body_bytes)
}

/// Define the template handler
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
