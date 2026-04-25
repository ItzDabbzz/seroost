use crate::parsers;
use colored::Colorize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;

use std::path::{Path, PathBuf};

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

use crate::lexer;

#[derive(Clone, Copy)]
pub enum Mode {
    Regular,
    Tree,
    Code,
}

pub fn display_usage() {
    println!("{}", "═".repeat(80).cyan());
    println!(
        "{}",
        "SEROOST DETAILED USAGE GUIDE".bold().green().underline()
    );
    println!("{}", "═".repeat(80).cyan());
    println!();

    display_installation_usage();

    // CLI Options section
    println!("{}", "CLI OPTIONS".yellow().bold());
    println!("  {}", "-i, --index-path <PATH> : directory to index; saved for later searches".bright_black());
    println!("  {}", "-f, --file-size <MB> : max file size; default 25".bright_black());
    println!("  {}", "-m, --mode <regular|tree|code> : search output mode; default regular".bright_black());
    println!("  {}", "-e, --ignore <PATTERNS> : comma-separated extra ignore patterns".bright_black());
    println!("  {}", "--no-default-ignore : disable built-in ignores".bright_black());
    println!("  {}", "-a, --ai : compact index output for AI consumption".bright_black());
    println!();
    // Sample documents section
    println!("{}", "CREATING SAMPLE DOCUMENTS".yellow().bold());
    println!("Create a sample document directory for testing:");
    println!("  {} mkdir -p ~/documents/samples", "$".bright_black());
    println!("  {} cd ~/documents/samples", "$".bright_black());
    println!(
        "  {} echo \"Rust is a systems programming language focused on safety.\" > rust.txt",
        "$".bright_black()
    );
    println!(
        "  {} echo \"Python is known for its simplicity and readability.\" > python.txt",
        "$".bright_black()
    );
    println!();

    // Indexing section
    println!("{}", "INDEXING DOCUMENTS".yellow().bold());
    println!("Index your documents directory:");
    println!(
        "  {} seroost --index-path ~/documents/samples index",
        "$".bright_black()
    );
    println!();
    println!("{}", "Expected output:".bright_blue());
    println!("  Creating configuration file @: ./indeces/config.json...");
    println!(
        "  {}Indexing directory:{} ~/documents/samples",
        "".green().bold(),
        "".blue()
    );
    println!(
        "  {}Indexing:{} ~/documents/samples/rust.txt",
        "".blue(),
        "".green()
    );
    println!(
        "  {}Indexing:{} ~/documents/samples/python.txt",
        "".blue(),
        "".green()
    );
    println!(
        "  {}Saving index to:{} ./indeces/index.json",
        "".green(),
        "".blue()
    );
    println!(
        "  {}Successfully indexed{} 2 {}documents",
        "".green().bold(),
        "".yellow().bold(),
        "".green().bold()
    );
    println!();

    // Searching section
    println!("{}", "SEARCHING DOCUMENTS".yellow().bold());
    println!("Search through indexed documents:");
    println!(
        "  {} seroost search \"programming language\"",
        "$".bright_black()
    );
    println!(
        "  {} seroost --mode tree search \"programming language\"",
        "$".bright_black()
    );
    println!();
    println!("{}", "Expected output:".bright_blue());
    println!("  {}Loading search index...", "".blue());
    println!(
        "  {}Search results for:{} programming language",
        "".green().bold(),
        "".white().on_blue().bold()
    );
    println!("  {}", "═".repeat(60));
    println!(
        "  {}1. ~/documents/samples/{}rust.txt (Score: 0.28768)",
        "".yellow().bold(),
        "".green().bold()
    );
    println!(
        "  {}2. ~/documents/samples/{}python.txt (Score: 0.14384)",
        "".yellow().bold(),
        "".green().bold()
    );
    println!("  {}", "═".repeat(60));
    println!();

    // Subsequent searches
    println!("{}", "SUBSEQUENT SEARCHES".yellow().bold());
    println!("After the first index, you can search without specifying the path again:");
    println!("  {} seroost search \"readability\"", "$".bright_black());
    println!();
    println!("{}", "═".repeat(80).cyan());
}

fn display_installation_usage() {
    println!("{}", "INSTALLATION".yellow().bold());
    println!("Clone and build the repository:");
    println!(
        "  {} git clone https://github.com/parado-xy/seroost.git",
        "$".bright_black()
    );
    println!("  {} cd seroost", "$".bright_black());
    println!("  {} cargo build --release", "$".bright_black());
    println!();
    println!("Make the binary executable from anywhere (optional):");
    println!(
        "  {} sudo ln -s \"$(pwd)/target/release/seroost\" /usr/local/bin/",
        "$".bright_black()
    );
    println!();
}

