use askama::Template;
use axum::{
    extract::{Form, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use gutp_types::{GutpComment, GutpPost, GutpSubspace};
use serde::{Deserialize, Serialize};

use crate::filters;
use crate::redirect_to_error_page;
use crate::HtmlTemplate;
use crate::LoggedUser;
use crate::{make_get, make_post};

#[derive(Template)]
#[template(path = "subspace.html")]
struct SubspaceTemplate {
    subspace: GutpSubspace,
    posts: Vec<GutpPost>,
}

#[derive(Deserialize)]
pub struct ViewSubspaceParams {
    id: String,
}

pub async fn view_subspace(
    logged_user: Option<Extension<LoggedUser>>,
    Query(params): Query<ViewSubspaceParams>,
) -> impl IntoResponse {
    let inner_params = [("id", &params.id)];
    let subspaces: Vec<GutpSubspace> = make_get("/v1/subspace", &inner_params)
        .await
        .unwrap_or(vec![]);
    if let Some(sp) = subspaces.into_iter().next() {
        let inner_params = [("subspace_id", &sp.id)];
        let posts: Vec<GutpPost> = make_get("/v1/post/list_by_subspace", &inner_params)
            .await
            .unwrap_or(vec![]);
        HtmlTemplate(SubspaceTemplate {
            subspace: sp,
            posts,
        })
        .into_response()
    } else {
        // redirect to the error page
        let action = format!("Query subspace: {}", params.id);
        let err_info = "No this subspace.";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Template)]
#[template(path = "subspace_create.html")]
struct SubspaceCreateTemplate {}

#[derive(Deserialize)]
pub struct ViewSubspaceCreateParams {}

pub async fn view_subspace_create(
    logged_user: Option<Extension<LoggedUser>>,
    Query(params): Query<ViewSubspaceCreateParams>,
) -> impl IntoResponse {
    // check the user login status
    // TODO: who can create a new subspace?
    // For forum case, only admin has the permission to create a new subspace
    // for blog case, every on can create their own blog subspace
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info).into_response();
    }

    HtmlTemplate(SubspaceCreateTemplate {}).into_response()
}

#[derive(Deserialize)]
pub struct PostSubspaceCreateParams {
    title: String,
    description: String,
}

pub async fn post_subspace_create(
    logged_user: Option<Extension<LoggedUser>>,
    Form(params): Form<PostSubspaceCreateParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info);
    }
    let Extension(LoggedUser { user_id }) = logged_user.unwrap();

    #[derive(Serialize)]
    struct InnerSubspaceCreateParams {
        title: String,
        description: String,
        banner: String,
        owner_id: String,
        profession: String,
        appid: String,
        is_public: bool,
        slug: String,
    }

    let inner_params = InnerSubspaceCreateParams {
        title: params.title,
        description: params.description,
        banner: "".to_string(),
        owner_id: user_id,
        profession: crate::APPPROFESSION.to_string(),
        appid: crate::APPID.to_string(),
        is_public: true,
        slug: "".to_string(),
    };

    let subspaces: Vec<GutpSubspace> = make_post("/v1/subspace/create", &inner_params)
        .await
        .unwrap_or(vec![]);
    if let Some(sp) = subspaces.into_iter().next() {
        let redirect_uri = format!("/subspace?id={}", sp.id);
        Redirect::to(&redirect_uri)
    } else {
        // redirect to the error page
        let action = format!("Create subspace");
        let err_info = "Unknown";
        redirect_to_error_page(&action, err_info)
    }
}

