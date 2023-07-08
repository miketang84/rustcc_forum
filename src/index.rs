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
use crate::{redirect_to_error_page, LoggedUserId};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    subspaces: Vec<GutpSubspace>,
}

pub async fn view_index(Extension(logged_user_id): Extension<LoggedUserId>) -> impl IntoResponse {
    // check the user login status

    let query_params: &[(&str, &str)] = &[];
    // get subspace tags
    let subspaces: Vec<GutpSubspace> = make_get("/v1/subspace/list", query_params)
        .await
        .unwrap_or(vec![]);

    // render the page
    HtmlTemplate(IndexTemplate { subspaces })
}