pub fn search_documents(query: &str, output_mode: Mode) -> Result<(), parsers::GlobalError> {
    let index_path = get_indeces_path();
    if !Path::new(&index_path).exists() {
        print_missing_index_error(output_mode);
        return Ok(());
    }

    let term_frequency_index = load_index(&index_path)?;
    match output_mode {
        Mode::Regular => println!("{}", "Loading search index...".blue()),
        Mode::Tree | Mode::Code => {}
    }

    let query_chars = query.chars().collect::<Vec<_>>();
    let lexer = lexer::Lexer::new(&query_chars);
    let query_terms: Vec<String> = lexer.collect();

    if query_terms.is_empty() {
        print_empty_query_error(output_mode);
        return Ok(());
    }

    let ranked_docs = rank_documents(&term_frequency_index, &query_terms)?;
    match output_mode {
        Mode::Regular => display_regular_results(query, &ranked_docs),
        Mode::Tree => display_tree_results(query, &ranked_docs),
        Mode::Code => display_code_results(query, &ranked_docs)?,
    }

    Ok(())
}

fn print_missing_index_error(output_mode: Mode) {
    match output_mode {
        Mode::Regular => eprintln!(
            "{}",
            "Error: index file not found. Please run index first."
                .red()
                .bold()
        ),
        Mode::Tree | Mode::Code => {
            eprintln!("{{\"error\": \"index file not found. Please run index first.\"}}");
        }
    }
}

fn print_empty_query_error(output_mode: Mode) {
    match output_mode {
        Mode::Regular => println!("{}", "No valid search terms found.".yellow()),
        Mode::Tree | Mode::Code => println!("{{\"error\": \"No valid search terms found.\"}}"),
    }
}

fn load_index(index_path: &Path) -> Result<TermFreqIndex, parsers::GlobalError> {
    let index_file = fs::File::open(index_path)?;
    let reader = std::io::BufReader::new(index_file);
    serde_json::from_reader(reader).map_err(|error| io::Error::other(error.to_string()).into())
}

