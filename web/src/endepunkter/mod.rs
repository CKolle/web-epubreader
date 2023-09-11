pub mod auth;
pub mod books;
pub mod hello;
pub mod images;
pub mod library;
use tokio::sync::OnceCell;

static METADATA_PATH: OnceCell<String> = OnceCell::const_new();

pub fn set_metadata_path(path: String) {
    METADATA_PATH.set(path).unwrap();
}
