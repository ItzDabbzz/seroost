use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};

use colored::Colorize;
use crossbeam::channel::{self, unbounded};

use crate::interact;
use crate::lexer;
use crate::parsers;

pub static DEFAULT_IGNORE_LIST: [&str; 122] = [
    ".env",
    ".env.*",
    "*.env",
    "*.env.*",
    "secrets",
    "secret",
    "credentials",
    "credential",
    "certs",
    "cert",
    "ssl",
    "tls",
    "tokens",
    "token",
    "keys",
    "key",
    "id_rsa",
    "id_ed25519",
    "known_hosts",
    "kubeconfig",
    "*.kubeconfig",
    ".kube",
    "*.pem",
    "*.key",
    "*.crt",
    "*.cer",
    "*.p12",
    "*.pfx",
    "*.jks",
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    ".bun",
    ".pnpm-store",
    ".yarn",
    "vendor",
    "venv",
    ".venv",
    "env",
    ".env",
    "envs",
    "virtualenvs",
    "venvs",
    "__pycache__",
    "target",
    ".gradle",
    ".m2",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".turbo",
    ".cache",
    "coverage",
    "storybook-static",
    "public/build",
    "logs",
    "log",
    "tmp",
    "temp",
    "*.tmp",
    "*.temp",
    "*.log",
    "*.pid",
    "dump.rdb",
    "*.rdb",
    "*.aof",
    "data",
    "storage",
    "uploads",
    "upload",
    "media",
    "private",
    "backups",
    "backup",
    "snapshots",
    "snapshot",
    "*.dump",
    "*.sql",
    "*.sqlite",
    "*.sqlite3",
    "*.db",
    "*.db-shm",
    "*.db-wal",
    "*.png",
    "*.jpg",
    "*.jpeg",
    "*.gif",
    "*.webp",
    "*.ico",
    "*.svg",
    "*.mp4",
    "*.mov",
    "*.avi",
    "*.mkv",
    "*.mp3",
    "*.wav",
    "*.ogg",
    "*.zip",
    "*.tar",
    "*.gz",
    "*.tgz",
    "*.7z",
    "*.rar",
    "*.exe",
    "*.dll",
    "*.so",
    "*.dylib",
    "*.bin",
    ".idea",
    ".vscode",
    ".DS_Store",
    "Thumbs.db",
    "*.swp",
    "*.swo",
    ".commandkit",
    ".old",
    ".agents",
    ".ignored",
];

type TermFreq = HashMap<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

fn should_ignore_path(path: &Path, ignore_list: &HashSet<String>) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let normalized_path = path.to_string_lossy().replace('\\', "/");

    ignore_list.iter().any(|pattern| {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return false;
        }

        let stripped_pattern = pattern.strip_suffix('/').unwrap_or(pattern);

        stripped_pattern == file_name
            || stripped_pattern == normalized_path
            || simple_glob_match(stripped_pattern, file_name)
            || simple_glob_match(stripped_pattern, &normalized_path)
    })
}

fn simple_glob_match(pattern: &str, value: &str) -> bool {
    if !pattern.contains('*') {
        return false;
    }

    if pattern == "*" {
        return true;
    }

    let anchored_start = !pattern.starts_with('*');
    let anchored_end = !pattern.ends_with('*');
    let parts: Vec<&str> = pattern.split('*').filter(|part| !part.is_empty()).collect();

    if parts.is_empty() {
        return true;
    }

    let mut remaining = value;
    if anchored_start {
        let first = parts[0];
        if !remaining.starts_with(first) {
            return false;
        }
        remaining = &remaining[first.len()..];
    }

    let start_index = usize::from(anchored_start);
    for part in &parts[start_index..] {
        match remaining.find(part) {
            Some(index) => remaining = &remaining[index + part.len()..],
            None => return false,
        }
    }

    if anchored_end {
        if let Some(last) = parts.last() {
            return value.ends_with(last);
        }
    }

    true
}

pub fn traverse_dirs<P: AsRef<Path>>(
    dir_path: P,
    sender: &channel::Sender<String>,
    ignored_sender: &channel::Sender<String>,
    ignore_list: &HashSet<String>,
) {
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if should_ignore_path(&path, ignore_list) {
                let _ = ignored_sender.send(path.to_string_lossy().to_string());
                continue;
            }

            if path.is_dir() {
                traverse_dirs(path, sender, ignored_sender, ignore_list);
            } else if path.is_file() {
                let _ = sender.send(path.to_string_lossy().to_string());
            }
        }
    }
}

