mod tokenizer;

use fext_core::InvertedIndex;
use std::{path::Path, process, rc::Rc};
use tokenizer::Tokenizer;

fn main() {
    let mut args = std::env::args().skip(1);

    let (dir, query) = match (args.next(), args.next()) {
        (Some(d), Some(q)) => (d, q),
        _ => {
            eprintln!("Usage: fext <directory> <query>");
            process::exit(1);
        }
    };

    let tkn = Tokenizer::new();

    let wd = tkn.walkdir(Path::new(&dir)).unwrap_or_else(|e| {
        eprintln!("Error reading directory '{}': {}", dir, e);
        process::exit(1);
    });

    let mut idx = InvertedIndex::new();

    for res in wd {
        let ft = match res {
            Ok(ft) => ft,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            }
        };

        let path: Rc<Path> = Rc::from(ft.path());

        for res in ft {
            match res {
                Ok(token) => idx.push(&token.text, Rc::clone(&path), token.pos),
                Err(e) => eprintln!("Error reading '{}': {}", path.display(), e),
            }
        }
    }

    let mut results: Vec<&Path> = match idx.search(&query.to_lowercase()) {
        Some(files) => files.keys().map(|p| p.as_ref()).collect(),
        None => Vec::new(),
    };

    results.sort();

    for path in results {
        println!("{}", path.display());
    }
}
