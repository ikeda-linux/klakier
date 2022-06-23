use libdlta::advanced::freeze::freeze;
use libdlta::base::uninstall::uninstall;
use std::env;
use std::fs;

mod operations;
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
        if arg.starts_with('-') {
            flags.push(arg.to_string());
        } else {
            packages.push(arg.to_string());
        }
    }

    // sets the 0th argument (e.g. dlta **install**) as "oper"
    let oper = args[0].clone();

    match oper.as_str() {
        "install" => {
            operations::install(packages);
            std::process::exit(0);
        }
        "uninstall" => {
            for package in packages.iter() {
                uninstall(package);
            }
            std::process::exit(0);
        }
        "query" => {
            operations::query(packages);
            std::process::exit(0);
        }
        "search" => {
            operations::search(packages);
            std::process::exit(0);
        }
        "freeze" => {
            freeze(true);
            println!("Package management frozen");
            std::process::exit(0);
        }
        "unfreeze" => {
            freeze(false);
            println!("Package management unfrozen");
            std::process::exit(0);
        }
        "debug" => {
            if flags.contains(&"-r".to_string()) {
                let config =
                    toml::from_str::<ConfigFile>(&fs::read_to_string("/etc/klakier.toml").unwrap_or_else(|_| {
                        eprintln!("Could not find /etc/klakier.toml");
                        std::process::exit(1);
                    }))
                        .unwrap_or_else(|err| {
                            eprintln!("Problem parsing config file: {}", err);
                            std::process::exit(1);
                        });
                println!("{:#?}", config);
            } else {
                let config: toml::Value = toml::from_str(&fs::read_to_string("klakier.toml").unwrap()).unwrap();
                println!("{:#?}", config)
            }
            std::process::exit(0);
        }
        _ => {
            println!("Usage: dlta <command> <package>");
        }
    }
}
