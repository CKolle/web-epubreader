use crate::epub_sandbox::Epub;
use crate::epub_sandbox::EpubError;
use database::assets::Asset;
use database::assets::InsertableAsset;
use database::library::Collection;
use database::library::{Book, InsertableBook, InsertableCollection, Library};
use futures::stream;
use futures::StreamExt;
use sqlx::pool;
use sqlx::sqlite::Sqlite;
use sqlx::Pool;
use std::f32::consts::E;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::io::AsyncWriteExt;

const MIME_TYPE_EXTENSIONS_MAP: [(&str, &str); 2] = [("image/jpeg", "jpg"), ("image/png", "png")];
const ALLOWED_COVER_MIME_TYPES: [&str; 2] = ["image/jpeg", "image/png"];

static COVER_PATH: OnceLock<String> = OnceLock::new();

pub fn set_metadata_path(path: String) {
    let path = COVER_PATH.get_or_init(|| path);
    println!("Metadata path set to {}", path);
}

#[derive(Debug)]
struct CollectionDicovery {
    // The name of the containing folder
    name: String,
    // Path to the containing folder
    path: PathBuf,
    book_paths: Vec<PathBuf>,
}

#[derive(Default)]
struct LibScanner {
    book_paths: Vec<PathBuf>,
    collections: Vec<CollectionDicovery>,
}

impl LibScanner {}

fn discover_books(folder_path: impl Into<PathBuf>) -> Result<(Vec<PathBuf>), ScanError> {
    let folder_path = folder_path.into();
    let book_paths: Vec<PathBuf> = fs::read_dir(folder_path)?
        .filter_map(|entry_res| entry_res.ok())
        .filter(|entry| match entry.path().extension() {
            Some(extension) => extension.to_str() == Some("epub"),
            None => false,
        })
        .map(|entry| entry.path())
        .collect();

    return Ok(book_paths);
}
fn discover_collections(
    folder_path: impl Into<PathBuf>,
) -> Result<Vec<CollectionDicovery>, ScanError> {
    let folder_path = folder_path.into();

    let collections: Vec<CollectionDicovery> = fs::read_dir(folder_path)?
        .filter_map(|entry_res| entry_res.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().into_string();
            if name.is_err() {
                return None;
            }
            let name = name.ok()?;
            let path = entry.path();
            let book_paths = discover_books(&path).ok()?;
            Some(CollectionDicovery {
                name,
                path,
                book_paths,
            })
        })
        .collect();

    return Ok(collections);
}

pub async fn extract_cover(epub: &mut Epub) -> Result<Option<InsertableAsset>, ScanError> {
    let cover = epub.get_cover();
    if cover.is_none() {
        return Ok(None);
    }
    let cover = cover.unwrap();
    let mime_type = cover.1;
    if !ALLOWED_COVER_MIME_TYPES.contains(&mime_type.as_str()) {
        return Ok(None);
    }
    let extension = MIME_TYPE_EXTENSIONS_MAP
        .iter()
        .find(|(mime, _)| mime == &mime_type)
        .map(|(_, ext)| ext)
        .ok_or(ScanError::InvalidCoverMimeType(mime_type.clone()))?;
    let cover_name = uuid::Uuid::new_v4().to_string();
    let cover_path = format!("{}/{}.{}", COVER_PATH.get().unwrap(), cover_name, extension);
    let mut file = tokio::fs::File::create(&cover_path).await?;
    file.write_all(&cover.0).await?;
    let asset = InsertableAsset {
        local_path: cover_path,
        file_extension: Some(mime_type.clone()),
    };
    Ok(Some(asset))
}

fn scan_collection(
    collection: &CollectionDicovery,
    library_id: i32,
) -> Result<InsertableCollection, ScanError> {
    Ok(InsertableCollection {
        name: collection.name.clone(),
        library_id,
        path: collection
            .path
            .to_str()
            .ok_or(ScanError::InvalidPath(
                "Collection path is not valid unicode".to_string(),
            ))?
            .to_string(),
    })
}

async fn scan_book(
    epub: &mut Epub,
    library_id: i32,
    collection_id: Option<i32>,
) -> Result<(InsertableBook, Option<InsertableAsset>), ScanError> {
    let title = epub
        .get_metadata("title")
        .ok_or(ScanError::EpubError("No title".to_string()))?;
    let path = epub.get_path();
    let path_str = path.to_str().ok_or(ScanError::InvalidPath(
        "Path is not valid unicode".to_string(),
    ))?;
    let insertable_book = InsertableBook {
        path: path_str.to_string(),
        name: title.to_string(),
        library_id: library_id,
        collection_id,
        primary_cover: None,
    };

    let cover = extract_cover(epub).await?;

    return Ok((insertable_book, cover));
}

