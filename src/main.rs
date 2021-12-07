use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use libdlta::base::{install::install, uninstall::uninstall};
use libdlta::database::{query::query, search::search};
use libdlta::advanced::freeze::freeze;

mod structs;
use structs::ConfigFile;

fn main() {
    // grabs the arguments
    let args: Vec<String> = env::args().collect::<Vec<String>>()[1..].to_vec();
    if args.is_empty() {
        println!("Usage: dlta <command> <package>");
        return;
    }

    // sorts arguments into either flags or packages
    let mut flags: Vec<String> = Vec::new();
    let mut packages: Vec<String> = Vec::new();
    for arg in args.iter().skip(1) {
        if arg.starts_with("-") {
            flags.push(arg.to_string());
        } else {
            packages.push(arg.to_string());
        }
    }

    // sets the 0th argument (e.g. dlta **install**) as "oper"
    let oper = args[0].clone();


    let config = toml::from_str::<ConfigFile>(&fs::read_to_string("/etc/dlta.toml").unwrap_or_else(|_| {
        eprintln!("Could not find /etc/dlta.toml");
        std::process::exit(1);
    })).unwrap_or_else(|err| {
        eprintln!("Problem parsing config file: {}", err);
        std::process::exit(1);
    });

    let repos = &config.repositories;
    #[allow(unused_variables)]
    let settings = &config.settings;

    match oper.as_str() {
        "install" => {
            for package in packages.iter() {
                for repo in &repos.local {
                    let pathstr = format!("{}/{}.tar.zst", repo.location, package);
                    let path = Path::new(&pathstr);
                    if path.exists() {
                     std::fs::copy(&pathstr, format!("/tmp/libdlta/pkg/{}", package)).unwrap_or_else(|_| {
                            eprintln!("Could not copy {} to /tmp/libdlta/pkg/{}", pathstr, package);
                            std::process::exit(1);
                        });
                    } else {
                        eprintln!("Could not find {} in local repos", package);
                        eprintln!("{} {}", pathstr, path.exists());
                    }
                }
                for repo in &repos.remote {
                    for url in &repo.locations {
                        let path = format!("{}/{}.tar.zst", url, package);
                        let resp = reqwest::blocking::get(&path).unwrap();
                        if resp.status().is_success() {
                            let mut file = fs::File::create(format!("/tmp/libdlta/pkg/{}", package)).unwrap();
                            file.write_all(&resp.bytes().unwrap()).unwrap();
                            break;
                        }
                    }
                }
                install(Path::new(&format!("/tmp/libdlta/pkg/{}", package))).unwrap_or_else( |err| {
                    eprintln!("Problem installing {}: {}", package, err);
                    std::process::exit(1);
                });
            }
        }
        "uninstall" => {
            for package in packages.iter() {
                uninstall(package);
            }
        }
        "query" => {
            let mut nonmatches: Vec<String> = Vec::new();
            for arg in packages {
                let pkg = query(&arg);
                if pkg.name == "" {
                    nonmatches.push(arg.to_string());
                } else {
                    if flags.contains(&"-v".to_string()) {
                        println!("{:#?}", pkg);
                    } else {
                        println!("{} {} - {}", pkg.name, pkg.version, pkg.description.unwrap_or("no desc.".to_string()));
                    }
                }
            }
            if !nonmatches.is_empty() {
                eprintln!("Could not find packages: {}", nonmatches.join(", "));
            }
            std::process::exit(1);
        }
        "search" => {
            let mut nonmatches: Vec<String> = Vec::new();
            for arg in packages {
                let pkgs = search(&arg);
                for pkg in pkgs {
                    if pkg.name == "" {
                        nonmatches.push(arg.to_string());
                    } else {
                        println!("{} {} - {}", pkg.name, pkg.version, pkg.description.unwrap_or("no desc.".to_string()));
                    }
                }
            }
            if !nonmatches.is_empty() {
                eprintln!("Could not find packages: {}", nonmatches.join(", "));
            }
            std::process::exit(1);
        }
        "freeze" => {
            freeze(true);
            println!("Package management frozen");
        }
        "unfreeze" => {
            freeze(false);
            println!("Package management unfrozen");
        }
        "debug" => {
            if flags.contains(&"-r".to_string()) {
                println!("{:?}", config);
            } else {
                println!("{:#?}", config);
            }
        }
        _ => {
            println!("Usage: dlta <command> <package>");
        }
    }
}