fn rank_documents(
    term_frequency_index: &TermFreqIndex,
    query_terms: &[String],
) -> Result<Vec<(PathBuf, f64)>, parsers::GlobalError> {
    let total_docs = term_frequency_index.len() as f64;
    let mut document_frequency: HashMap<String, usize> = HashMap::new();

    for term in query_terms {
        for doc_term_freq in term_frequency_index.values() {
            if doc_term_freq.contains_key(term) {
                *document_frequency.entry(term.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut document_scores: HashMap<PathBuf, f64> = HashMap::new();
    for (doc_path, term_freq) in term_frequency_index {
        let total_terms = term_freq.values().sum::<usize>() as f64;
        let mut score = 0.0;

        for term in query_terms {
            if let Some(&term_count) = term_freq.get(term) {
                let tf = term_count as f64 / total_terms;
                let doc_freq = document_frequency.get(term).unwrap_or(&1);
                let idf = ((total_docs + 1.0) / (*doc_freq as f64 + 1.0)).ln() + 1.0;
                score += tf * idf;
            }
        }

        if score > 0.0 {
            document_scores.insert(doc_path.clone(), score);
        }
    }

    let mut ranked_docs = document_scores.into_iter().collect::<Vec<_>>();
    ranked_docs.sort_by(|(_, score1), (_, score2)| {
        score2
            .partial_cmp(score1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(ranked_docs)
}

fn display_regular_results(query: &str, ranked_docs: &[(PathBuf, f64)]) {
    println!(
        "{} {}",
        "Search results for:".green().bold(),
        query.white().on_blue().bold()
    );

    if ranked_docs.is_empty() {
        println!("{}", "No matching documents found.".yellow());
        return;
    }

    println!("{}", "═".repeat(60).cyan());
    for (index, (path, score)) in ranked_docs.iter().take(10).enumerate() {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        let rank = format!("{}.", index + 1).yellow().bold();
        let colorized_path = if let Some(parent) = path.parent() {
            format!(
                "{}{}/",
                parent.display().to_string().cyan(),
                filename.green().bold()
            )
        } else {
            filename.green().bold().to_string()
        };
        let score_str = format!("Score: {score:.5}").bright_blue();

        println!("{rank} {colorized_path}({score_str})");
    }
    println!("{}", "═".repeat(60).cyan());
}

fn display_tree_results(query: &str, ranked_docs: &[(PathBuf, f64)]) {
    println!(
        "{} {}",
        "Search tree for:".green().bold(),
        query.white().on_blue().bold()
    );

    if ranked_docs.is_empty() {
        println!("{} {}", "└──".bright_black(), "no matches".yellow());
        return;
    }

    let tree = build_search_tree(ranked_docs);
    print_tree_entries(&tree, "", true);
}

fn display_code_results(
    query: &str,
    ranked_docs: &[(PathBuf, f64)],
) -> Result<(), parsers::GlobalError> {
    let results = ranked_docs
        .iter()
        .take(10)
        .enumerate()
        .map(|(index, (path, score))| {
            let line_matches = if is_code_file(path) {
                parsers::get_code_line_info(path, query).unwrap_or_default()
            } else {
                Vec::new()
            };
            let line_matches = line_matches
                .into_iter()
                .map(|(line, content)| serde_json::json!({ "line": line, "content": content }))
                .collect::<Vec<_>>();

            serde_json::json!({
                "rank": index + 1,
                "path": path.to_string_lossy(),
                "score": score,
                "line_matches": line_matches,
            })
        })
        .collect::<Vec<_>>();

    let output = serde_json::json!({
        "query": query,
        "results": results,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn build_search_tree(ranked_docs: &[(PathBuf, f64)]) -> BTreeMap<String, TreeNode> {
    let mut root = BTreeMap::new();

    for (rank, (path, score)) in ranked_docs.iter().take(10).enumerate() {
        let parts = path
            .components()
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        insert_tree_path(&mut root, &parts, rank + 1, *score);
    }

    root
}

fn insert_tree_path(
    tree: &mut BTreeMap<String, TreeNode>,
    parts: &[String],
    rank: usize,
    score: f64,
) {
    let Some((part, remaining)) = parts.split_first() else {
        return;
    };

    let node = tree.entry(part.clone()).or_default();
    if remaining.is_empty() {
        node.result = Some(SearchResultMeta { rank, score });
        return;
    }

    insert_tree_path(&mut node.children, remaining, rank, score);
}

fn print_tree_entries(tree: &BTreeMap<String, TreeNode>, prefix: &str, is_root: bool) {
    let entries = tree.iter().collect::<Vec<_>>();
    for (index, (name, node)) in entries.iter().enumerate() {
        let is_last = index + 1 == entries.len();
        let connector = if is_last { "└── " } else { "├── " };
        let suffix = node
            .result
            .as_ref()
            .map(|meta| {
                format!(
                    " [{} {}]",
                    format!("#{}", meta.rank).yellow().bold(),
                    format!("score={:.5}", meta.score).bright_blue()
                )
            })
            .unwrap_or_default();
        let display_name = if node.result.is_some() {
            name.green().bold().to_string()
        } else {
            name.blue().bold().to_string()
        };
        let connector = connector.bright_black();

        if is_root {
            println!("{connector}{display_name}{suffix}");
        } else {
            println!("{prefix}{connector}{display_name}{suffix}");
        }

        let next_prefix = if is_root {
            if is_last {
                "    ".to_string()
            } else {
                format!("{}   ", "│".bright_black())
            }
        } else {
            format!(
                "{prefix}{}",
                if is_last {
                    "    ".to_string()
                } else {
                    format!("{}   ", "│".bright_black())
                }
            )
        };
        print_tree_entries(&node.children, &next_prefix, false);
    }
}

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, Self>,
    result: Option<SearchResultMeta>,
}

struct SearchResultMeta {
    rank: usize,
    score: f64,
}

fn is_code_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "java"
                    | "cpp"
                    | "c"
                    | "h"
                    | "go"
                    | "php"
                    | "rb"
                    | "swift"
                    | "kt"
            )
        })
}

/// Returns the configuration path based on the system used.
/// If no config path found, it results to directory based config storage.
pub fn get_config_path() -> PathBuf {
    dirs::config_dir().map_or_else(
        || PathBuf::from("./indeces/config.json"),
        |path| path.join("seroost").join("config.json"),
    )
}

/// Returns the configuration path based on the system used.
/// If no config path found, it results to directory based index storage.
pub fn get_indeces_path() -> PathBuf {
    dirs::config_dir().map_or_else(
        || PathBuf::from("./indeces/index.json"),
        |path| path.join("seroost").join("index.json"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index(paths: &[&str], terms: &[&[&str]]) -> TermFreqIndex {
        let mut index = TermFreqIndex::new();
        for (i, path) in paths.iter().enumerate() {
            let mut term_freq = TermFreq::new();
            if i < terms.len() {
                for &term in terms[i] {
                    *term_freq.entry(term.to_string()).or_insert(0) += 1;
                }
            }
            index.insert(PathBuf::from(*path), term_freq);
        }
        index
    }

    #[test]
    fn test_rank_single_match() {
        let terms: [&[&str]; 2] = [&["rust", "programming", "language"], &["python", "language"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["doc1.txt", "doc2.txt"], &term_slices);
        let query = ["rust".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PathBuf::from("doc1.txt"));
    }

    #[test]
    fn test_rank_no_match() {
        let terms: [Vec<&str>; 2] = [vec!["rust", "programming"], vec!["python", "language"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["doc1.txt", "doc2.txt"], &term_slices);
        let query = ["java".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_rank_multi_term() {
        let terms: [Vec<&str>; 3] = [
            vec!["rust", "programming", "language", "rust"],
            vec!["python", "programming", "language"],
            vec!["java", "programming", "language"],
        ];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["doc1.txt", "doc2.txt", "doc3.txt"], &term_slices);
        let query = ["rust".to_string(), "programming".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert!(result.len() >= 2);
        assert_eq!(result[0].0, PathBuf::from("doc1.txt"));
    }

    #[test]
    fn test_rank_score_ordering() {
        let terms: [Vec<&str>; 3] = [
            vec!["rust", "rust", "rust", "other"],
            vec!["rust", "rust", "other", "other"],
            vec!["rust", "other", "other", "other"],
        ];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["a.txt", "b.txt", "c.txt"], &term_slices);
        let query = ["rust".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result[0].1 > result[1].1);
        assert!(result[1].1 > result[2].1);
    }

    #[test]
    fn test_rank_same_score() {
        let terms: [Vec<&str>; 2] = [vec!["rust", "language"], vec!["rust", "language"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["a.txt", "b.txt"], &term_slices);
        let query = ["rust".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file(Path::new("file.rs")));
        assert!(is_code_file(Path::new("file.py")));
        assert!(is_code_file(Path::new("file.js")));
        assert!(is_code_file(Path::new("file.ts")));
        assert!(is_code_file(Path::new("file.java")));
        assert!(is_code_file(Path::new("file.cpp")));
        assert!(is_code_file(Path::new("file.c")));
        assert!(is_code_file(Path::new("file.h")));
        assert!(is_code_file(Path::new("file.go")));
        assert!(is_code_file(Path::new("file.php")));
        assert!(is_code_file(Path::new("file.rb")));
        assert!(is_code_file(Path::new("file.swift")));
        assert!(is_code_file(Path::new("file.kt")));
        assert!(!is_code_file(Path::new("file.txt")));
        assert!(!is_code_file(Path::new("file.md")));
        assert!(!is_code_file(Path::new("file")));
    }

    #[test]
    fn test_is_code_file_case_insensitive() {
        assert!(is_code_file(Path::new("file.RS")));
        assert!(is_code_file(Path::new("file.PY")));
        assert!(is_code_file(Path::new("file.Js")));
    }

    #[test]
    fn test_build_search_tree_single_file() {
        let docs = [(PathBuf::from("/repo/src/main.rs"), 0.5)];
        let tree = build_search_tree(&docs);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_build_search_tree_multiple_files() {
        let docs = [
            (PathBuf::from("/repo/src/main.rs"), 0.5),
            (PathBuf::from("/repo/src/lib.rs"), 0.3),
            (PathBuf::from("/repo/readme.md"), 0.1),
        ];
        let tree = build_search_tree(&docs);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_rank_documents_empty_index() {
        let index = TermFreqIndex::new();
        let query: Vec<String> = vec![];
        let result = rank_documents(&index, &query).unwrap();
        assert!(result.is_empty());
    }

    // --- Additional rank_documents edge cases ---

    #[test]
    fn test_rank_single_document_matching() {
        let terms: [Vec<&str>; 1] = [vec!["rust", "programming", "language"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["only_doc.txt"], &term_slices);
        let query = ["rust".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PathBuf::from("only_doc.txt"));
    }

    #[test]
    fn test_rank_single_document_no_match() {
        let terms: [Vec<&str>; 1] = [vec!["rust", "programming", "language"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["only_doc.txt"], &term_slices);
        let query = ["java".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_rank_all_docs_contain_term() {
        let terms: [Vec<&str>; 5] = [
            vec!["common"],
            vec!["common"],
            vec!["common"],
            vec!["common"],
            vec!["common"],
        ];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["a.txt", "b.txt", "c.txt", "d.txt", "e.txt"], &term_slices);
        let query = ["common".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 5);
        let score = result[0].1;
        for (_, s) in &result {
            assert!((*s - score).abs() < 1e-10);
        }
    }

    #[test]
    fn test_rank_no_doc_contains_any_term() {
        let terms: [Vec<&str>; 3] = [
            vec!["rust", "lang"],
            vec!["python", "lang"],
            vec!["java", "lang"],
        ];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["a.txt", "b.txt", "c.txt"], &term_slices);
        let query = ["ruby".to_string(), "go".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_rank_empty_query_terms() {
        let terms: [Vec<&str>; 2] = [vec!["rust"], vec!["python"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["a.txt", "b.txt"], &term_slices);
        let query: Vec<String> = vec![];
        let result = rank_documents(&index, &query).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_rank_duplicate_query_terms() {
        let terms: [Vec<&str>; 2] = [vec!["rust", "rust", "language"], vec!["rust", "language"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["a.txt", "b.txt"], &term_slices);
        let query = ["rust".to_string(), "rust".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_rank_paths_with_spaces() {
        let terms: [Vec<&str>; 2] = [vec!["rust"], vec!["python"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["my docs/a.txt", "my docs/b.txt"], &term_slices);
        let query = ["rust".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_rank_paths_with_special_chars() {
        let terms: [Vec<&str>; 2] = [vec!["rust"], vec!["python"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["path-with-dashes/file.txt", "path_with_underscores/file.txt"], &term_slices);
        let query = ["rust".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_rank_paths_with_deep_nesting() {
        let terms: [Vec<&str>; 1] = [vec!["rust", "programming", "deep", "nested"]];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["/a/b/c/d/e/f/g/h/i/file.txt"], &term_slices);
        let query = ["rust".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_rank_many_documents() {
        const N: usize = 100;
        let mut paths: Vec<String> = Vec::with_capacity(N);
        let mut all_terms: Vec<Vec<String>> = Vec::with_capacity(N);
        for i in 0..N {
            paths.push(format!("doc_{}.txt", i));
            all_terms.push(vec!["common".to_string(), format!("term_{}", i)]);
        }
        let paths_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let terms_refs: Vec<Vec<&str>> = all_terms.iter().map(|t| t.iter().map(|s| s.as_str()).collect()).collect();
        let term_slices: Vec<&[&str]> = terms_refs.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&paths_refs, &term_slices);
        let query = ["common".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn test_rank_isolated_term_high_score() {
        let terms: [Vec<&str>; 4] = [
            vec!["rust", "programming", "language", "rust"],
            vec!["rust", "programming", "language", "language"],
            vec!["rust"],
            vec!["rust"],
        ];
        let term_slices: Vec<&[&str]> = terms.iter().map(|t| t.as_ref()).collect();
        let index = make_index(&["a.txt", "b.txt", "c.txt", "d.txt"], &term_slices);
        let query = ["rust".to_string()];
        let result = rank_documents(&index, &query).unwrap();
        assert_eq!(result.len(), 4);
        assert!(result[0].1 >= result[1].1);
    }

    // --- is_code_file additional tests ---

    #[test]
    fn test_is_code_file_missing_extensions() {
        assert!(!is_code_file(Path::new("file.jsx")));
        assert!(!is_code_file(Path::new("file.tsx")));
        assert!(!is_code_file(Path::new("file.vue")));
        assert!(!is_code_file(Path::new("file.cs")));
        assert!(!is_code_file(Path::new("file.scala")));
        assert!(!is_code_file(Path::new("file.ex")));
        assert!(!is_code_file(Path::new("file.lua")));
        assert!(!is_code_file(Path::new("file.r")));
    }

    #[test]
    fn test_is_code_file_dotfile() {
        assert!(!is_code_file(Path::new(".gitignore")));
        assert!(!is_code_file(Path::new(".env")));
        assert!(!is_code_file(Path::new(".DS_Store")));
    }

    #[test]
    fn test_is_code_file_dotfile_with_ext() {
        assert!(is_code_file(Path::new(".config.rs")));
        assert!(is_code_file(Path::new(".settings.py")));
    }

    #[test]
    fn test_is_code_file_double_extension() {
        assert!(!is_code_file(Path::new("archive.tar.gz")));
        assert!(!is_code_file(Path::new("backup.tar.zst")));
    }

    // --- build_search_tree additional tests ---

    #[test]
    fn test_build_search_tree_empty() {
        let docs: [(PathBuf, f64); 0] = [];
        let tree = build_search_tree(&docs);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_build_search_tree_single_component_paths() {
        let docs = [(PathBuf::from("file.txt"), 0.5)];
        let tree = build_search_tree(&docs);
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_build_search_tree_shared_prefix() {
        let docs = [
            (PathBuf::from("/repo/src/a.rs"), 0.5),
            (PathBuf::from("/repo/src/b.rs"), 0.3),
        ];
        let tree = build_search_tree(&docs);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_build_search_tree_truncation_at_ten() {
        let mut docs = Vec::new();
        for i in 0..15 {
            docs.push((PathBuf::from(format!("file_{}.txt", i)), (15.0 - i as f64) * 0.1));
        }
        let _tree = build_search_tree(&docs);
    }

    #[test]
    fn test_build_search_tree_same_directory() {
        let docs = [
            (PathBuf::from("/same/dir/a.txt"), 0.9),
            (PathBuf::from("/same/dir/b.txt"), 0.7),
            (PathBuf::from("/same/dir/c.txt"), 0.5),
        ];
        let tree = build_search_tree(&docs);
        assert!(!tree.is_empty());
    }

    // --- display_regular_results tests ---

    #[test]
    fn test_display_regular_results_empty() {
        let docs: Vec<(PathBuf, f64)> = Vec::new();
        display_regular_results("test query", &docs);
    }

    #[test]
    fn test_display_regular_results_single_result() {
        let docs = [(PathBuf::from("/path/file.txt"), 0.12345)];
        display_regular_results("test", &docs);
    }

    #[test]
    fn test_display_regular_results_max_ten() {
        let mut docs = Vec::new();
        for i in 0..15 {
            docs.push((PathBuf::from(format!("doc_{}.txt", i)), (15.0 - i as f64) * 0.1));
        }
        display_regular_results("test", &docs);
    }

    // --- display_tree_results tests ---

    #[test]
    fn test_display_tree_results_empty() {
        let docs: Vec<(PathBuf, f64)> = Vec::new();
        display_tree_results("test", &docs);
    }

    #[test]
    fn test_display_tree_results_many_levels() {
        let docs = [(PathBuf::from("/a/b/c/d/e/f/g/h/i/j/file.txt"), 0.5)];
        display_tree_results("test", &docs);
    }

    // --- search_documents tests ---

    #[test]
    fn test_search_documents_no_index() {
        let result = search_documents("test", Mode::Regular);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_documents_empty_query() {
        let result = search_documents("", Mode::Regular);
        assert!(result.is_ok());
    }

    // --- get_config_path and get_indeces_path tests ---

    #[test]
    fn test_get_config_path_returns_path() {
        let path = get_config_path();
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_get_indeces_path_returns_path() {
        let path = get_indeces_path();
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_mode_clone_copy() {
        let mode = Mode::Regular;
        let _copied = mode;
        let _tree = Mode::Tree;
        let _code = Mode::Code;
    }

    // --- display_code_results tests ---

    #[test]
    fn test_display_code_results_non_code_file() {
        let docs = [(PathBuf::from("/path/file.txt"), 0.5)];
        let result = display_code_results("test", &docs);
        assert!(result.is_ok());
    }

    // --- print_empty_query_error tests ---

    #[test]
    fn test_print_empty_query_error_regular() {
        print_empty_query_error(Mode::Regular);
    }

    #[test]
    fn test_print_empty_query_error_tree() {
        print_empty_query_error(Mode::Tree);
    }

    #[test]
    fn test_print_empty_query_error_code() {
        print_empty_query_error(Mode::Code);
    }

    // --- print_missing_index_error tests ---

    #[test]
    fn test_print_missing_index_error_regular() {
        print_missing_index_error(Mode::Regular);
    }

    #[test]
    fn test_print_missing_index_error_tree() {
        print_missing_index_error(Mode::Tree);
    }

    #[test]
    fn test_display_usage_runs() {
        display_usage();
    }
}
