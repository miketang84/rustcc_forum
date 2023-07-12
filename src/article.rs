use askama::Template;
use axum::{
    extract::{Form, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use gutp_types::{GutpComment, GutpPost, GutpSubspace, GutpUser};
use serde::{Deserialize, Serialize};

use crate::filters;
use crate::redirect_to_error_page;
use crate::AppState;
use crate::HtmlTemplate;
use crate::LoggedUser;
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

#[derive(Deserialize)]
pub struct ViewArticleParams {
    id: String,
}

pub async fn view_article(
    logged_user: Option<Extension<LoggedUser>>,
    Query(params): Query<ViewArticleParams>,
) -> impl IntoResponse {
    let logged_user_id = if let Some(Extension(LoggedUser { user_id })) = logged_user {
        Some(user_id)
    } else {
        None
    };

    // #[derive(Serialize)]
    // struct QueryMaker00 {
    //     id: String,
    // }

    // let query_params = QueryMaker00 {
    //     id: params.id.to_owned(),
    // };

    // or use this for simple case
    let query_params = [("id", &params.id)];
    let posts: Vec<GutpPost> = make_get("/v1/post", &query_params).await.unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        // continue to query comments
        let query_params = [("post_id", &post.id)];
        let comments: Vec<GutpComment> = make_get("/v1/comment/list_by_post", &query_params)
            .await
            .unwrap_or(vec![]);

        // TODO: query tags of this article
        // this is a N:M relationship, so it's relatively complex
        // let query_params = [("post_id", post.id)];
        // let post_tags: Vec<GutpPostTag> = make_get("/v1/posttag/list_by_post", &query_params)
        //     .await
        //     .unwrap_or(vec![]);

        // query coresponding subspace of this article
        let query_params = [("id", &post.subspace_id)];
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
        let query_params = [("id", &post.author_id)];
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
        .into_response()
    } else {
        let action = format!("Query article: {}", params.id);
        let err_info = "Article doesn't exist!";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Template)]
#[template(path = "article_create.html")]
struct ArticleCreateTemplate {
    subspace: GutpSubspace,
}

#[derive(Deserialize)]
pub struct ViewArticleCreateParams {
    subspace_id: String,
}

pub async fn view_article_create(
    logged_user: Option<Extension<LoggedUser>>,
    Query(params): Query<ViewArticleCreateParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info).into_response();
    }

    let inner_params = [("id", &params.subspace_id)];
    let subspaces: Vec<GutpSubspace> = make_get("/v1/subspace", &inner_params)
        .await
        .unwrap_or(vec![]);
    if let Some(subspace) = subspaces.into_iter().next() {
        // render the page
        HtmlTemplate(ArticleCreateTemplate { subspace }).into_response()
    } else {
        let action = format!("Query subspace: {}", &params.subspace_id);
        let err_info = "Subspace doesn't exist, article couldn't be added to it!";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Deserialize)]
pub struct PostArticleCreateParams {
    subspace_id: String,
    title: String,
    content: String,
    extlink: String,
}

pub async fn post_article_create(
    logged_user: Option<Extension<LoggedUser>>,
    Form(params): Form<PostArticleCreateParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info).into_response();
    }
    let Extension(LoggedUser { user_id }) = logged_user.unwrap();

    let inner_params = [("id", &user_id)];
    let users: Vec<GutpUser> = make_get("/v1/user", &inner_params).await.unwrap_or(vec![]);
    // because author isn't the care factor, if it's invalid, just git it a default value
    if users.is_empty() {
        let action = format!("Query user: {}", user_id);
        let err_info = "Unknown error.";
        return redirect_to_error_page(&action, err_info).into_response();
    }
    let user = users[0].to_owned();

    #[derive(Serialize)]
    struct InnerArticleCreateParams {
        title: String,
        content: String,
        author_id: String,
        author_nickname: String,
        subspace_id: String,
        extlink: String,
        profession: String,
        appid: String,
        is_public: bool,
    }

    let inner_params = InnerArticleCreateParams {
        title: params.title,
        content: params.content,
        author_id: user.id.to_owned(),
        author_nickname: user.nickname.to_owned(),
        subspace_id: params.subspace_id.to_owned(),
        extlink: params.extlink,
        profession: crate::APPPROFESSION.to_string(),
        appid: crate::APPID.to_string(),
        is_public: true,
    };

    let posts: Vec<GutpPost> = make_post("/v1/post/create", &inner_params)
        .await
        .unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        // redirect to the article page
        let redirect_uri = format!("/article?id={}", post.id);
        Redirect::to(&redirect_uri).into_response()
    } else {
        // redirect to the error page
        let action = format!("Create article in subspace: {}", &params.subspace_id);
        let err_info = "Unknown";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Template)]
