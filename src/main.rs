use clap::{Parser, Subcommand};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs; // Get the file system.
use std::io;
use std::path::Path;

// Import Modules.
mod interact;
mod interactives;
mod lexer;
mod parsers;

// Define CLI Interface.
#[derive(Parser)]
#[command(name = "seroost")]
#[command(version = env!("APP_VERSION"))]
#[command(about = "Searches the content of documents", long_about = None)]
struct Cli {
    /// Pass an index path.
    /// This path will be saved and used should the search command be used.
    #[arg(short, long)]
    index_path: Option<String>,

    /// Pass a max file size.
    /// Defaults to 25mb
    #[arg(short, long, default_value = "25")]
    file_size: u64,

    /// Pass an output mode.
    /// Available modes: regular, tree, code
    /// Defaults to regular
    #[arg(short, long, default_value = "regular")]
    mode: Option<String>,

    /// Pass a comma-separated list of directories/files to ignore.
    /// Supports simple globs like "*.log" or ".env.*".
    #[arg(short = 'e', long, value_delimiter = ',')]
    ignore: Option<Vec<String>>,

    /// Disable the built-in default ignore list.
    #[arg(long)]
    no_default_ignore: bool,

    /// Minimize output for AI consumption (compact pattern).
    #[arg(short, long)]
    ai: bool,

    /// Pass a comma-separated list of extra file extensions to treat as code files.
    /// Merges with the built-in default set. Example: "jsx,tsx,vue"
    #[arg(short = 'x', long, value_delimiter = ',')]
    code_ext: Option<Vec<String>>,

    #[command(subcommand)]
    command: Option<AppCommands>,
}

#[derive(Subcommand)]
enum AppCommands {
    /// Indexes a directory to enable searching functionality.
    Index,

    /// Searches the Indexed documents for a document matching your description.
    Search {
        /// Term to search for
        #[arg(required = true)]
        term: String,
    },

    /// Displays detailed usage instructions and examples
    Usage,
}

fn invalid_output_mode_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "Invalid Output mode, expected one of: [regular, tree, code]",
    )
}

fn main() -> Result<(), parsers::GlobalError> {
    let cli = Cli::parse();
    let config_path = interact::get_config_path();
    let mut configuration = load_configuration(&config_path)?;
    let index_path = resolve_index_path(&cli, &config_path, &mut configuration)?;
    let max_file_size = effective_file_size(&cli, &configuration) * 1024u64 * 1024u64;
    let output_mode = output_mode(&cli)?;
    let ignore_set = build_ignore_set(&cli, &configuration);
    let code_ext_set = build_code_ext_set(&cli, &configuration);

    match &cli.command {
        Some(AppCommands::Index) => {
            index_documents(&index_path, max_file_size, &ignore_set, cli.ai);
        }
        Some(AppCommands::Search { term }) => {
            interact::search_documents(term, output_mode, &code_ext_set)?;
        }
        Some(AppCommands::Usage) => interact::display_usage(),
        None => {
            println!(
                "{}. Use --help for usage information.",
                "No command provided".red()
            );
            println!("Or try: {} for detailed examples", "seroost usage".green());
        }
    }

    Ok(())
}

fn load_configuration(config_path: &Path) -> Result<HashMap<String, String>, parsers::GlobalError> {
    if !config_path.exists() {
        return Ok(HashMap::new());
    }

    let file = fs::File::open(config_path)?;
    serde_json::from_reader(file).map_err(Into::into)
}

fn resolve_index_path(
    cli: &Cli,
    config_path: &Path,
    configuration: &mut HashMap<String, String>,
) -> Result<String, parsers::GlobalError> {
    if let Some(path) = &cli.index_path {
        save_configuration(cli, config_path, configuration)?;
        return Ok(path.clone());
    }

    if !config_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No index path provided and no saved configuration found. Run `seroost --index-path /path/to/documents index` first.",
        )
        .into());
    }

    Ok(configuration
        .get("index_path")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No index_path in config file"))?
        .clone())
}

fn save_configuration(
    cli: &Cli,
    config_path: &Path,
    configuration: &mut HashMap<String, String>,
) -> Result<(), parsers::GlobalError> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if let Some(path) = &cli.index_path {
        configuration.insert("index_path".to_string(), path.clone());
    }
    if let Some(cli_ignore) = &cli.ignore {
        configuration.insert("ignore".to_string(), cli_ignore.join(","));
    }
    if let Some(code_ext) = &cli.code_ext {
        configuration.insert("code_ext".to_string(), code_ext.join(","));
    }
    configuration.insert("file_size".to_string(), cli.file_size.to_string());
    configuration.insert(
        "no_default_ignore".to_string(),
        cli.no_default_ignore.to_string(),
    );

    let file = fs::File::create(config_path)?;
    serde_json::to_writer(file, configuration)?;
    Ok(())
}

fn effective_file_size(cli: &Cli, configuration: &HashMap<String, String>) -> u64 {
    if cli.file_size != 25 {
        return cli.file_size;
    }

    configuration
        .get("file_size")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(25)
}

fn output_mode(cli: &Cli) -> Result<interact::Mode, io::Error> {
    match cli.mode.as_deref() {
        Some("regular") => Ok(interact::Mode::Regular),
        Some("tree") => Ok(interact::Mode::Tree),
        Some("code") => Ok(interact::Mode::Code),
        _ => Err(invalid_output_mode_error()),
    }
}

fn build_ignore_set(cli: &Cli, configuration: &HashMap<String, String>) -> HashSet<String> {
    let no_default_ignore = cli.no_default_ignore
        || configuration
            .get("no_default_ignore")
            .is_some_and(|value| value == "true");

    let mut ignore_set: HashSet<String> = if no_default_ignore {
        HashSet::new()
    } else {
        interactives::DEFAULT_IGNORE_LIST
            .as_slice()
            .iter()
            .map(|&s| s.to_string())
            .collect()
    };

    if let Some(s) = configuration.get("ignore") {
        for item in s.split(',').map(str::trim).filter(|i| !i.is_empty()) {
            ignore_set.insert(item.to_string());
        }
    }

    if let Some(cli_ignore) = &cli.ignore {
        for item in cli_ignore {
            ignore_set.insert(item.clone());
        }
    }

    ignore_set
}

fn build_code_ext_set(cli: &Cli, configuration: &HashMap<String, String>) -> HashSet<String> {
    let mut set = HashSet::new();

    if let Some(s) = configuration.get("code_ext") {
        for item in s.split(',').map(str::trim).filter(|i| !i.is_empty()) {
            set.insert(item.to_ascii_lowercase());
        }
    }

    if let Some(cli_ext) = &cli.code_ext {
        for item in cli_ext {
            set.insert(item.to_ascii_lowercase());
        }
    }

    set
}

fn index_documents(
    index_path: &str,
    max_file_size: u64,
    ignore_set: &HashSet<String>,
    ai_mode: bool,
) {
    let mut final_ignore_set = ignore_set.clone();
    let gitignore_path = Path::new(index_path).join(".gitignore");
    if let Ok(content) = fs::read_to_string(gitignore_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                final_ignore_set.insert(trimmed.to_string());
            }
        }
    }

    interactives::process_file(
        index_path.to_string(),
        max_file_size,
        final_ignore_set,
        ai_mode,
    );
}
