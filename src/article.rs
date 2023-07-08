use askama::Template;
use axum::{
    extract::{Form, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use gutp_types::{GutpComment, GutpPost, GutpSubspace, GutpUser};
use serde::{Deserialize, Serialize};

use crate::redirect_to_error_page;
use crate::AppState;
use crate::HtmlTemplate;
use crate::{make_get, make_post};

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate {
    post: GutpPost,
    comments: Vec<GutpComment>,
    subspace: GutpSubspace,
    author: GutpUser,
    logged_user_id: Option<String>,
}

struct ViewArticleParams {
    id: String,
}

pub async fn view_article(
    State(app_state): State<AppState>,
    Query(params): Query<ViewArticleParams>,
    RawQuery(query): RawQuery,
    Extension(logged_user_id): Extension<Option<String>>,
) -> impl IntoResponse {
    // check the user login status

    // We must specify the subspace_id
    let id = params.id;

    // #[derive(Serialize)]
    // struct QueryMaker00 {
    //     id: String,
    // }

    // let query_params = QueryMaker00 {
    //     id: params.id.to_owned(),
    // };

    // or use this for simple case
    let query_params = [("id", params.id)];
    let posts: Vec<GutpPost> = make_get("/v1/post", &query_params).await.unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        // continue to query comments
        let query_params = [("post_id", post.id)];
        let comments: Vec<GutpComment> = make_get("/v1/comment/list_by_post_id", &query_params)
            .await
            .unwrap_or(vec![]);

        // TODO: query tags of this article
        // this is a N:M relationship, so it's relatively complex
        // let query_params = [("post_id", post.id)];
        // let post_tags: Vec<GutpPostTag> = make_get("/v1/posttag/list_by_post_id", &query_params)
        //     .await
        //     .unwrap_or(vec![]);

        // query coresponding subspace of this article
        let query_params = [("id", post.subspace_id)];
        let sps: Vec<GutpSubspace> = make_get("/v1/subspace", &query_params)
            .await
            .unwrap_or(vec![]);
        // because subspace isn't the care factor, if it's invalid, just git it a default value
        let subspace = if sps.is_empty() {
            Default::default()
        } else {
            sps[0].to_owned()
        };

        // query coresponding author of this article
        let query_params = [("id", post.author_id)];
        let authors: Vec<GutpUser> = make_get("/v1/user", &query_params).await.unwrap_or(vec![]);
        // because author isn't the care factor, if it's invalid, just git it a default value
        let author = if authors.is_empty() {
            Default::default()
        } else {
            authors[0].to_owned()
        };

        // if user logged in, add it
        // let query_params = [("id", logged_user_id)];
        // let users: Vec<GutpUser> = make_get("/v1/user", &query_params).await.unwrap_or(vec![]);
        // // because author isn't the care factor, if it's invalid, just git it a default value
        // let user = if users.is_empty() {
        //     None
        // } else {
        //     Some(users[0].to_owned())
        // };

        // render the page
        HtmlTemplate(ArticleTemplate {
            post,
            comments,
            subspace,
            author,
            logged_user_id,
        })
    } else {
        let action = format!("Query article: {}", params.id);
        let err_info = "Article doesn't exist!";
        redirect_to_error_page(&action, err_info)
    }
}

#[derive(Template)]
#[template(path = "article_create.html")]
struct ArticleCreateTemplate {
    subspace: GutpSubspace,
}

struct ViewArticleCreateParams {
    subspace_id: String,
}

pub async fn view_article_create(
    State(app_state): State<AppState>,
    Query(params): Query<ViewArticleCreateParams>,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    // check the user login status

    // We must specify the subspace_id
    let subspace_id = params.subspace_id;

    // get the post object from the gutp service, to check the exsistence of that post
    let query = format!("id={}", subspace_id);
    let res_bytes = make_get(app_state.hclient, "/v1/subspace", Some(query)).await;
    let subspaces: Vec<GutpSubspace> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);
    if let Some(subspace) = subspaces.into_iter().next() {
        // render the page
        HtmlTemplate(ArticleCreateTemplate { subspace })
    } else {
        let action = format!("Query subspace: {}", subspace_id);
        let err_info = "Subspace doesn't exist, article couldn't be added to it!";
        redirect_to_error_page(&action, err_info)
    }
}

