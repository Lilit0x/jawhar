use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::exit,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to extracted shamela artifacts
    #[arg(short, long, env = "SHAMELA_EXTRACTED_DIR")]
    extract_dir: String,

    /// id of the book to search for
    #[arg(short, long)]
    id: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for a book using a partail token from the name
    Search {
        ///
        #[arg(short, long)]
        name: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct BookRecord {
    id: String,
    body_store: Option<String>,
    hint: Option<String>,
    body: Option<String>,
    author: Option<String>,
    book: Option<String>,
    date: Option<String>,
    group: Option<String>,
}

fn find_book_by_id(id: &str, shamela_extracted_path: &Path) -> Result<Option<BookRecord>> {
    let books_path = shamela_extracted_path.join("books.jsonl");
    let file = File::open(books_path)?;
    let reader = BufReader::new(file);

    for l in reader.lines() {
        let line = l?;
        let record: BookRecord = serde_json::from_str(&line)
            .with_context(|| "failed to deserialize book record".to_string())?;
        if record.id == id {
            return Ok(Some(record));
        }
    }

    Ok(None)
}

/// for now it returns the id
fn search_for_book(token: &str, shamela_extracted_path: &Path) -> Result<Vec<BookRecord>> {
    let books_path = shamela_extracted_path.join("books.jsonl");
    let file = File::open(books_path)?;
    let reader = BufReader::new(file);

    let mut results = Vec::new();

    for l in reader.lines() {
        let line = l?;
        if line.contains(token) {
            let record: BookRecord = serde_json::from_str(&line)?;
            results.push(record)
        }
    }

    return Ok(results);
}

fn main() -> Result<()> {
    let args = Args::parse();

    let shamela_extraction_dir = PathBuf::from(args.extract_dir);
    ["books", "pages", "titles"].iter().for_each(|file| {
        let path = shamela_extraction_dir.join(format!("{file}.jsonl"));
        if !path.exists() {
            eprintln!(
                "{file}.jsonl does not exist in {}",
                shamela_extraction_dir.to_str().unwrap()
            );
            exit(1);
        }
    });

    if let Some(id) = args.id {
        let res =
            find_book_by_id(&id, &shamela_extraction_dir).context(format!("finding book: {id}"))?;
        println!("{res:#?}")
    }

    if let Some(cmd) = args.command {
        match cmd {
            Commands::Search { name } => {
                search_for_book(&name, &shamela_extraction_dir)
                    .context(format!("searching for book with token: {name}"))?
                    .iter()
                    .for_each(|r| println!("{r:#?}"));
            }
        }
    };

    Ok(())
}
