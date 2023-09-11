use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Weak;
use std::sync::{Arc, RwLock};

/* 
    Old Scanner code will be removed 
*/

fn extract_styles(doc: &mut EpubDoc<BufReader<File>>, style_path: String) -> Vec<u8> {
    doc.set_current_page(0);
    let style_path = style_path.replace("../", "");
    let styles = match doc.get_resource_by_path(&style_path) {
        Some(styles) => styles,
        None => {
            println!("Could not find styles by path {}", &style_path);
            return Vec::new();
        }
    };

    // Next remove all -webkit- prefixes and -epub- prefixes
    let styles = std::str::from_utf8(&styles)
        .unwrap()
        .replace("-webkit-", "")
        .replace("-epub-", "");
    let styles_with_newline = format!("\n{}", styles);
    styles_with_newline.into_bytes()
}

pub fn process_page(file_path: PathBuf, page_num: usize, book_id: i32) -> String {
    let mut doc = EpubDoc::new(&file_path).unwrap();
    doc.set_current_page(page_num);
    let mut converter = EpubXmlHtmlConverter::new(doc, book_id);
    converter.convert();
    converter.generate_html()
}

#[derive(Debug, PartialEq)]
enum XmlToken {
    ProcessingInstruction,

    TagOpen(String),
    TagClose(String),

    TagSelfClose,
    TagEnd,

    Assert,
    AttributeName(String),
    AttributeValue(String),

    Comment(String),

    Data(String),

    Eof,
}

impl Display for XmlToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmlToken::ProcessingInstruction => write!(f, "ProcessingInstruction"),
            XmlToken::TagOpen(tag_name) => write!(f, "TagOpen({})", tag_name),
            XmlToken::TagClose(tag_name) => write!(f, "TagClose({})", tag_name),
            XmlToken::TagSelfClose => write!(f, "TagSelfClose"),
            XmlToken::TagEnd => write!(f, "TagEnd"),
            XmlToken::Assert => write!(f, "Assert"),
            XmlToken::AttributeName(attribute_name) => {
                write!(f, "AttributeName({})", attribute_name)
            }
            XmlToken::AttributeValue(attribute_value) => {
                write!(f, "AttributeValue({})", attribute_value)
            }
            XmlToken::Comment(comment) => write!(f, "Comment({})", comment),
            XmlToken::Data(data) => write!(f, "Data({})", data),
            XmlToken::Eof => write!(f, "Eof"),
        }
    }
}

#[derive(Debug)]
struct xmlLexer {
    position: usize,
    char: u8,
    read_position: usize,
    input: Vec<u8>,
}

impl xmlLexer {
    pub fn new(input: String) -> xmlLexer {
        let mut lexer = xmlLexer {
            position: 0,
            char: 0,
            read_position: 0,
            input: input.into_bytes(),
        };

        lexer.read_char();

        lexer
    }

    pub fn next_token(&mut self) -> XmlToken {
        let token = match self.char {
            b'<' => {
                if self.look_forward() == b'/' {
                    self.read_char();
                    self.read_char();
                    let tag_name = self.read_name();
                    self.skip_space();
                    XmlToken::TagClose(tag_name)
                } else if self.look_forward() == b'?' {
                    self.read_processing_instruction();
                    XmlToken::ProcessingInstruction
                } else {
                    self.read_char();
                    let tag_name = self.read_name();
                    self.skip_space();
                    XmlToken::TagOpen(tag_name)
                }
            }

            b'>' => XmlToken::TagEnd,
            b'a'..=b'z' | b'A'..=b'Z' => {
                self.skip_space();

                if self.look_backward() == b'>' {
                    let data = self.read_data();
                    XmlToken::Data(data)
                } else {
                    let attribute_name = self.read_attribute_name();
                    XmlToken::AttributeName(attribute_name)
                }
            }
            b'"' => {
                self.read_char();
                if self.char == b'"' {
                    XmlToken::AttributeValue(String::new())
                } else {
                    let attribute_value = self.read_attribute_value();
                    self.read_char();
                    self.skip_space();
                    XmlToken::AttributeValue(attribute_value)
                }
            }
            b'=' => {
                if self.look_backward() == b'>' {
                    let data = self.read_data();
                    XmlToken::Data(data)
                } else {
                    XmlToken::Assert
                }
            }
            b'/' => {
                if self.look_forward() == b'>' {
                    self.read_char();
                    XmlToken::TagSelfClose
                } else {
                    let data = self.read_data();
                    XmlToken::Data(data)
                }
            }
            char if char.is_ascii_whitespace() => {
                if self.look_backward() == b'=' {
                    self.skip_space();
                    // For the space
                    self.read_char();
                    // For the "
                    self.read_char();
                    let attribute_value = self.read_attribute_value();
                    // For the "
                    self.read_char();
                    // For the space
                    self.skip_space();
                    XmlToken::AttributeValue(attribute_value)
                } else {
                    let data = self.read_data();
                    if self.char == 0 {
                        return XmlToken::Eof;
                    }
                    XmlToken::Data(data)
                }
            }
            0 => XmlToken::Eof,
            _ => {
                let data = self.read_data();
                if self.char == 0 {
                    return XmlToken::Eof;
                }
                XmlToken::Data(data)
            }
        };

        self.read_char();
        return token;
    }

