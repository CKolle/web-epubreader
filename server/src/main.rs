mod kjerne;
mod konfig;
mod storer;

use konfig::Konfig;
use scanner::set_metadata_path;

#[tokio::main]
async fn main() {
    let konfig = Konfig::new();
    set_metadata_path("./covers".to_owned());
    scanner::scanner::set_metadata_path("./covers".to_owned());

    kjerne::kjerne_pakker(konfig).await;
}
