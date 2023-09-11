use crate::Konfig;
use axum::routing::{delete, get, post};
use axum::Router;
use std::thread;
use tokio::select;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::Modify;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use web::endepunkter::{auth, books, hello, images, library};
struct Terminal {
    kanal: tokio::sync::mpsc::Receiver<String>,
}

fn terminal() -> Terminal {
    let (sender, mottaker) = tokio::sync::mpsc::channel(100);

    let terminal = Terminal { kanal: mottaker };

    thread::spawn(move || loop {
        let mut melding = String::new();
        std::io::stdin().read_line(&mut melding).unwrap();
        sender.blocking_send(melding).unwrap();
    });

    return terminal;
}

async fn hent_termianl_meldinger(mut terminal: Terminal) {
    loop {
        let melding = terminal.kanal.recv().await.unwrap();

        println!("Fikk melding: {:?}", melding);

        if melding == "stopp\n" {
            break;
        }
    }
}

pub async fn kjerne_pakker(konfig: Konfig) {
    #[derive(OpenApi)]
    #[openapi(
        security(
            ("app_bearer" = [])
        ),
        paths(
            web::endepunkter::hello::root,
            web::endepunkter::auth::register,
            web::endepunkter::auth::login,
            web::endepunkter::library::add_library,
            web::endepunkter::library::get_libraries,
            web::endepunkter::library::delete_library,
            web::endepunkter::library::scan_library,
            web::endepunkter::books::get_books,
            web::endepunkter::books::get_book,
            web::endepunkter::books::get_book_page,
            web::endepunkter::books::get_book_resource,
            web::endepunkter::images::get_cover,
        ),
        components(
            schemas(
                database::users::Register,
                database::users::Login,
                database::library::InsertableLibrary,
            )
        ),
        tags(
            (name = "hello", description = "Hello world!")
        ),
    )]
    struct ApiDoc;

    let schema = utoipa::openapi::ComponentsBuilder::new()
        .security_scheme(
            "app_bearer",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
        )
        .build();

    let mut open_api_schema = ApiDoc::openapi();

    open_api_schema.components = Some(schema);

    let mut open_api = ApiDoc::openapi();

    // Stupid hack to get around the fact that the openapi crate doesn't support adding security through the macro
    open_api.merge(open_api_schema);

    let pool = database::create_pool(&konfig.database_path).await.unwrap();

    println!("Bruker web-ui fra {:?}", konfig.web_ui_path);
    let web_ui_mappe = ServeDir::new(&konfig.web_ui_path)
        .not_found_service(ServeFile::new(&konfig.web_ui_path.join("index.html")));

    let ruter = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", open_api))
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/hello", get(hello::root))
        .route("/api/v1/library", post(library::add_library))
        .route("/api/v1/library", get(library::get_libraries))
        .route("/api/v1/library/:id", delete(library::delete_library))
        .route("/api/v1/library/scan", post(library::scan_library))
        .route("/api/v1/book", get(books::get_books))
        .route("/api/v1/book/:id", get(books::get_book))
        .route("/api/v1/book/:id/page/:page_num", get(books::get_book_page))
        .route(
            "/api/v1/book/:id/resource/*path",
            get(books::get_book_resource),
        )
        .route("/api/v1/images/covers/:id", get(images::get_cover))
        .nest_service("/", web_ui_mappe.clone())
        .fallback_service(web_ui_mappe)
        .with_state(pool.clone());

    let web_fremtid = web::serve(konfig.server_address, ruter);

    let terminal = terminal();

    let terminal_fremtid = hent_termianl_meldinger(terminal);

    println!("Lytter på nettverkssocket på {}", konfig.server_address);

    select! {
        _ = web_fremtid => {},
        _ = terminal_fremtid => {
            println!("Fikk stopp-melding fra terminal, avslutter...");
            pool.close().await;
            println!("Stengte databasetilkobling, avsluttning ferdig.");
        },
    }
}
