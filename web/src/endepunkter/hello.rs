#[utoipa::path(
    get,
    path = "/hello",
    responses(
        (status = 200, content_type = "text/plain")
    )
)]
pub async fn root() -> &'static str {
    "Hello, World!"
}
