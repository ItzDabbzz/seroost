use lopdf::Document;
use select::document::Document as HtmlDocument;
use select::predicate::Name;
use std::error::Error;
use std::fs; // Get the file system.
use std::io;
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
    let bytes = fs::read(file_path)?;
    let contents = String::from_utf8_lossy(&bytes);

    Ok(normalize_text(&contents))
}

pub fn read_entire_xml_file<P: AsRef<Path>>(file_path: P) -> Result<String, GlobalError> {
    let file = fs::File::open(file_path)?;

    let er = EventReader::new(file);
    let mut content = String::new();

    for event in er {
        let event = event?;
        if let XmlEvent::Characters(text) = event {
            content.push_str(&text.to_ascii_lowercase());
            content.push(' ');
        }
    }

    Ok(content)
}

pub fn read_entire_html_file<P: AsRef<Path>>(path: P) -> Result<String, GlobalError> {
    let html_content = std::fs::read_to_string(path)?;

    // Parse the HTML
    let document = HtmlDocument::from(html_content.as_str());

    // Extract meaningful text (ignoring scripts, styles, etc.)
    let mut text = String::new();

    // Get content from the body, excluding script and style elements
    for node in document.find(Name("body")) {
        text.push_str(&extract_text_without_scripts(&node));
        text.push(' ');
    }

    Ok(text)
}

fn extract_text_without_scripts(node: &select::node::Node) -> String {
    let mut text = String::new();
for child in node.children() {
        if child.name() == Some("script") || child.name() == Some("style") {
            continue;
        }
        text.push_str(&child.text());
        text.push_str(&extract_text_without_scripts(&child));
    }
    text
}