    fn read_processing_instruction(&mut self) {
        // Will just read until the next > and skip it
        while self.char != b'>' && self.char != 0 {
            self.read_char()
        }
    }

    fn read_attribute_value(&mut self) -> String {
        if self.char == b'"' {
            return String::new();
        }

        let mut attr_val = vec![self.char];
        while self.look_forward() != b'"' && self.char != 0 {
            attr_val.push(self.look_forward());
            self.read_char()
        }
        std::string::String::from_utf8(attr_val).unwrap()
    }

    fn read_attribute_name(&mut self) -> String {
        let mut attr_name = vec![self.char];
        while self.look_forward() != b'=' && self.char != 0 {
            attr_name.push(self.look_forward());
            self.read_char()
        }
        std::string::String::from_utf8(attr_name)
            .unwrap()
            .trim()
            .to_string()
    }

    fn read_name(&mut self) -> String {
        let mut tag_name = vec![self.char];
        while self.look_forward().is_ascii_alphanumeric()
            || self.look_forward() == b'-'
            || self.look_forward() == b'_' && self.char != 0
        {
            tag_name.push(self.look_forward());
            self.read_char()
        }
        return std::string::String::from_utf8(tag_name).unwrap();
    }

    fn read_data(&mut self) -> String {
        let mut data = vec![self.char];
        while self.look_forward() != b'<' && self.char != 0 {
            data.push(self.look_forward());
            self.read_char()
        }
        return std::string::String::from_utf8(data).unwrap();
    }

    fn look_forward(&mut self) -> u8 {
        if self.read_position >= self.input.len() {
            return 0;
        }
        self.input[self.read_position]
    }

    fn look_backward(&mut self) -> u8 {
        if self.position == 0 {
            return 0;
        }
        self.input[self.position - 1]
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.char = 0;
        } else {
            self.char = self.input[self.read_position]
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn skip_space(&mut self) {
        while self.look_forward().is_ascii_whitespace() {
            self.read_char()
        }
    }
}

// Link to concept https://en.wikipedia.org/wiki/Region-based_memory_management

#[derive(Debug)]
struct Tree<T> {
    nodes: Vec<T>,
    children: HashMap<usize, Vec<usize>>,
    parents: HashMap<usize, usize>,
    tag_name: HashMap<String, Vec<usize>>,
}

type StartTagClosed = bool;
type EndTagClosed = bool;

#[derive(Debug)]
enum ElementType {
    Open(StartTagClosed, EndTagClosed),
    Raw,
    SelfClose,
    Unset,
}

#[derive(Debug)]
struct XmlNode {
    element_name: String,
    element_type: ElementType,
    attributes: Option<HashMap<String, String>>,
    data: Option<String>,
}

impl Tree<XmlNode> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            children: HashMap::new(),
            parents: HashMap::new(),
            tag_name: HashMap::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&XmlNode> {
        self.nodes.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut XmlNode> {
        self.nodes.get_mut(index)
    }

    pub fn get_tag(&self, tag_name: &str) -> Option<&Vec<usize>> {
        self.tag_name.get(tag_name)
    }

    pub fn add(&mut self, node: XmlNode, parent: Option<usize>) -> usize {
        let index = self.nodes.len();

        let tag_name = node.element_name.clone();
        self.tag_name
            .entry(tag_name)
            .or_insert_with(Vec::new)
            .push(index);

        self.nodes.push(node);

        if let Some(parent) = parent {
            self.children
                .entry(parent)
                .or_insert_with(Vec::new)
                .push(index);
            self.parents.insert(index, parent);
        }

        index
    }

    pub fn get_children(&self, index: usize) -> Option<&Vec<usize>> {
        self.children.get(&index)
    }
}

#[derive(Debug)]
struct CurrentNodeStack {
    stack: Vec<usize>,
}

impl CurrentNodeStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, node: usize) {
        self.stack.push(node);
    }

    pub fn pop(&mut self) -> Option<usize> {
        self.stack.pop()
    }

    pub fn top(&self) -> Option<&usize> {
        self.stack.last()
    }
}

