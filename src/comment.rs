use askama::Template;
use axum::{
    extract::{Form, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use gutp_types::{GutpComment, GutpPost, GutpUser};
use serde::{Deserialize, Serialize};

use crate::redirect_to_error_page;
use crate::HtmlTemplate;
use crate::LoggedUser;
use crate::{make_get, make_post};

#[derive(Template)]
#[template(path = "comment_create.html")]
struct CommentCreateTemplate {
    post: GutpPost,
}

#[derive(Deserialize)]
pub struct ViewCommentCreateParams {
    post_id: String,
}

pub async fn view_comment_create(
    logged_user: Option<Extension<LoggedUser>>,
    Query(params): Query<ViewCommentCreateParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info).into_response();
    }

    let inner_params = [("id", &params.post_id)];
    let posts: Vec<GutpPost> = make_get("/v1/post", &inner_params).await.unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        HtmlTemplate(CommentCreateTemplate { post }).into_response()
    } else {
        let action = format!("Query Article: {}", &params.post_id);
        let err_info = "Article doesn't exist, comment couldn't be added to it!";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Deserialize)]
pub struct PostCommentCreateParams {
    post_id: String,
    content: String,
}

pub async fn post_comment_create(
    logged_user: Option<Extension<LoggedUser>>,
    Form(params): Form<PostCommentCreateParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info);
    }
    let Extension(LoggedUser { user_id }) = logged_user.unwrap();

    let inner_params = [("id", &params.post_id)];
    let posts: Vec<GutpPost> = make_get("/v1/post", &inner_params).await.unwrap_or(vec![]);
    if let Some(post) = posts.into_iter().next() {
        // retreive author info
        let inner_params = [("id", &user_id)];
        let authors: Vec<GutpUser> = make_get("/v1/user", &inner_params).await.unwrap_or(vec![]);
        if let Some(author) = authors.into_iter().next() {
            #[derive(Serialize)]
            struct InnerCommentCreateParams {
                content: String,
                author_id: String,
                author_nickname: String,
                post_id: String,
                parent_comment_id: String,
                is_public: bool,
            }

            let inner_params = InnerCommentCreateParams {
                content: params.content.to_owned(),
                author_id: author.id.to_owned(),
                author_nickname: author.nickname.to_owned(),
                post_id: post.id.to_owned(),
                parent_comment_id: "".to_owned(),
                is_public: true,
            };

            let comments: Vec<GutpComment> = make_post("/v1/comment/create", &inner_params)
                .await
                .unwrap_or(vec![]);
            if let Some(comment) = comments.into_iter().next() {
                // redirect to the article page
                let redirect_uri = format!("/article?id={}", comment.post_id);
                Redirect::to(&redirect_uri)
            } else {
                // redirect to the error page
                let action = format!("Create comment for article: {}", &post.id);
                let err_info = "Unknown";
                redirect_to_error_page(&action, err_info)
            }
        } else {
            let action = format!("Query author: {}", &user_id);
            let err_info = "Unknown";
            redirect_to_error_page(&action, err_info)
        }
    } else {
        let action = format!("Query Article: {}", &params.post_id);
        let err_info = "Article doesn't exist, comment couldn't be added to it!";
        redirect_to_error_page(&action, err_info)
    }
}

#[derive(Template)]
#[template(path = "comment_delete.html")]
struct CommentDeleteTemplate {
    comment: GutpComment,
}

#[derive(Deserialize)]
pub struct ViewCommentDeleteParams {
    id: String,
}

pub async fn view_comment_delete(
    logged_user: Option<Extension<LoggedUser>>,
    Query(params): Query<ViewCommentDeleteParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info).into_response();
    }

    let inner_params = [("id", &params.id)];
    let comments: Vec<GutpComment> = make_get("/v1/comment", &inner_params)
        .await
        .unwrap_or(vec![]);
    if let Some(comment) = comments.into_iter().next() {
        HtmlTemplate(CommentDeleteTemplate { comment }).into_response()
    } else {
        let action = format!("Query comment: {}", &params.id);
        let err_info = "Comment doesn't exist!";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Deserialize)]
pub struct PostCommentDeleteParams {
    id: String,
    post_id: String,
}

pub async fn post_comment_delete(
    logged_user: Option<Extension<LoggedUser>>,
    Form(params): Form<PostCommentDeleteParams>,
) -> Redirect {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info);
    }

    let inner_params = [("id", &params.id)];
    let _comments: Vec<GutpComment> = make_post("/v1/comment/delete", &inner_params)
        .await
        .unwrap_or(vec![]);

    // TODO: process the error branch of deleting

    let redirect_uri = format!("/article?id={}", &params.post_id);
    Redirect::to(&redirect_uri)
}
