use std::net::SocketAddr;
use std::path::PathBuf;

pub struct Konfig {
    pub server_address: SocketAddr,
    pub database_path: String,
    pub web_ui_path: PathBuf,
    pub installation_path: PathBuf,
}

impl Konfig {
    pub fn new() -> Konfig {
        let installation_path = std::env::var("INSTALLATION_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap());

        let database_path = std::env::var("DATABASE_URL")
            .map(PathBuf::from)
            .unwrap_or_else(|_| installation_path.join("database.db"))
            .to_str()
            .unwrap()
            .to_string();

        let database_path = format!("sqlite://{}", database_path);

        let web_ui_path = std::env::var("WEB_UI_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| installation_path.join("web-ui/dist"));

        let server_address = "127.0.0.1:8273".parse().unwrap();

        Konfig {
            server_address,
            database_path,
            web_ui_path,
            installation_path,
        }
    }
}
