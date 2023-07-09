use askama::Template;
use axum::{
    extract::{Form, Query, RawQuery, State},
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use gutp_types::{GutpComment, GutpPost, GutpSubspace, GutpUser};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::HtmlTemplate;
use crate::{make_get, make_post};
use crate::{redirect_to_error_page, LoggedUser};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    subspaces: Vec<GutpSubspace>,
}

pub async fn view_index(logged_user: Option<Extension<LoggedUser>>) -> impl IntoResponse {
    // check the user login status
    if let Some(Extension(logged_user)) = logged_user {
        println!("user: {:?}", logged_user);
    } else {
        println!("no user: {:?}", logged_user);
    }

    let query_params: &[(&str, &str)] = &[];
    // get subspace tags
    let subspaces: Vec<GutpSubspace> = make_get("/v1/subspace/list", query_params)
        .await
        .unwrap_or(vec![]);

    // render the page
    HtmlTemplate(IndexTemplate { subspaces })
}
