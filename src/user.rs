use askama::Template;
use axum::{
    extract::{Form, Query, RawQuery, State},
    http::header,
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use gutp_types::{GutpComment, GutpPost, GutpSubspace, GutpUser};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::redirect_to_error_page;
use crate::AppState;
use crate::HtmlTemplate;
use crate::LoggedUserId;
use crate::{make_get, make_post};

const TTL: usize = 60 * 24 * 3600;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    client_id: String,
}

pub async fn view_login() -> impl IntoResponse {
    let client_id = dotenv::var("GITHUB_APP_CLIENT_ID").unwrap();
    HtmlTemplate(LoginTemplate { client_id })
}

#[derive(Template)]
#[template(path = "account.html")]
struct AccountTemplate {
    user: GutpUser,
}

pub async fn view_account(Extension(logged_user_id): Extension<LoggedUserId>) -> impl IntoResponse {
    // if logged_user_id.is_none() {
    //     let action = format!("Not logged in");
    //     let err_info = "Need login firstly to get proper permission.";
    //     return redirect_to_error_page(&action, err_info);
    // }

    // has login info
    if let Some(user_id) = logged_user_id {
        // render user info page
        let inner_params = [("id", &user_id)];
        let users: Vec<GutpUser> = make_get("/v1/user", &inner_params).await.unwrap_or(vec![]);
        if let Some(user) = users.into_iter().next() {
            HtmlTemplate(AccountTemplate { user }).into_response()
        } else {
            let action = format!("Query user: {}", &user_id);
            let err_info = "Unknown.";
            redirect_to_error_page(&action, err_info).into_response()
        }
    } else {
        // if not logged in, redirect to login page
        let redirect_uri = format!("/user/login");
        Redirect::to(&redirect_uri).into_response()
    }
}

#[derive(Deserialize)]
pub struct GithubOauthCallbackParams {
    code: String,
}

pub async fn github_oauth_callback(
    State(app_state): State<AppState>,
    Query(params): Query<GithubOauthCallbackParams>,
) -> impl IntoResponse {
    let mut redis_conn = app_state.rclient.get_async_connection().await.unwrap();
    // returned from github
    let code = params.code;

    // get github app client_id and secret
    let client_id = dotenv::var("GITHUB_APP_CLIENT_ID").unwrap();
    let client_secret = dotenv::var("GITHUB_APP_CLIENT_SECRET").unwrap();

    if let Ok(github_credentials) = get_github_token(&code, &client_id, &client_secret).await {
        // use this access_token to retreive user info
        if let Ok(github_user_info) = get_github_user_info(&github_credentials.access_token).await {
            let account = github_user_info.account.to_owned();
            // now we get user info from github
            // we use the account to check whether this user exist in gutp
            let inner_params = [("account", &account)];
            let users: Vec<GutpUser> = make_get("/v1/user/get_by_account", &inner_params)
                .await
                .unwrap_or(vec![]);
            if let Some(user) = users.into_iter().next() {
                // if user exists, log it in
                login_user_with_account(redis_conn, &account)
                    .await
                    .into_response()
            } else {
                // if user doesn't exist, register it

                #[derive(Serialize)]
                struct InnerUserCreateParams {
                    pub account: String,
                    pub oauth_source: String,
                    pub nickname: String,
                    pub avatar: String,
                    pub pub_settings: String,
                    pub ext: String,
                }

                let inner_params = InnerUserCreateParams {
                    account: github_user_info.account.to_owned(),
                    oauth_source: "github".to_owned(),
                    nickname: github_user_info.account.to_owned(),
                    avatar: "".to_owned(),
                    pub_settings: "".to_owned(),
                    ext: "".to_owned(),
                };
                let users: Vec<GutpUser> = make_post("/v1/user/create", &inner_params)
                    .await
                    .unwrap_or(vec![]);
                if let Some(user) = users.into_iter().next() {
                    // registerd successfully
                    login_user_with_account(redis_conn, &account)
                        .await
                        .into_response()
                } else {
                    // redirect to the error page
                    let action = format!("Register user: {}", &account);
                    let err_info = "Unknown";
                    redirect_to_error_page(&action, err_info).into_response()
                }
            }
        } else {
            // error on getting github user info
            let action = format!("Get user info from github");
            let err_info = "Failed to get response from github";
            redirect_to_error_page(&action, err_info).into_response()
        }
    } else {
        // error on getting github access token
        let action = format!("Get access token from github");
        let err_info = "Failed to request access token from github";
        redirect_to_error_page(&action, err_info).into_response()
    }
}

