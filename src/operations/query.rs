use libdlta::database::{query::query as libquery, search::search as libsearch};
use std::path::Path;

pub fn query(packages: Vec<String>) {
    let mut nonmatches: Vec<String> = Vec::new();
    for arg in packages {
        let pkg = libquery(
            &arg,
            Path::new(libdlta::database::initialise::DATABASE_PATH),
        );
        if pkg.name.is_empty() {
            nonmatches.push(arg.to_string());
        } else {
            println!(
                "{} {} - {}",
                pkg.name,
                pkg.version,
                pkg.description.unwrap_or_else(|| "no desc.".to_string())
            );
        }
    }
    if !nonmatches.is_empty() {
        eprintln!("Could not find packages: {}", nonmatches.join(", "));
    }
}

pub fn search(packages: Vec<String>) {
    let mut nonmatches: Vec<String> = Vec::new();
    for arg in packages {
        let pkgs = libsearch(
            &arg,
            Path::new(libdlta::database::initialise::DATABASE_PATH),
        );
        for pkg in pkgs {
            if pkg.name.is_empty() {
                nonmatches.push(arg.to_string());
            } else {
                println!(
                    "{} {} - {}",
                    pkg.name,
                    pkg.version,
                    pkg.description.unwrap_or_else(|| "no desc.".to_string())
                );
            }
        }
    }
    if !nonmatches.is_empty() {
        eprintln!("Could not find packages: {}", nonmatches.join(", "));
    }
}
