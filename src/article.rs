use gutp_types::GutpPost;
use gutp_types::GutpTag;
use gutp_types::{GutpComment, GutpExtobj};

#[derive(Template)]
#[template(path = "articles.html")]
struct LatestArticlesTemplate {
    tags: Vec<GutpTag>,
    posts: Vec<GutpPost>,
    extobjs: Vec<GutpExtobj>,
}

pub async fn latest_articles(
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

pub async fn article(State(client): State<Client>, RawQuery(query): RawQuery) -> impl IntoResponse {
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