pub async fn clean_up(path: impl Into<PathBuf>, pool: &Pool<Sqlite>) -> Result<(), ScanError> {
    let path: PathBuf = path.into();
    let path_str = path.to_str().ok_or(ScanError::InvalidPath(
        "Path is not valid unicode".to_string(),
    ))?;

    let library = match Library::get_by_path(path_str, pool).await? {
        Some(library) => library,
        None => return Ok(()),
    };

    let books = Book::get_books_by_library(library.id, pool).await?;

    let cover_assets = stream::iter(&books).filter_map(|book| async move {
        let assets = Asset::get_cover_assets_book(book.id, &pool).await.ok();
        assets
    });

    let book_covers = cover_assets.collect::<Vec<Vec<Asset>>>().await;

    let collections = Collection::get_by_libary(library.id, &pool).await?;

    for cover_asset in book_covers {
        println!("Deleting cover");
        for asset in cover_asset {
            let local_path = asset.local_path.clone();
            println!("Deleting {}", local_path);
            tokio::fs::remove_file(local_path).await?;
            asset.delete_self(&pool).await?;
        }
    }

    for book in books {
        let assets = Asset::get_asset(&book.asset_id, pool)
            .await?
            .ok_or(ScanError::AssetNotFound)?;
        assets.delete_self(&pool).await?;
    }

    for (i, c) in collections.iter().enumerate() {
        if i == 0 {
            continue;
        }
        c.delete_self(&pool).await?;
    }
    Ok(())
}

// Responsible for initializing an epub library scan
pub async fn force_scan(
    path: impl Into<PathBuf>,
    library_id: i32,
    pool: &Pool<Sqlite>,
) -> Result<(), ScanError> {
    let path = path.into();

    // clean_up(&path, &pool).await?;

    // 1. Discover collections
    let collections_discov = discover_collections(&path)?;
    // 2. Discover books
    let root_books = discover_books(&path)?;
    // 3. Scan collections
    let insertable_collections: Vec<InsertableCollection> = collections_discov
        .iter()
        .map(|collection| scan_collection(collection, library_id))
        .filter_map(|collection_res| collection_res.ok())
        .collect();

    // 4. Insert collections into database
    let collections = stream::iter(insertable_collections)
        .filter_map(|collection| async move {
            let collection = collection.insert(pool).await.ok()?;
            Some(collection)
        })
        .collect::<Vec<Collection>>()
        .await;

    // 5. Scan books
    let insertable_books: Vec<(InsertableBook, Option<InsertableAsset>)> = stream::iter(root_books)
        .filter_map(|book_path| async move {
            let mut epub = Epub::new(&book_path).unwrap();
            scan_book(&mut epub, library_id, None).await.ok()
        })
        .collect()
        .await;
    // 6. Insert books into database

    for (mut book, cover) in insertable_books {
        if let Some(cover) = cover {
            let cover_asset = cover.insert(&pool).await?;
            book.add_cover(cover_asset.id.clone());
            let book = book.insert(pool).await?;
            cover_asset.into_book_cover(book.id, pool).await?;
        } else {
            book.insert(pool).await?;
        }

    }

    // 7. Scan books in collections
    for collection in collections {
        let collection_path = PathBuf::from(&collection.path);
        let collection_books = discover_books(&collection_path)?;
        let insertable_books: Vec<(InsertableBook, Option<InsertableAsset>)> =
            stream::iter(collection_books)
                .filter_map(|book_path| async move {
                    let mut epub = Epub::new(&book_path).unwrap();
                    scan_book(&mut epub, library_id, Some(collection.id))
                        .await
                        .ok()
                })
                .collect()
                .await;

        for (mut book, cover) in insertable_books {
            if let Some(cover) = cover {
                let cover_asset = cover.insert(&pool).await?;
                book.add_cover(cover_asset.id.clone());
                let book = book.insert(pool).await?;
                cover_asset.into_book_cover(book.id, pool).await?;
            } else {
                book.insert(pool).await?;
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum ScanError {
    InvalidPath(String),
    DatabaseError,
    EpubError(String),
    MetadataNotSet(String),
    InvalidCoverMimeType(String),
    AssetNotFound,
}

impl From<std::io::Error> for ScanError {
    fn from(error: std::io::Error) -> Self {
        ScanError::InvalidPath(error.to_string())
    }
}

impl From<EpubError> for ScanError {
    fn from(error: EpubError) -> Self {
        ScanError::EpubError(error.to_string())
    }
}

impl From<sqlx::Error> for ScanError {
    fn from(error: sqlx::Error) -> Self {
        ScanError::DatabaseError
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

}