#[derive(Debug)]
struct XmlPraser {
    tree: Tree<XmlNode>,
    current_token: XmlToken,
    current_node_stack: CurrentNodeStack,
    current_attribute_name: Option<String>,
    lexer: xmlLexer,
}

#[derive(Debug)]
enum XmlParserError {
    ElementTypeError,
    UnexpectedSymbol(String),
    NodeNotFound,
}

impl<'a> XmlPraser {
    pub fn new(input: String) -> Self {
        let mut lexer = xmlLexer::new(input);
        let current_token = lexer.next_token();

        let mut tree = Tree::new();

        let mut current_node_stack = CurrentNodeStack::new();
        let current_attribute_name = None;

        Self {
            tree,
            current_token,
            current_node_stack,
            current_attribute_name,
            lexer,
        }
    }

    pub fn parse(mut self) -> Tree<XmlNode> {
        let mut line = 0;
        while self.current_token != XmlToken::Eof {
            line += 1;
            match &self.current_token {
                XmlToken::Data(data) => self.parse_data_token(data.to_string()),
                XmlToken::TagOpen(tag_name) => self.parse_tag_open_token(tag_name.to_string()),
                XmlToken::TagSelfClose => self.prase_tag_self_close_token().unwrap(),
                XmlToken::TagEnd => self.prase_tag_end_token().unwrap(),
                XmlToken::AttributeName(attribute_name) => self
                    .parse_attribute_name_token(attribute_name.to_string())
                    .unwrap(),
                XmlToken::Assert => self.parse_assert_token().unwrap(),
                XmlToken::AttributeValue(attribute_value) => self
                    .parse_attribute_value_token(attribute_value.to_string())
                    .unwrap(),
                XmlToken::TagClose(tag_name) => {
                    self.parse_tag_close_token(tag_name.to_string()).unwrap()
                }
                _ => (),
            };

            self.current_token = self.lexer.next_token();
        }

        self.tree
    }

    fn parse_data_token(&mut self, data: String) {
        // If the current node has no data, set it
        // Inadvertently it will discard the data, as it is likely to be a whitespace
        // before the first tag

        // Check if is whitespace
        if data.trim().is_empty() {
            return;
        }
        // Need to create a raw node as the order of the data is important
        let node = XmlNode {
            element_name: "raw".to_string(),
            element_type: ElementType::Raw,
            attributes: None,
            data: Some(data),
        };

        self.tree.add(node, self.current_node_stack.top().copied());
    }

    fn parse_tag_open_token(&mut self, tag_name: String) {
        let node = XmlNode {
            element_name: tag_name,
            element_type: ElementType::Unset,
            attributes: None,
            data: None,
        };
        // The element_type is unknown until it is closed

        let node_index = self.tree.add(node, self.current_node_stack.top().copied());
        self.current_node_stack.push(node_index);
    }

    fn prase_tag_self_close_token(&mut self) -> Result<(), XmlParserError> {
        let current_node = match self.current_node_stack.top() {
            Some(node) => node,
            None => {
                return Err(XmlParserError::UnexpectedSymbol(
                    XmlToken::TagSelfClose.to_string(),
                ))
            }
        };

        let mut node = match self.tree.get_mut(*current_node) {
            Some(node) => node,
            None => return Err(XmlParserError::NodeNotFound),
        };

        match node.element_type {
            ElementType::Unset => {
                self.current_node_stack.pop();
                node.element_type = ElementType::SelfClose;
            }
            _ => return Err(XmlParserError::ElementTypeError),
        }
        Ok(())
    }

    fn parse_tag_close_token(&mut self, tag_name: String) -> Result<(), XmlParserError> {
        let current_node = match self.current_node_stack.top() {
            Some(node) => node,
            None => {
                return Err(XmlParserError::UnexpectedSymbol(
                    XmlToken::TagClose(tag_name).to_string(),
                ))
            }
        };

        let mut node = match self.tree.get_mut(*current_node) {
            Some(node) => node,
            None => return Err(XmlParserError::NodeNotFound),
        };

        match node.element_type {
            ElementType::Open(start_tag_open, _) => {
                if !start_tag_open {
                    return Err(XmlParserError::ElementTypeError);
                }
                node.element_type = ElementType::Open(start_tag_open, true);
            }
            _ => return Err(XmlParserError::ElementTypeError),
        }

        if node.element_name != tag_name {
            return Err(XmlParserError::ElementTypeError);
        }

        Ok(())
    }

    fn prase_tag_end_token(&mut self) -> Result<(), XmlParserError> {
        let current_node = match self.current_node_stack.top() {
            Some(node) => node,
            None => return Err(XmlParserError::UnexpectedSymbol(">".to_string())),
        };

        let node = match self.tree.get_mut(*current_node) {
            Some(node) => node,
            None => return Err(XmlParserError::NodeNotFound),
        };

        match node.element_type {
            ElementType::Open(true, true) => {
                self.current_node_stack.pop();
            }
            ElementType::Unset => {
                node.element_type = ElementType::Open(true, false);
            }
            _ => return Err(XmlParserError::ElementTypeError),
        };
        Ok(())
    }

    fn parse_attribute_name_token(&mut self, attibute_name: String) -> Result<(), XmlParserError> {
        match self.current_attribute_name {
            Some(_) => return Err(XmlParserError::UnexpectedSymbol(attibute_name)),
            _ => (),
        }

        self.current_attribute_name = Some(attibute_name);
        Ok(())
    }

    fn parse_assert_token(&self) -> Result<(), XmlParserError> {
        match self.current_attribute_name {
            Some(_) => return Ok(()),
            _ => {
                return Err(XmlParserError::UnexpectedSymbol(
                    XmlToken::Assert.to_string(),
                ))
            }
        }
    }

    fn parse_attribute_value_token(
        &mut self,
        attribute_value: String,
    ) -> Result<(), XmlParserError> {
        match self.current_attribute_name {
            Some(_) => (),
            _ => {
                return Err(XmlParserError::UnexpectedSymbol(
                    XmlToken::AttributeValue(attribute_value).to_string(),
                ))
            }
        };

        let current_node = match self.current_node_stack.top() {
            Some(node) => node,
            None => return Err(XmlParserError::UnexpectedSymbol(">".to_string())),
        };

        let node = match self.tree.get_mut(*current_node) {
            Some(node) => node,
            None => return Err(XmlParserError::NodeNotFound),
        };

        match node.element_type {
            ElementType::Unset => (),
            _ => return Err(XmlParserError::ElementTypeError),
        };

        match node.attributes {
            Some(ref mut attributes) => {
                attributes.insert(self.current_attribute_name.take().unwrap(), attribute_value);
            }
            None => {
                let mut attributes = HashMap::new();
                attributes.insert(self.current_attribute_name.take().unwrap(), attribute_value);
                node.attributes = Some(attributes);
            }
        };

        Ok(())
    }
}