pub fn process_file(path: String, max_file_size: u64, ignore_list: HashSet<String>, ai_mode: bool) {
    let (file_sender, file_receiver) = unbounded::<String>();
    let (ignored_sender, ignored_receiver) = unbounded::<String>();
    let (processing_sender, processing_receiver) = unbounded::<(String, Vec<char>)>();

    let file_sender_clone = file_sender.clone();
    let ignored_sender_clone = ignored_sender.clone();
    let dir_traversal_handle: JoinHandle<()> = thread::spawn(move || {
        traverse_dirs(
            path,
            &file_sender_clone,
            &ignored_sender_clone,
            &ignore_list,
        );
    });

    let term_frequency_calc_handle =
        thread::spawn(move || calculate_term_frequency(&processing_receiver));

    let num_threads = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(2);
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(num_threads);

    for _ in 0..num_threads {
        let file_receiver = file_receiver.clone();
        let processing_sender = processing_sender.clone();

        let handle = thread::spawn(move || {
            while let Ok(file_path) = file_receiver.recv() {
                if let Ok(metadata) = fs::metadata(&file_path) {
                    let file_size = metadata.len();
                    if file_size > max_file_size {
                        if !ai_mode {
                            let megabyte = 1024 * 1024;
                            let whole_mb = file_size / megabyte;
                            let fraction_mb = (file_size % megabyte) * 100 / megabyte;
                            println!(
                                "{} {:?} ({}.{:02}MB)",
                                "Skipping large file:".yellow(),
                                file_path,
                                whole_mb,
                                fraction_mb
                            );
                        }
                        continue;
                    }
                }

                if let Some(content) = read_supported_file(&file_path) {
                    if let Err(err) = processing_sender.send((file_path, content)) {
                        eprintln!(
                            "{} {}",
                            "Error sending file content to receiving channel:".red(),
                            err
                        );
                    }
                }
            }
        });

        handles.push(handle);
    }

    let _ = dir_traversal_handle.join();
    drop(file_sender);
    drop(ignored_sender);

    for handle in handles {
        let _ = handle.join();
    }

    drop(processing_sender);
    let ignored_paths = collect_ignored_paths(&ignored_receiver);

    if let Ok(term_freq_index) = term_frequency_calc_handle.join() {
        save_index(&term_freq_index, &ignored_paths, ai_mode);
    }
}

fn collect_ignored_paths(ignored_receiver: &channel::Receiver<String>) -> Vec<String> {
    ignored_receiver
        .try_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn read_supported_file(file_path: &str) -> Option<Vec<char>> {
    let extension = Path::new(file_path)
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())?;

    let result = match extension.as_str() {
        "pdf" => parsers::read_entire_pdf_file(file_path),
        "txt" => parsers::read_entire_txt_file(file_path),
        "xml" | "xhtml" => parsers::read_entire_xml_file(file_path),
        "html" | "htm" => parsers::read_entire_html_file(file_path).map_err(Into::into),
        "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" | "go" | "php" | "rb" | "swift"
        | "kt" => parsers::read_code_file(file_path).map_err(Into::into),
        _ => return None,
    };

    match result {
        Ok(text) => Some(text.chars().collect()),
        Err(err) => {
            eprintln!(
                "{} {:?}: {}",
                "Error processing file:".red(),
                file_path,
                err
            );
            None
        }
    }
}

fn save_index(term_freq_index: &TermFreqIndex, ignored_paths: &[String], ai_mode: bool) {
    let index_path = interact::get_indeces_path();
    if let Some(parent) = index_path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!(
                "{} {}",
                "Error creating parent directory for index path:".red(),
                err
            );
            return;
        }
    }

    if ai_mode {
        println!("index:{}", index_path.display());
    } else {
        println!(
            "{} {}",
            "Saving index to:".green(),
            index_path.to_string_lossy().blue()
        );
    }

    match fs::File::create(&index_path) {
        Ok(index_file) => {
            if let Err(err) = serde_json::to_writer(index_file, &term_freq_index)
                .map_err(|err| io::Error::other(err.to_string()))
            {
                eprintln!("{} {}", "Error writing search index:".red(), err);
                return;
            }
        }
        Err(err) => {
            eprintln!("{} {}", "Error creating search index:".red(), err);
            return;
        }
    }

    if ai_mode {
        print_ai_indexed_files(term_freq_index);
        print_ai_ignored_paths(ignored_paths);
        println!("done");
    } else {
        print_regular_ignored_paths(ignored_paths);
        println!(
            "{} {} {}",
            "Successfully indexed".green().bold(),
            term_freq_index.len().to_string().yellow().bold(),
            "documents".green().bold()
        );
    }
}

fn print_ai_indexed_files(term_freq_index: &TermFreqIndex) {
    let mut paths = term_freq_index
        .keys()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    paths.sort();

    println!("indexed:{}", paths.len());
    for path in paths {
        println!("I:{path}");
    }
}

fn print_ai_ignored_paths(ignored_paths: &[String]) {
    println!("ignored:{}", ignored_paths.len());
    for path in ignored_paths {
        println!("G:{path}");
    }
}

fn print_regular_ignored_paths(ignored_paths: &[String]) {
    if ignored_paths.is_empty() {
        return;
    }

    println!(
        "{} {}",
        "Ignored roots:".yellow().bold(),
        ignored_paths.len().to_string().yellow()
    );
    for path in ignored_paths {
        println!("  {}", path.bright_black());
    }
}

fn calculate_term_frequency(
    processing_receiver: &channel::Receiver<(String, Vec<char>)>,
) -> TermFreqIndex {
    let mut term_frequency_index = TermFreqIndex::new();

    while let Ok((file_path, content)) = processing_receiver.recv() {
        let mut term_freq = TermFreq::new();
        let lexer = lexer::Lexer::new(&content);

        for term in lexer {
            *term_freq.entry(term).or_insert(0) += 1;
        }

        term_frequency_index.insert(PathBuf::from(file_path), term_freq);
    }

    term_frequency_index
}