#[template(path = "article_edit.html")]
struct ArticleEditTemplate {
    post: GutpPost,
    // subspace: GutpSubspace,
}

#[derive(Deserialize)]
pub struct ViewArticleEditParams {
    id: String,
}

pub async fn view_article_edit(
    logged_user: Option<Extension<LoggedUser>>,
    Query(params): Query<ViewArticleEditParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info).into_response();
    }

    let inner_params = [("id", &params.id)];
    // get the old article by request to gutp
    let posts: Vec<GutpPost> = make_get("/v1/post", &inner_params).await.unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        HtmlTemplate(ArticleEditTemplate { post }).into_response()
    } else {
        let action = format!("Query Article: {}", &params.id);
        let err_info = "Article doesn't exist!";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Deserialize)]
pub struct PostArticleEditParams {
    id: String,
    title: String,
    content: String,
    extlink: String,
}

pub async fn post_article_edit(
    logged_user: Option<Extension<LoggedUser>>,
    Form(params): Form<PostArticleEditParams>,
) -> Redirect {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info);
    }
    let Extension(LoggedUser { user_id }) = logged_user.unwrap();

    #[derive(Serialize)]
    struct InnerArticleEditParams {
        id: String,
        title: String,
        content: String,
        author_id: String,
        extlink: String,
        is_public: bool,
    }

    let inner_params = InnerArticleEditParams {
        id: params.id.to_owned(),
        title: params.title,
        content: params.content,
        author_id: user_id,
        extlink: params.extlink,
        is_public: true,
    };
    // post to gutp
    let posts: Vec<GutpPost> = make_post("/v1/post/update", &inner_params)
        .await
        .unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        // redirect to the article page
        let redirect_uri = format!("/article?id={}", post.id);
        Redirect::to(&redirect_uri)
    } else {
        // redirect to the error page
        let action = format!("Edit article: {}", &params.id);
        let err_info = "Unknown";
        redirect_to_error_page(&action, err_info)
    }
}

#[derive(Template)]
#[template(path = "article_delete.html")]
struct ArticleDeleteTemplate {
    post: GutpPost,
}

#[derive(Deserialize)]
pub struct ViewArticleDeleteParams {
    id: String,
}

pub async fn view_article_delete(
    logged_user: Option<Extension<LoggedUser>>,
    Query(params): Query<ViewArticleDeleteParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info).into_response();
    }

    let inner_params = [("id", &params.id)];
    let posts: Vec<GutpPost> = make_get("/v1/post", &inner_params).await.unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        // continue to query comments
        let inner_params = [("post_id", &post.id)];
        let comments: Vec<GutpComment> = make_get("/v1/comment/list_by_post", &inner_params)
            .await
            .unwrap_or(vec![]);

        if comments.is_empty() {
            // can be deleted
            HtmlTemplate(ArticleDeleteTemplate { post }).into_response()
        } else {
            // error
            let action = format!("Intend to delete article: {}", &params.id);
            let err_info = "Article has comments attached, could not be deleted!";
            redirect_to_error_page(&action, err_info).into_response()
        }
    } else {
        let action = format!("Query article: {}", params.id);
        let err_info = "Article doesn't exist!";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Deserialize)]
pub struct PostArticleDeleteParams {
    id: String,
    subspace_id: String,
}

pub async fn post_article_delete(
    logged_user: Option<Extension<LoggedUser>>,
    Form(params): Form<PostArticleDeleteParams>,
) -> Redirect {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info);
    }

    // We must precheck the id, we can do it in the params type definition
    let inner_params = [("id", &params.id)];
    let _posts: Vec<GutpPost> = make_post("/v1/post/delete", &inner_params)
        .await
        .unwrap_or(vec![]);

    // TODO: process the error branch of deleting

    // TODO: redirect to an article list page with a tag

    // redirect to index page
    let redirect_uri = format!("/subspace?id={}", &params.subspace_id);
    Redirect::to(&redirect_uri)
}
