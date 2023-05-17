#![allow(unused)]

use axum::{
    extract::{Json, State},
    http::{uri::Uri, Request, Response},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
// use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/v1/subspace", get(subspace_by_id))
        .route("/v1/subspace/list", get(subspace_list))
        .route("/v1/subspace/list_by_owner", get(subspace_list_by_owner))
        .route(
            "/v1/subspace/list_by_profession",
            get(subspace_list_by_profession),
        )
        .route("/v1/subspace/list_by_appid", get(subspace_list_by_appid))
        .route("/v1/subspace/create", post(subspace_create))
        .route("/v1/subspace/update", post(subspace_update))
        .route("/v1/subspace/delete", post(subspace_delete))
        .route("/v1/post", get(handler))
        .route("/v1/post/list", get(handler))
        .route("/v1/post/list_by_subspace", get(handler))
        .route("/v1/post/list_by_author", get(handler))
        .route("/v1/post/list_by_profession", get(handler))
        .route("/v1/post/list_by_appid", get(handler))
        .route("/v1/post/create", post(post_create))
        .route("/v1/post/update", post(post_update))
        .route("/v1/post/delete", post(post_delete))
        .route("/v1/comment", get(comment_get_by_id))
        .route("/v1/comment/list", get(comment_get_by_id))
        .route("/v1/comment/list_by_post", get(comment_get_by_id))
        .route("/v1/comment/list_by_author", get(comment_get_by_id))
        .route("/v1/comment/create", post(comment_get_by_id))
        .route("/v1/comment/update", post(comment_get_by_id))
        .route("/v1/comment/delete", post(comment_get_by_id))
        .route("/v1/tag", get(tag_get_by_id))
        .route("/v1/tag/list", get(handler))
        .route("/v1/tag/list_by_subspace", get(handler))
        .route("/v1/tag/list_by_creator", get(handler))
        .route("/v1/tag/create", post(handler))
        .route("/v1/tag/update", post(handler))
        .route("/v1/tag/delete", post(handler))
        .route("/v1/posttag", get(posttag_get_by_id))
        .route("/v1/posttag/list", get(posttag_get_by_id))
        .route("/v1/posttag/list_by_post", get(posttag_get_by_id))
        .route("/v1/posttag/list_by_tag", get(posttag_get_by_id))
        .route("/v1/posttag/create", post(handler))
        .route("/v1/posttag/update", post(handler))
        .route("/v1/posttag/delete", post(handler))
        .route("/v1/postdiff", get());
        .route("/v1/postdiff/list", get());
        .route("/v1/postdiff/list_by_post", get());
        .route("/v1/postdiff/create", post());
        .route("/v1/postdiff/update", post());
        .route("/v1/postdiff/delete", post());
        .route("/v1/moderator", get(moderator_get_by_id))
        .route("/v1/moderator/list", get(handler))
        .route("/v1/moderator/list_by_subspace", get(handler))
        .route("/v1/moderator/list_by_user", get(handler))
        .route("/v1/moderator/list_by_tag", get(handler))
        .route("/v1/moderator/create", post(handler))
        .route("/v1/moderator/update", post(handler))
        .route("/v1/moderator/delete", post(handler))
        .route("/v1/user", get(user_by_id))
        .route("/v1/user_by_account", get(user_by_id))
        .route("/v1/user/list", get(posttag_get_by_id))
        .route("/v1/user/create", post(handler))
        .route("/v1/user/update", post(handler))
        .route("/v1/user/delete", post(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    println!("mock server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn subspace_by_id() -> Json<String> {
    Json(String::from("hello"))
}

async fn handler() -> Json<String> {
    Json(String::from("hello"))
}
