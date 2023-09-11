use database::assets::{Asset, InsertableAsset};
use database::library::{Book, InsertableBook, Library};
use sqlx::Pool;
use sqlx::Sqlite;
use std::cell::OnceCell;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;
pub mod epub_sandbox;
pub mod scanner;

const FILE_EXTENSIONS: [&str; 1] = ["epub"];

const ALLOWED_COVER_MIME_TYPES: [&str; 2] = ["image/jpeg", "image/png"];

const MIME_TYPE_EXTENSIONS_MAP: [(&str, &str); 2] = [("image/jpeg", "jpg"), ("image/png", "png")];

// Need to make this configurable, but for now it's fine
const TEMPORARY_COVER_DIR: &str = "./covers";
static COVER_PATH: OnceLock<String> = OnceLock::new();

pub fn set_metadata_path(path: String) {
    let path = COVER_PATH.get_or_init(|| path);
    println!("Metadata path set to {}", path);
}

struct BookCover {
    id: i32,
    data: (Vec<u8>, String),
}

#[derive(Clone)]
struct BookPath(PathBuf);

#[derive(Clone)]
struct Collection {
    name: String,
    books: Vec<BookPath>,
}

pub struct LibraryScanner {
    path: PathBuf,
    books: Vec<BookPath>,
    collections: Vec<Collection>,
}

// impl LibraryScanner {
//     pub fn new(path: impl Into<PathBuf>) -> Self {
//         Self {
//             path: path.into(),
//             books: Vec::new(),
//             collections: Vec::new(),
//         }
//     }

//     async fn scan_book(&mut self, path: PathBuf) -> Result<(), LibraryScannerError> {
//         let extension = path.extension().unwrap().to_str().unwrap();
//         if FILE_EXTENSIONS.contains(&extension) {
//             let book = BookPath(path);
//             self.books.push(book);
//         }
//         Ok(())
//     }

//     async fn scan_one_book_level(&mut self, path: PathBuf) -> Result<(), LibraryScannerError> {
//         let entries = tokio::fs::read_dir(path).await;

//         let mut entries = match entries {
//             Ok(entries) => entries,
//             Err(_) => return Err(LibraryScannerError::InvalidPath),
//         };

//         while let Some(entry) = entries.next_entry().await? {
//             if entry.metadata().await?.is_file() {
//                 self.scan_book(entry.path()).await?;
//             }
//         }

//         Ok(())
//     }

//     pub async fn scan(&mut self) -> Result<(), LibraryScannerError> {
//         let entries = tokio::fs::read_dir(&self.path).await;

//         let mut entries = match entries {
//             Ok(entries) => entries,
//             Err(_) => return Err(LibraryScannerError::InvalidPath),
//         };

//         while let Some(entry) = entries.next_entry().await? {
//             if entry.metadata().await?.is_dir() {
//                 let collection = Collection {
//                     name: entry.file_name().to_string_lossy().to_string(),
//                     books: Vec::new(),
//                 };
//                 self.collections.push(collection);
//                 self.scan_one_book_level(entry.path()).await?;
//             } else {
//                 self.scan_book(entry.path()).await?;
//             }
//         }
//         Ok(())
//     }

//     pub async fn store_to_db(
//         &self,
//         library_id: i32,
//         pool: &Pool<Sqlite>,
//     ) -> Result<(), LibraryScannerError> {
//         let existing_cover_assets = Asset::get_cover_assets_library(library_id, pool)
//             .await
//             .unwrap();

//         for existing_cover_asset in existing_cover_assets {
//             tokio::fs::remove_file(existing_cover_asset.local_path)
//                 .await
//                 .unwrap();
//         }

//         // Necessary to delete the library contents before inserting new ones
//         Library::delete_library_contents(library_id, pool)
//             .await
//             .unwrap();

//         // This stores the non-collection books
//         let insertable_books = self
//             .books
//             .iter()
//             .map(|book| {
//                 let epub_doc = EpubDoc::new(&book.0).unwrap();
//                 let title = epub_doc.mdata("title").unwrap_or("Unknown".into());

//                 InsertableBook {
//                     library_id,
//                     collection_id: None,
//                     name: title,
//                     path: book.0.to_string_lossy().to_string(),
//                 }
//             })
//             .collect::<Vec<_>>();

//         let book_tasks = insertable_books
//             .into_iter()
//             .map(|insertable_book| {
//                 let pool = pool.clone();
//                 let join_handle = tokio::spawn(async move {
//                     return insertable_book.insert(&pool).await.unwrap();
//                 });
//                 join_handle
//             })
//             .collect::<Vec<_>>();

//         let books: Vec<Book> = futures::future::join_all(book_tasks)
//             .await
//             .into_iter()
//             .map(|book_result| book_result.unwrap())
//             .collect();

//         let assets_tasks: Vec<JoinHandle<()>> = books
//             .into_iter()
//             .map(|book| {
//                 let pool = pool.clone();
//                 let join_handle = tokio::spawn(async move {
//                     let book_path = Asset::get_asset(&book.asset_id, &pool)
//                         .await
//                         .unwrap()
//                         .unwrap()
//                         .local_path;

//                     let mut epub_doc = epub_sandbox::Epub::new(&book_path.into()).unwrap();
//                     let cover = epub_doc.get_cover().unwrap_or((Vec::new(), "".into()));
//                     let uuid = uuid::Uuid::new_v4();
//                     let extension = MIME_TYPE_EXTENSIONS_MAP
//                         .iter()
//                         .find(|(mime_type, _)| *mime_type == cover.1)
//                         .unwrap()
//                         .1;

//                     let cover_path =
//                         format!("{}/{}.{}", COVER_PATH.get().unwrap(), uuid, extension);

//                     let insertable_asset = InsertableAsset {
//                         local_path: cover_path,
//                         file_extension: Some(cover.1),
//                     };
//                     let mut file = tokio::fs::File::create(&insertable_asset.local_path)
//                         .await
//                         .unwrap();
//                     file.write_all(&cover.0).await.unwrap();
//                     let asset = insertable_asset.insert(&pool).await.unwrap();
//                     asset.into_book_cover(book.id, &pool).await.unwrap();
//                 });
//                 return join_handle;
//             })
//             .collect::<Vec<_>>();

//         futures::future::join_all(assets_tasks).await;

//         Ok(())
//     }

//     fn get_books(&self) -> &Vec<BookPath> {
//         &self.books
//     }

//     fn get_collections(&self) -> &Vec<Collection> {
//         &self.collections
//     }
// }

// pub enum LibraryScannerError {
//     Io(std::io::Error),
//     InvalidPath,
//     DatabaseError,
// }

// impl From<std::io::Error> for LibraryScannerError {
//     fn from(error: std::io::Error) -> Self {
//         Self::Io(error)
//     }
// }