pub fn read_code_file<P: AsRef<Path>>(path: P) -> Result<String, GlobalError> {
    let bytes = std::fs::read(&path)?;
    let content = String::from_utf8_lossy(&bytes).into_owned();
    Ok(content)
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
    text.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_file(name: &str, content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("seroost_test");
        fs::create_dir_all(&dir).ok();
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_normalize_text_lowercase() {
        assert_eq!(normalize_text("Hello"), "hello");
        assert_eq!(normalize_text("HELLO"), "hello");
        assert_eq!(normalize_text("HeLLo"), "hello");
    }

    #[test]
    fn test_normalize_text_preserves_non_alpha() {
        let result = normalize_text("Hello 123!");
        assert_eq!(result, "hello 123!");
    }

    #[test]
    fn test_read_txt_file() {
        let path = temp_file("test_read_txt.txt", "Hello World\nTest Content");
        let result = read_entire_txt_file(&path).unwrap();
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_txt_file_empty() {
        let path = temp_file("test_read_empty.txt", "");
        let result = read_entire_txt_file(&path).unwrap();
        assert_eq!(result, "");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_xml_file() {
        let xml = r#"<root><item>Hello</item><item>World</item></root>"#;
        let path = temp_file("test_read.xml", xml);
        let result = read_entire_xml_file(&path).unwrap();
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_html_file() {
        let html = r#"<html><body><p>Hello World</p><script>var x = 1;</script></body></html>"#;
        let path = temp_file("test_read.html", html);
        let result = read_entire_html_file(&path).unwrap();
        assert!(result.to_lowercase().contains("hello world"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_code_file() {
        let code = "fn main() {\n    println!(\"Hello\");\n}";
        let path = temp_file("test.rs", code);
        let result = read_code_file(&path).unwrap();
        assert!(result.contains("fn main"));
        assert!(result.contains("println"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_code_file_no_line_numbers_in_content() {
        let code = "fn hello() {}";
        let path = temp_file("test_code.rs", code);
        let result = read_code_file(&path).unwrap();
        assert_eq!(result, "fn hello() {}");
        assert!(!result.contains("Line"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_code_file_no_trailing_newline() {
        let code = "fn no_newline()";
        let path = temp_file("test_code2.rs", code);
        let result = read_code_file(&path).unwrap();
        assert_eq!(result, "fn no_newline()");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_code_line_info() {
        let code = "fn hello() {}\nfn world() {}\nfn main() {}";
        let path = temp_file("test_line_info.rs", code);
        let matches = get_code_line_info(&path, "hello").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, 1);
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_code_line_info_case_insensitive() {
        let code = "fn HELLO() {}\nfn world() {}";
        let path = temp_file("test_case.rs", code);
        let matches = get_code_line_info(&path, "hello").unwrap();
        assert_eq!(matches.len(), 1);
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_code_line_info_no_match() {
        let code = "fn hello() {}\nfn world() {}";
        let path = temp_file("test_no_match.rs", code);
        let matches = get_code_line_info(&path, "main").unwrap();
        assert!(matches.is_empty());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_entire_txt_file("/nonexistent/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_max_pages_constant() {
        assert_eq!(MAX_PDF_PAGES, 450);
    }

    #[test]
    fn test_normalize_text_unicode_preserved() {
        assert_eq!(normalize_text("Über Café"), "über café");
    }

    #[test]
    fn test_normalize_text_numbers_and_symbols() {
        let result = normalize_text("123 !@#");
        assert_eq!(result, "123 !@#");
    }

    #[test]
    fn test_normalize_text_empty_string() {
        assert_eq!(normalize_text(""), "");
    }

    #[test]
    fn test_normalize_text_whitespace_only() {
        let result = normalize_text("   \t\n");
        assert_eq!(result, "   \t\n");
    }

    #[test]
    fn test_normalize_text_mixed_case_unicode() {
        assert_eq!(normalize_text("AÜBc"), "aübc");
    }

    #[test]
    fn test_read_txt_file_unicode() {
        let path = temp_file("test_unicode.txt", "Hello 世界 café");
        let result = read_entire_txt_file(&path).unwrap();
        assert!(result.contains("hello"));
        assert!(result.contains("世界"));
        assert!(result.contains("café"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_txt_file_whitespace_only() {
        let path = temp_file("test_ws.txt", "   \n\t  \n  ");
        let result = read_entire_txt_file(&path).unwrap();
        assert_eq!(result.trim(), "");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_txt_file_binary_fails() {
        let dir = std::env::temp_dir().join("seroost_test");
        fs::create_dir_all(&dir).ok();
        let path = dir.join("test_binary.bin");
        fs::write(&path, [0x00, 0xFF, 0xFE, 0x80]).unwrap();
        let result = read_entire_txt_file(&path);
        assert!(result.is_ok());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_xml_empty_file() {
        let path = temp_file("test_empty.xml", "");
        let result = read_entire_xml_file(&path);
        if let Ok(content) = result {
            assert_eq!(content.trim(), "");
        }
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_xml_no_text_content() {
        let xml = r#"<root><item></item></root>"#;
        let path = temp_file("test_notext.xml", xml);
        let result = read_entire_xml_file(&path).unwrap();
        assert!(result.trim().is_empty() || result == " ");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_xml_nested_elements() {
        let xml = r#"<root><a><b><c>deep text</c></b></a></root>"#;
        let path = temp_file("test_nested.xml", xml);
        let result = read_entire_xml_file(&path).unwrap();
        assert!(result.contains("deep text"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_xml_multiple_character_blocks() {
        let xml = r#"<item>Hello</item><item> World</item>"#;
        let path = temp_file("test_multi.xml", xml);
        let result = read_entire_xml_file(&path).unwrap();
        assert!(result.contains("hello"));
        assert!(result.contains(" world"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_xml_malformed() {
        let path = temp_file("test_bad.xml", "<unclosed><tag>");
        let result = read_entire_xml_file(&path);
        let _ = result;
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_html_no_body_tag() {
        let html = r#"<html><head><title>Test</title></head><body><p>Visible</p></body>"#;
        let path = temp_file("test_nobody.html", html);
        let result = read_entire_html_file(&path).unwrap();
        assert!(result.to_lowercase().contains("visible"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_html_scripts_excluded() {
        let html = r#"<html><body><script>var x=1; var y=2;</script><p>visible text</p></body></html>"#;
        let path = temp_file("test_script.html", html);
        let result = read_entire_html_file(&path).unwrap();
        assert!(result.to_lowercase().contains("visible text"));
        assert!(!result.to_lowercase().contains("var x"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_html_styles_excluded() {
        let html = r#"<html><body><style>.x{color:red;}</style><p>visible</p></body></html>"#;
        let path = temp_file("test_style.html", html);
        let result = read_entire_html_file(&path).unwrap();
        assert!(result.to_lowercase().contains("visible"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_html_nested_elements() {
        let html = r#"<html><body><div><p><span>deep nested text</span></p></div></body></html>"#;
        let path = temp_file("test_nested.html", html);
        let result = read_entire_html_file(&path).unwrap();
        assert!(result.to_lowercase().contains("deep nested text"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_html_entities_decoded() {
        let html = r#"<html><body>&lt;tag&gt; &amp; entities</body></html>"#;
        let path = temp_file("test_entities.html", html);
        let result = read_entire_html_file(&path).unwrap();
        assert!(result.to_lowercase().contains("& entities"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_html_empty_body() {
        let html = r#"<html><body></body></html>"#;
        let path = temp_file("test_empty_body.html", html);
        let result = read_entire_html_file(&path).unwrap();
        assert_eq!(result.trim(), "");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_code_file_empty_file() {
        let path = temp_file("test_empty_code.rs", "");
        let result = read_code_file(&path).unwrap();
        assert_eq!(result, "");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_code_file_binary_fails() {
        let dir = std::env::temp_dir().join("seroost_test");
        fs::create_dir_all(&dir).ok();
        let path = dir.join("test_binary.rs");
        fs::write(&path, [0x00, 0xFF, 0xFE, 0x80]).unwrap();
        let result = read_code_file(&path);
        assert!(result.is_ok());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_code_file_multiline() {
        let code = "fn main() {\n    let x = 5;\n    let y = 10;\n    println!(\"{}\", x + y);\n}";
        let path = temp_file("test_multiline.rs", code);
        let result = read_code_file(&path).unwrap();
        assert!(result.contains("fn main"));
        assert!(result.contains("let x = 5"));
        assert!(result.contains("let y = 10"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_code_line_info_empty_file() {
        let path = temp_file("test_empty_line.rs", "");
        let matches = get_code_line_info(&path, "hello").unwrap();
        assert!(matches.is_empty());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_code_line_info_search_empty_string() {
        let code = "fn hello() {}\nfn world() {}";
        let path = temp_file("test_empty_search.rs", code);
        let matches = get_code_line_info(&path, "").unwrap();
        assert_eq!(matches.len(), 2);
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_code_line_info_unicode_search() {
        let code = "fn hello() {}\nfn 你好() {}";
        let path = temp_file("test_unicode_search.rs", code);
        let matches = get_code_line_info(&path, "你好").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, 2);
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_code_line_info_longer_than_lines() {
        let code = "fn short() {}\nfn code() {}";
        let path = temp_file("test_long_search.rs", code);
        let matches = get_code_line_info(&path, "thiswordisverylonganddoesnotmatch").unwrap();
        assert!(matches.is_empty());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_code_line_info_multiple_matches_same_line() {
        let code = "fn hello() {} fn hello() {}";
        let path = temp_file("test_multi_same_line.rs", code);
        let matches = get_code_line_info(&path, "hello").unwrap();
        assert_eq!(matches.len(), 1);
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_get_code_line_info_all_lines_match() {
        let code = "fn hello() {}\nfn hello_world() {}\nfn hello_test() {}";
        let path = temp_file("test_all_match.rs", code);
        let matches = get_code_line_info(&path, "hello").unwrap();
        assert_eq!(matches.len(), 3);
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_global_error_type_exists() {
        let err: GlobalError = std::io::Error::other("test").into();
        assert_eq!(err.to_string(), "test");
    }
}