static XML_HTML_TAG_MAP: [(&str, &str); 1] = [("image", "img")];
static INVALID_TAGS: [&str; 2] = ["script", "style"];
static NEEDS_PROCESSING: [&str; 2] = ["img", "link"];
struct EpubXmlHtmlConverter {
    xml_tree: Tree<XmlNode>,
    epub_doc: EpubDoc<BufReader<File>>,
    html_doc: String,
    book_id: i32,
}

impl EpubXmlHtmlConverter {
    pub fn new(mut epub_doc: EpubDoc<BufReader<File>>, book_id: i32) -> Self {
        let xml_parser = XmlPraser::new(epub_doc.get_current_str().unwrap().0);
        let xml_tree = xml_parser.parse();
        let html_doc = String::new();

        Self {
            xml_tree,
            epub_doc,
            html_doc,
            book_id,
        }
    }

    pub fn convert(&mut self) {
        // First loop through the tree and mutate the tree
        // As xml and html have the same structure, we can use the same tree

        // First we need to find all the images and replace them with the local path
        if let Some(image_nodes) = self.xml_tree.get_tag("image") {
            for node in image_nodes.clone() {
                let node = self.xml_tree.get_mut(node).unwrap();
                let attributes = node.attributes.as_mut().unwrap();
                let image_path = attributes.get("xlink:href").unwrap();
                let new_image_path = format!(
                    "http://127.0.0.1:8273/api/v1/book/{}/resource?path={}",
                    self.book_id,
                    image_path.strip_prefix("../").unwrap_or(image_path)
                );

                attributes.insert("xlink:href".to_string(), new_image_path);
            }
        }

        // Now we need to find all stylelinks and insert the stylesheet as a style tag
        if let Some(stylelink_nodes) = self.xml_tree.get_tag("link") {
            for node in stylelink_nodes.clone() {
                let node = self.xml_tree.get_mut(node).unwrap();
                let attributes = node.attributes.as_mut().unwrap();
                let style_path = attributes.get("href").unwrap();
                let style = extract_styles(
                    &mut self.epub_doc,
                    style_path
                        .strip_prefix("../")
                        .unwrap_or(style_path)
                        .to_string(),
                );
                let style = std::str::from_utf8(&style).unwrap();
                node.element_name = "style".to_string();
                node.element_type = ElementType::Open(true, true);
                node.attributes = None;
                node.data = Some(style.to_string());
            }
        }

        if let Some(img_nodes) = self.xml_tree.get_tag("img") {
            for node in img_nodes.clone() {
                let node = self.xml_tree.get_mut(node).unwrap();
                let attributes = node.attributes.as_mut().unwrap();
                let image_path = attributes.get("src").unwrap();
                let new_image_path = format!(
                    "http://127.0.0.1:8273/api/v1/book/{}/resource?path={}",
                    self.book_id,
                    image_path.strip_prefix("../").unwrap_or(image_path)
                );
                attributes.insert("src".to_string(), new_image_path);
            }
        }

        // Just as a test we end here for now, but we need to do more

        // Chnage the html tag to a div tag, as we don't need the html tag
    }