struct PostArticleCreateParams {
    subspace_id: String,
}

pub async fn post_article_create(
    State(app_state): State<AppState>,
    Form(params): Form<PostArticleCreateParams>,
    body: String,
) -> impl IntoResponse {
    // check the user login status

    // TODO: parse form params
    let subspace_id = params.subspace_id;

    // TODO: check the existence of subspace by query

    // forward post form body to gutp
    let res_bytes = make_post(app_state.hclient, "/v1/post/create", body).await;
    let posts: Vec<GutpPost> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        // redirect to the article page
        let redirect_uri = format!("/article?id={}", post.id);
        Redirect::to(&redirect_uri)
    } else {
        // redirect to the error page
        let action = format!("Create article in subspace: {}", subspace_id);
        let err_info = "Unknown";
        redirect_to_error_page(&action, err_info)
    }
}

#[derive(Template)]
#[template(path = "article_edit.html")]
struct ArticleEditTemplate {
    post: GutpPost,
    // subspace: GutpSubspace,
}

struct ViewArticleEditParams {
    id: String,
}

pub async fn view_article_edit(
    State(app_state): State<AppState>,
    Query(params): Query<ViewArticleEditParams>,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    // check the user login status

    // We must specify the tag_id, We can do it in the params type definition
    let post_id = params.id;

    // get the old article by request to gutp
    let res_bytes = make_get(app_state.hclient, "/v1/article", query).await;
    let posts: Vec<GutpPost> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        HtmlTemplate(ArticleEditTemplate { post })
    } else {
        let action = format!("Query Article: {}", post_id);
        let err_info = "Article doesn't exist!";
        redirect_to_error_page(&action, err_info)
    }
}

struct PostArticleEditParams {
    id: String,
}

pub async fn post_article_edit(
    State(app_state): State<AppState>,
    Form(params): Form<PostArticleEditParams>,
    body: String,
) -> Redirect {
    // check the user login status

    // We must precheck the tag_id, we can do it in the params type definition
    // let tag_id = params.id;

    // post to gutp
    let res_bytes = make_post(app_state.hclient, "/v1/article/edit", body).await;
    let posts: Vec<GutpPost> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        // redirect to the article page
        let redirect_uri = format!("/article?id={}", post.id);
        Redirect::to(&redirect_uri)
    } else {
        // redirect to the error page
        let action = format!("Edit article: {}", params.id);
        let err_info = "Unknown";
        redirect_to_error_page(&action, err_info)
    }
}

#[derive(Template)]
#[template(path = "article_delete.html")]
struct ArticleDeleteTemplate {
    post_id: String,
    title: String,
}

struct ViewArticleDeleteParams {
    id: String,
    title: String,
}

pub async fn view_article_delete(
    State(app_state): State<AppState>,
    Query(params): Query<ViewArticleDeleteParams>,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    // check the user login status

    // We must specify the tag_id, We can do it in the params type definition
    let id = params.id;
    let title = params.title;

    let query = format!("post_id={}", id);
    // get the old tag by request to gutp
    let res_bytes = make_get(
        app_state.hclient,
        "/v1/comment/list_by_post_id",
        Some(query),
    )
    .await;
    let comments: Vec<GutpComment> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);
    if comments.is_empty() {
        // can be deleted
        HtmlTemplate(ArticleDeleteTemplate { post_id: id, title })
    } else {
        // error
        let action = format!("Intend to delete article: {}", id);
        let err_info = "Article has comments attached, could not be deleted!";
        redirect_to_error_page(&action, err_info)
    }
}

struct PostArticleDeleteParams {
    id: String,
    subspace_id: String,
}

pub async fn post_article_delete(
    State(app_state): State<AppState>,
    Form(params): Form<PostArticleDeleteParams>,
    body: String,
) -> Redirect {
    // check the user login status

    // We must precheck the id, we can do it in the params type definition
    let post_id = params.id;
    let subspace_id = params.subspace_id;

    let res_bytes = make_post(app_state.hclient, "/v1/post/delete", body).await;
    let _posts: Vec<GutpPost> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);

    // TODO: process the error branch of deleting

    // TODO: redirect to an article list page with a tag
    // redirect to index page
    let redirect_uri = format!("/subspace?id={}", subspace_id);
    Redirect::to(&redirect_uri)
}
