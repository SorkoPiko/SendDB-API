use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::endpoint::common;
use crate::model::info::SearchResult;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SearchQuery {
    pub search: String,
    pub limit: i64,
}


#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchResponse {
    pub total: i32,
    pub results: Vec<SearchResult>
}

#[utoipa::path(summary = "Search levels and creators", responses(
    (status = OK, description = "Search levels and creators", body = SearchResponse)
))]
#[post("")]
pub async fn search(
    app_state: web::Data<AppState>,
    query: web::Json<SearchQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    if query.limit <= 0 {
        return Ok(HttpResponse::Ok().json(SearchResponse { total: 0, results: vec![] }));
    } else if query.limit > 50 {
        return Err(common::bad_request("Too many results requested"));
    }

    let response = {
        let db = app_state.database.lock().await;
        db.search(&query).await
            .map_err(|e| {
                log::error!("{:?}", e);
                common::internal_server_error("Database error")
            })?
    };

    Ok(HttpResponse::Ok().json(response))
}