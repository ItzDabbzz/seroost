use crate::parsers;
use colored::Colorize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::num::TryFromIntError;
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
    let total_docs = usize_to_f64(term_frequency_index.len())?;
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
        let total_terms = usize_to_f64(term_freq.values().sum())?;
        let mut score = 0.0;

        for term in query_terms {
            if let Some(&term_count) = term_freq.get(term) {
                let tf = usize_to_f64(term_count)? / total_terms;
                let doc_freq = document_frequency.get(term).unwrap_or(&1);
                let idf = ((total_docs + 1.0) / (usize_to_f64(*doc_freq)? + 1.0)).ln() + 1.0;
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
        let path_str = path.to_string_lossy();
        let colorized_path = path_str.replace(&*filename, &filename.green().bold().to_string());
        let score_str = format!("Score: {score:.5}").bright_blue();

        println!("{rank} {colorized_path} ({score_str})");
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

fn usize_to_f64(value: usize) -> Result<f64, TryFromIntError> {
    u32::try_from(value).map(f64::from)
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
