use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use libdlta::base::{install::install, uninstall::uninstall};

mod structs;
use structs::ConfigFile;

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<String>>()[1..].to_vec();
    if args.is_empty() {
        println!("Usage: dlta <command> <package>");
        return;
    }

    let config = toml::from_str::<ConfigFile>(&fs::read_to_string("/etc/dlta.toml").unwrap_or_else(|_| {
        eprintln!("Could not find /etc/dlta.toml");
        std::process::exit(1);
    })).unwrap_or_else(|err| {
        eprintln!("Problem parsing config file: {}", err);
        std::process::exit(1);
    });

    let repos = &config.repositories;
    let settings = &config.settings;

    match args[0].as_str() {
        "install" => {
            for repo in &repos.local {
                let pathstr = format!("{}/{}", repo.location, args[1]);
                let path = Path::new(&pathstr);
                if path.exists() {
                    std::fs::copy(&pathstr, format!("/tmp/libdlta/pkg/{}", args[1])).unwrap_or_else(|_| {
                        eprintln!("Could not copy {} to /tmp/libdlta/pkg/{}", pathstr, args[1]);
                        std::process::exit(1);
                    });
                } else {
                    eprintln!("Could not find {} in local repos", args[1]);
                }
            }
            for repo in &repos.remote {
                for url in &repo.locations {
                    let path = format!("{}/{}.tar.zst", url, args[2]);
                    let resp = reqwest::blocking::get(&path).unwrap();
                    if resp.status().is_success() {
                        let mut file = fs::File::create(format!("/tmp/libdlta/pkg/{}", args[2])).unwrap();
                        file.write_all(&resp.bytes().unwrap()).unwrap();
                        break;
                    }
                }
            }
            install(Path::new(&format!("/tmp/libdlta/pkg/{}", args[2]))).unwrap_or_else( |err| {
                eprintln!("Problem installing {}: {}", args[1], err);
                std::process::exit(1);
            });
        }
        "remove" => {
            uninstall(&args[2]);
        }
        "debug_p" => {
            println!("{:#?}", config);
        }
        "debug" => {
            println!("{:?}", repos);
        }
        _ => {
            println!("Usage: dlta <command> <package>");
        }
    }
}