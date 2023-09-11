use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use quick_xml::Writer;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::Display;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::path::PathBuf;
use zip::read::ZipFile;
use zip::ZipArchive;

struct EpubSandbox {
    path: PathBuf,
    epub: Epub,
}

type SpineItem = String;
type MimeType = String;
type EpubArchive = ZipArchive<BufReader<std::fs::File>>;
type Resource = HashMap<String, (PathBuf, MimeType)>;
pub struct Epub {
    file: EpubArchive,
    spine: Vec<SpineItem>,
    resources: Vec<Resource>,
    metadata: HashMap<String, String>,
    cover_id: Option<String>,
    path: PathBuf,
}

impl Epub {
    pub fn new(path: &PathBuf) -> Result<Self, EpubError> {
        let file = std::fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let zip = ZipArchive::new(reader)?;

        let mut epub = Epub {
            spine: Vec::new(),
            file: zip,
            resources: Vec::new(),
            metadata: HashMap::new(),
            cover_id: None,
            path: path.clone(),
        };

        epub.populate_epub()?;
        Ok(epub)
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    fn populate_epub(&mut self) -> Result<(), EpubError> {
        let content = self.file.by_name("content.opf")?;
        let content_reader = BufReader::new(content);
        let mut reader = Reader::from_reader(content_reader);

        let mut buff = Vec::new();

        loop {
            match reader.read_event_into(&mut buff) {
                Ok(Event::Start(ref e)) => {
                    if let b"spine" = e.name().as_ref() {
                        read_spine(reader.borrow_mut(), &mut self.spine)?;
                    } else if let b"manifest" = e.name().as_ref() {
                        read_manifest(reader.borrow_mut(), &mut self.resources)?;
                    } else if let b"metadata" = e.name().as_ref() {
                        read_metadata(reader.borrow_mut(), &mut self.metadata, &mut self.cover_id)?;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
        }

        Ok(())
    }

    pub fn get_res_by_path(&mut self, path: &PathBuf) -> Option<Vec<u8>> {
        let path = match path.to_str() {
            Some(path) => path,
            None => return None,
        };
        let mut zip_file = match self.file.by_name(path) {
            Ok(zip) => zip,

            Err(_) => return None,
        };

        let mut buff = Vec::new();
        zip_file.read_to_end(&mut buff);

        if path.ends_with(".css") {
            let styles = std::str::from_utf8(&buff)
                .unwrap()
                .replace("-webkit-", "")
                .replace("-epub-", "");
            return Some(styles.into_bytes());
        }

        Some(buff)
    }

    pub fn get_res(&self, id: String) -> Option<Resource> {
        self.resources
            .iter()
            .find(|resource| resource.contains_key(&id))
            .map(|resource| resource.clone())
    }

    pub fn get_page(&mut self, index: usize, asset_id: &String) -> Option<(Vec<u8>, String)> {
        let id = self.spine.get(index)?;
        let resource = self.get_res(id.clone())?;
        let (path, mime_type) = resource.get(id)?;
        // Need to do some modification to the html file
        // Need to add a base tag to the head
        let file = self.get_res_by_path(&path)?;
        let base = format!(
            "http://localhost:8273/api/v1/book/{}/resource/{}",
            asset_id,
            path.to_str()?
        );
        let file = add_base(file, &base).unwrap();

        Some((file, mime_type.clone()))
    }

    pub fn get_cover(&mut self) -> Option<(Vec<u8>, String)> {
        let id = self.cover_id.as_ref()?;
        let resource = self.get_res(id.clone())?;
        let (path, mime_type) = resource.get(id)?;
        let file = self.get_res_by_path(&path)?;
        Some((file, mime_type.clone()))
    }

    pub fn get_metadata(&self, identifier: &str) -> Option<&String> {
        self.metadata.get(identifier)
    }
}

fn add_base(buff: Vec<u8>, base_url: &str) -> Result<Vec<u8>, EpubError> {
    let str = match std::str::from_utf8(&buff) {
        Ok(str) => str,
        Err(e) => Err(EpubError::Convertion)?,
    };

    let mut reader = Reader::from_str(str);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"head" => {
                assert!(writer.write_event(Event::Start(e)).is_ok());

                let mut base_el = BytesStart::new("base");
                base_el.push_attribute(("href", base_url));

                assert!(writer.write_event(Event::Start(base_el)).is_ok());
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                assert!(writer.write_event(e).is_ok())
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }
    Ok(writer.into_inner().into_inner())
}
fn read_spine(
    reader: &mut Reader<BufReader<ZipFile<'_>>>,
    spine: &mut Vec<SpineItem>,
) -> Result<(), EpubError> {
    let mut buff = Vec::new();

    loop {
        match reader.read_event_into(&mut buff) {
            Ok(Event::Empty(ref e)) => {
                if let b"itemref" = e.name().as_ref() {
                    let attributes = e.attributes();
                    let id_attr = attributes
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"idref");

                    if let Some(id_attr) = id_attr {
                        let id = id_attr.decode_and_unescape_value(&reader)?.to_string();
                        spine.push(id)
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if let b"spine" = e.name().as_ref() {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                Err(EpubError::from(e))?;
            }
            _ => (),
        }
    }

    Ok(())
}

fn read_manifest(
    reader: &mut Reader<BufReader<ZipFile<'_>>>,
    resources: &mut Vec<Resource>,
) -> Result<(), EpubError> {
    let mut buff = Vec::new();

    loop {
        match reader.read_event_into(&mut buff) {
            Ok(Event::Empty(ref e)) => {
                if b"item" != e.name().as_ref() {
                    continue;
                }

                let mut id = String::new();
                let mut href = PathBuf::new();
                let mut media_type = String::new();

                for attr in e.attributes().filter_map(|a| a.ok()) {
                    match attr.key.as_ref() {
                        b"href" => href.push(attr.decode_and_unescape_value(&reader)?.as_ref()),
                        b"media-type" => {
                            media_type.push_str(attr.decode_and_unescape_value(&reader)?.as_ref())
                        }
                        b"id" => id.push_str(attr.decode_and_unescape_value(&reader)?.as_ref()),
                        _ => {}
                    }
                }

                let mut resource = Resource::new();
                resource.insert(id, (href, media_type));
                resources.push(resource);
            }
            Ok(Event::End(ref e)) => {
                if let b"manifest" = e.name().as_ref() {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                Err(EpubError::from(e))?;
            }
            _ => (),
        }
    }

    Ok(())
}

fn read_metadata(
    reader: &mut Reader<BufReader<ZipFile<'_>>>,
    metadata: &mut HashMap<String, String>,
    cover_id: &mut Option<String>,
) -> Result<(), EpubError> {
    let mut buff = Vec::new();

    loop {
        match reader.read_event_into(&mut buff) {
            // Get the cover_id
            Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"meta" => {
                    let mut name = String::new();
                    let mut content = String::new();

                    for attr in e.attributes().filter_map(|a| a.ok()) {
                        match attr.key.as_ref() {
                            b"name" => {
                                name.push_str(attr.decode_and_unescape_value(&reader)?.as_ref())
                            }
                            b"content" => {
                                content.push_str(attr.decode_and_unescape_value(&reader)?.as_ref())
                            }
                            _ => {}
                        }
                    }
                    if name == "cover" {
                        *cover_id = Some(content);
                    }
                }
                // e if e.starts_with("dc:".as_bytes()) => {
                //     let name = std::str::from_utf8(&e[2..])
                //         .map_err(|_| EpubError::Convertion)?
                //         .to_string();
                //     let mut data = String::new();

                //     match reader.read_event_into(&mut buff) {
                //         Ok(Event::Text(e)) => data.push_str(e.unescape()?.into_owned().as_ref()),
                //         _ => continue,
                //     }

                //     metadata.insert(name, data);
                // }
                _ => continue,
            },
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                e if e.starts_with("dc:".as_bytes()) => {
                    let name = std::str::from_utf8(&e[3..])
                        .map_err(|_| EpubError::Convertion)?
                        .to_string();
                    let mut data = String::new();

                    match reader.read_event_into(&mut buff) {
                        Ok(Event::Text(e)) => data.push_str(e.unescape()?.into_owned().as_ref()),
                        _ => continue,
                    }

                    metadata.insert(name, data);
                }

                _ => continue,
            },

            Ok(Event::End(ref e)) => {
                if let b"metadata" = e.name().as_ref() {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                Err(EpubError::from(e))?;
            }
            _ => (),
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum EpubError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Xml(quick_xml::Error),
    Convertion,
}

impl From<std::io::Error> for EpubError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<zip::result::ZipError> for EpubError {
    fn from(error: zip::result::ZipError) -> Self {
        Self::Zip(error)
    }
}

impl From<quick_xml::Error> for EpubError {
    fn from(error: quick_xml::Error) -> Self {
        Self::Zip(zip::result::ZipError::FileNotFound)
    }
}

impl Display for EpubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpubError::Io(e) => write!(f, "{}", e),
            EpubError::Zip(e) => write!(f, "{}", e),
            EpubError::Xml(e) => write!(f, "{}", e),
            EpubError::Convertion => write!(f, "Could not convert to utf8"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

}