#[derive(Deserialize)]
struct GithubCredentials {
    access_token: String,
}

async fn get_github_token(
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<GithubCredentials, reqwest::Error> {
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("accept", "json"),
    ];

    let client = reqwest::Client::new();
    let res = client
        .post("https://github.com/login/oauth/access_token")
        .form(&params)
        .send()
        .await?;

    println!("in get_github_token, {:?}", res);

    let res: GithubCredentials = res.json().await?;

    Ok(res)
}

#[derive(Deserialize)]
struct GithubUserInfo {
    account: String,
    // nickname: String,
}

async fn get_github_user_info(access_token: &str) -> Result<GithubUserInfo, reqwest::Error> {
    let params = [("access_token", access_token)];

    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/user")
        .query(&params)
        .header("User-Agent", "gutp-discux")
        .send()
        .await?;

    println!("in get_github_user_info, {:?}", res);

    let user_info: GithubUserInfo = res.json().await?;

    Ok(user_info)
}

async fn login_user_with_account(conn: redis::aio::Connection, account: &str) -> impl IntoResponse {
    // first, set session key in server cache
    let cookiestr = set_session(conn, account).await;

    let cookie_key = format!("{}_sid", &crate::APPID);
    let cookie = Cookie::build(&cookie_key, &cookiestr)
        // .domain("/")
        .path("/")
        //.secure(true)
        .max_age(cookie::time::Duration::seconds(TTL as i64))
        .http_only(true)
        .finish();

    (
        [(header::SET_COOKIE, cookie.to_string())],
        Redirect::to("/"),
    )
}

pub async fn set_session(mut conn: redis::aio::Connection, account: &str) -> String {
    let x = rand::random::<[u8; 32]>();
    let cookie = sha256::digest(&x);
    let cookie_key = format!("{}_session:{}", &crate::APPID, cookie);
    let _: Result<(), redis::RedisError> = conn.set(&cookie_key, account).await;
    let _: Result<(), redis::RedisError> = conn.expire(&cookie, TTL).await;

    cookie
}

pub async fn clear_session(mut conn: redis::aio::Connection, session_id: &str) {
    let session_key = format!("{}_session:{}", &crate::APPID, session_id);
    let _: Result<(), redis::RedisError> = conn.del(&session_key).await;
}

pub async fn signout(
    State(app_state): State<AppState>,
    Extension(logged_user_id): Extension<LoggedUserId>,
    cookie_jar: CookieJar,
) -> impl IntoResponse {
    if logged_user_id.is_none() {
        let action = format!("Not logged in");
        let err_info = "Need login firstly to get proper permission.";
        return redirect_to_error_page(&action, err_info).into_response();
    }

    let mut redis_conn = app_state.rclient.get_async_connection().await.unwrap();
    let cookie_key = format!("{}_sid", &crate::APPID);
    if let Some(session_id) = cookie_jar.get(&cookie_key) {
        clear_session(redis_conn, &session_id.to_string());
        (
            cookie_jar.remove(Cookie::named(cookie_key)),
            Redirect::to("/"),
        )
            .into_response()
    } else {
        let redirect_uri = format!("/login");
        Redirect::to(&redirect_uri).into_response()
    }
}
