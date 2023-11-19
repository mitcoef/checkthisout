use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Html;
use axum::response::IntoResponse;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::fs;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use sea_orm::DbErr;

mod database;
mod rest;
mod utils;
mod traits;

use axum::{
    routing::{get, patch},
    Router,
};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let state = rest::app_state::init_state().await?;

    let router: Router = Router::new()
        .route("/craftsmen", get(rest::get_craftsmen::handler))
        .route("/craftsmen/:id", patch(rest::patch_craftsmen::handler))
        .fallback_service(get(|req: Request<Body>| async move {
            let res = ServeDir::new("./dist").oneshot(req).await.unwrap(); // serve dir is infallible
            let status = res.status();
            match status {
                // If we don't find a file corresponding to the path we serve index.html.
                // If you want to serve a 404 status code instead you can add a route check as shown in
                // https://github.com/rksm/axum-yew-setup/commit/a48abfc8a2947b226cc47cbb3001c8a68a0bb25e
                StatusCode::NOT_FOUND => {
                    let index_path = PathBuf::from("./dist/index.html");
                    fs::read_to_string(index_path.clone())
                        .await
                        .map(|index_content| (StatusCode::OK, Html(index_content)).into_response())
                        .unwrap_or_else(|_| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!(
                                    "{}{}",
                                    std::env::current_dir().unwrap().display(),
                                    " not found"
                                ),
                            )
                                .into_response()
                        })
                }

                // path was found as a file in the static dir
                _ => res.into_response(),
            }
        }))
        .with_state(state);

    let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 1339));
    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