    fn generate_html(&mut self) -> String {
        let mut result = String::new();

        let preprocess = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_string();
        result.push_str(&preprocess);
        serialize_element(0, &self.xml_tree, &mut result);

        return result;
    }
}

fn serialize_element(node_index: usize, tree: &Tree<XmlNode>, result: &mut String) {
    let node = tree.get(node_index).unwrap();

    match node.element_type {
        ElementType::Open(_, _) => {
            result.push_str(&format!("<{}", node.element_name));
            if let Some(attributes) = &node.attributes {
                for (key, value) in attributes {
                    result.push_str(&format!(" {}=\"{}\"", key, value));
                }
            }
            result.push_str(">");

            if let Some(children) = tree.get_children(node_index) {
                for child in children {
                    serialize_element(*child, tree, result);
                }
            }

            if let Some(data) = &node.data {
                result.push_str(&format!("{}", data));
            }

            result.push_str(&format!("</{}>", node.element_name));
        }
        ElementType::SelfClose => {
            result.push_str(&format!("<{}", node.element_name));
            if let Some(attributes) = &node.attributes {
                for (key, value) in attributes {
                    result.push_str(&format!(" {}=\"{}\"", key, value));
                }
            }
            result.push_str("/>");
        }
        ElementType::Raw => {
            if let Some(data) = &node.data {
                result.push_str(&format!("{}", data));
            }
        }
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_lexer() {
        let input = String::from("<a href=\"test\">test</a> ");
        let mut lexer = xmlLexer::new(input);

        assert_eq!(lexer.next_token(), XmlToken::TagOpen(String::from("a")));
        assert_eq!(
            lexer.next_token(),
            XmlToken::AttributeName(String::from("href"))
        );
        assert_eq!(lexer.next_token(), XmlToken::Assert);
        assert_eq!(
            lexer.next_token(),
            XmlToken::AttributeValue(String::from("test"))
        );
        assert_eq!(lexer.next_token(), XmlToken::TagEnd);
        assert_eq!(lexer.next_token(), XmlToken::Data(String::from("test")));
        assert_eq!(lexer.next_token(), XmlToken::TagClose(String::from("a")));
        assert_eq!(lexer.next_token(), XmlToken::TagEnd);
        assert_eq!(lexer.next_token(), XmlToken::Eof);
    }

    #[test]
    fn test_xml_lexer_complex() {
        let input = String::from("<a href = \"test\"> test</a><b>test</b> ");
        let mut lexer = xmlLexer::new(input);

        assert_eq!(lexer.next_token(), XmlToken::TagOpen(String::from("a")));
        assert_eq!(
            lexer.next_token(),
            XmlToken::AttributeName(String::from("href"))
        );
        assert_eq!(lexer.next_token(), XmlToken::Assert);
        assert_eq!(
            lexer.next_token(),
            XmlToken::AttributeValue(String::from("test"))
        );
        assert_eq!(lexer.next_token(), XmlToken::TagEnd);
        assert_eq!(lexer.next_token(), XmlToken::Data(String::from(" test")));
        assert_eq!(lexer.next_token(), XmlToken::TagClose(String::from("a")));
        assert_eq!(lexer.next_token(), XmlToken::TagEnd);
        assert_eq!(lexer.next_token(), XmlToken::TagOpen(String::from("b")));
        assert_eq!(lexer.next_token(), XmlToken::TagEnd);
        assert_eq!(lexer.next_token(), XmlToken::Data(String::from("test")));
        assert_eq!(lexer.next_token(), XmlToken::TagClose(String::from("b")));
        assert_eq!(lexer.next_token(), XmlToken::TagEnd);
        assert_eq!(lexer.next_token(), XmlToken::Eof);
    }

    #[test]
    fn test_xml_lexer_exotic() {
        let input = r#"
        <html xmlns="http://www.w3.org/1999/xhtml">
        <head>
          <title>Test</title>
          <meta name="viewport" content="width=1434, height=2048"/>
          <meta http-equiv="Content-Type" content="text/html; charset=utf-8"/>
        <link href="../stylesheet.css" rel="stylesheet" type="text/css"/>
      <link href="../page_styles.css" rel="stylesheet" type="text/css"/>
      </head>
        <body class="calibre1">
      <div class="main">
      
      <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" width="100%" height="100%" viewBox="0 0 1434 2048" amzn-src-id="18">
      <image width="1434" height="2048" xlink:href="../images/00001.jpeg" amzn-src-id="19"/>
      </svg>
      
      </div>
      </body></html> 
      "#;

        let mut lexer = xmlLexer::new(input.to_string());

        while let token = lexer.next_token() {
            if token == XmlToken::Eof {
                println!("{}", token);
                break;
            }
            println!("{}", token);
        }
    }

    #[test]
    fn test_xml_parser() {
        let input =
            std::fs::read_to_string("")
                .unwrap();

        let mut parser = XmlPraser::new(input.to_string());

        let result = parser.parse();
        println!("{:?}", result);
    }

    #[test]
    fn test_xml_html_converter() {
        let file_path = "";
        let mut doc = EpubDoc::new(&file_path).unwrap();
        doc.set_current_page(23);

        let mut converter = EpubXmlHtmlConverter::new(doc, 1);
        converter.convert();
        let html = converter.generate_html();
        println!("{}", html);
    }

    #[test]
    fn test_lexer() {
        let file_path = "";
        let mut doc = EpubDoc::new(&file_path).unwrap();
        doc.set_current_page(23);

        let mut xmlLexer = xmlLexer::new(doc.get_current_str().unwrap().0);
        while let token = xmlLexer.next_token() {
            if token == XmlToken::Eof {
                println!("{}", token);
                break;
            }
            println!("{}", token);
        }
    }
}