/*
#[derive(Template)]
#[template(path = "subspace_edit.html")]
struct SubspaceEditTemplate {
    subspace: GutpSubspace,
}

struct ViewSubspaceEditParams {
    id: String,
}

pub async fn view_subspace_edit(
    State(client): State<Client>,
    Query(params): Query<ViewSubspaceEditParams>,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    // check the user login status

    // We must specify the tag_id, We can do it in the params type definition
    let sp_id = params.id;

    let res_bytes = make_get(client, "/v1/subspace", query).await;
    let sps: Vec<GutpSubspace> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);
    if let Some(sp) = sps.into_iter().next() {
        HtmlTemplate(SubspaceEditTemplate { subspace: sp })
    } else {
        let action = format!("Query subspace: {}", sp_id);
        let err_info = "Subspace doesn't exist!";
        redirect_to_error_page(&action, err_info)
    }
}

struct PostSubspaceEditParams {
    id: String,
}

pub async fn post_subspace_edit(
    State(client): State<Client>,
    Form(params): Form<PostSubspaceEditParams>,
    body: String,
) -> Redirect {
    // check the user login status

    // We must precheck the tag_id, we can do it in the params type definition
    let id = params.id;

    // post to gutp
    let res_bytes = make_post(client, "/v1/subspace/edit", body).await;
    let sps: Vec<GutpSubspace> = serde_json::from_slice(res_bytes).unwrap_or(vec![]);
    if let Some(sp) = sps.into_iter().next() {
        let redirect_uri = format!("/subspace?id={}", id);
        Redirect::to(&redirect_uri)
    } else {
        // redirect to the error page
        let action = format!("Edit subspace: {}", id);
        let err_info = "Unknown";
        redirect_to_error_page(&action, err_info)
    }
}
*/

#[derive(Template)]
#[template(path = "subspace_delete.html")]
struct SubspaceDeleteTemplate {
    subspace: GutpSubspace,
}

#[derive(Deserialize)]
pub struct ViewSubspaceDeleteParams {
    id: String,
}

pub async fn view_subspace_delete(
    logged_user: Option<Extension<LoggedUser>>,
    Query(params): Query<ViewSubspaceDeleteParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info).into_response();
    }

    let inner_params = [("id", &params.id)];
    let subspaces: Vec<GutpSubspace> = make_get("/v1/subspace", &inner_params)
        .await
        .unwrap_or(vec![]);
    if let Some(sp) = subspaces.into_iter().next() {
        let inner_params = [("subspace_id", &sp.id)];
        let posts: Vec<GutpPost> = make_get("/v1/post/list_by_subspace_id", &inner_params)
            .await
            .unwrap_or(vec![]);
        if posts.is_empty() {
            // can be deleted
            HtmlTemplate(SubspaceDeleteTemplate { subspace: sp }).into_response()
        } else {
            // error
            let action = format!("Intend to delete subspace: {}", sp.id);
            let err_info = "This subspace has article attached, could not be deleted!";
            redirect_to_error_page(&action, err_info).into_response()
        }
    } else {
        // redirect to the error page
        let action = format!("Query subspace: {}", params.id);
        let err_info = "No this subspace.";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Deserialize)]
pub struct PostSubspaceDeleteParams {
    id: String,
}

pub async fn post_subspace_delete(
    logged_user: Option<Extension<LoggedUser>>,
    Form(params): Form<PostSubspaceDeleteParams>,
) -> impl IntoResponse {
    // check the user login status
    if logged_user.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info);
    }

    let inner_params = [("id", &params.id)];
    let subspaces: Vec<GutpSubspace> = make_get("/v1/subspace", &inner_params)
        .await
        .unwrap_or(vec![]);
    if let Some(sp) = subspaces.into_iter().next() {
        let inner_params = [("subspace_id", &sp.id)];
        let posts: Vec<GutpPost> = make_get("/v1/post/list_by_subspace", &inner_params)
            .await
            .unwrap_or(vec![]);
        if posts.is_empty() {
            // can be deleted
            let inner_params = [("id", &sp.id)];
            let _sps: Vec<GutpSubspace> = make_post("/v1/subspace/delete", &inner_params)
                .await
                .unwrap_or(vec![]);

            // redirect to index page
            Redirect::to("/")
        } else {
            // error
            let action = format!("Intend to delete subspace: {}", sp.id);
            let err_info = "This subspace has article attached, could not be deleted!";
            redirect_to_error_page(&action, err_info)
        }
    } else {
        // redirect to the error page
        let action = format!("Query subspace: {}", params.id);
        let err_info = "No this subspace.";
        redirect_to_error_page(&action, err_info)
    }
}
