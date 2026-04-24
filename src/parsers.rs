use lopdf::Document;
use select::document::Document as HtmlDocument;
use select::predicate::{Name, Predicate, Text};
use std::error::Error;
use std::fmt::Write as _;
use std::fs; // Get the file system.
use std::io;
use std::io::{BufReader, Read}; // Get the io module.
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};

pub type GlobalError = Box<dyn Error>;

const MAX_PDF_PAGES: usize = 450;

pub fn read_entire_pdf_file<P: AsRef<Path>>(file_path: P) -> Result<String, GlobalError> {
    let doc = Document::load(file_path)?;
    let pages = doc.get_pages().len().min(MAX_PDF_PAGES);
    let mut page_content = String::new();

    for page in 1..=pages {
        let page_number = u32::try_from(page)?;
        page_content.push_str(&normalize_text(&doc.extract_text(&[page_number])?));
    }

    Ok(page_content)
}

pub fn read_entire_txt_file<P: AsRef<Path>>(file_path: P) -> Result<String, GlobalError> {
    let file = fs::File::open(file_path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    Ok(normalize_text(&contents))
}

pub fn read_entire_xml_file<P: AsRef<Path>>(file_path: P) -> Result<String, GlobalError> {
    let file = fs::File::open(file_path)?;

    let er = EventReader::new(file);
    let mut content = String::new();

    for event in er {
        if let XmlEvent::Characters(text) = event? {
            content.push_str(&text.to_ascii_lowercase());
            content.push(' ');
        }
    }

    Ok(content)
}

pub fn read_entire_html_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let html_content = std::fs::read_to_string(path)?;

    // Parse the HTML
    let document = HtmlDocument::from(html_content.as_str());

    // Extract meaningful text (ignoring scripts, styles, etc.)
    let mut text = String::new();

    // Get content from the body
    for node in document.find(Name("body").descendant(Text)) {
        text.push_str(&node.text());
        text.push(' ');
    }

    Ok(text)
}

pub fn read_code_file<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
    let code_content = std::fs::read_to_string(&path)?;

    let mut numbered_content = String::new();

    for (line_number, line) in code_content.lines().enumerate() {
        let _ = writeln!(numbered_content, "Line {}: {}", line_number + 1, line);
    }

    Ok(numbered_content)
}

// function specifically for getting line information
pub fn get_code_line_info<P: AsRef<Path>>(
    path: P,
    search_term: &str,
) -> Result<Vec<(usize, String)>, io::Error> {
    let code_content = std::fs::read_to_string(&path)?;
    let mut matches = Vec::new();

    for (line_number, line) in code_content.lines().enumerate() {
        if line.to_lowercase().contains(&search_term.to_lowercase()) {
            matches.push((line_number + 1, line.to_string()));
        }
    }

    Ok(matches)
}

fn normalize_text(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_alphabetic() {
                character.to_ascii_lowercase()
            } else {
                character
            }
        })
        .collect()
}
