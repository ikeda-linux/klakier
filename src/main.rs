use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use semver::Version;
use libdlta::base::{install::install, uninstall::uninstall};
use libdlta::base::structs::Package;
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
            let mut packages_nodeps: Vec<Package> = Vec::new();
            let mut packages_deps: Vec<Package> = Vec::new();
            let mut not_found: Vec<String> = Vec::new();
            for package in packages.iter() {
                for repo in &repos.local {
                    // breaks if package already found
                    if Path::new(&format!("/tmp/libdlta/pkg/{}.tar.zst", package)).exists() {
                        break;
                    }
                    // downloads the remote repository db and queries it for the package
                    let db_pathstr = format!("/tmp/dlta/repo_dbs/{}/db.sqlite", repo.name);
                    fs::create_dir_all(format!("/tmp/dlta/repo_dbs/{}", repo.name)).unwrap_or_else(|err| {
                        eprintln!("Could not create database directory: {}", err);
                        std::process::exit(1);
                    });
                    let db_path = Path::new(&db_pathstr);
                    let db_loc = format!("{}/{}", repo.location, "db.sqlite");
                    let mut db_file = fs::File::create(&db_path).unwrap_or_else(|err| {
                        eprintln!("Could not create database file: {}", err);
                        std::process::exit(1);
                    });
                    let db_data = fs::read(&db_loc).unwrap_or_else(|err| {
                        eprintln!("Could not read database file: {}", err);
                        std::process::exit(1);
                    });
                    db_file.write_all(&db_data).unwrap_or_else(|err| {
                        eprintln!("Could not write database file: {}", err);
                        std::process::exit(1);
                    });
                    let res = query(&package, &db_path);
                    let re2 = query(&package, &db_path); // lousy way to do this
                    let re3 = query(&package, &db_path); // but it works

                    // pushes the package to not_found if it's not found in the repo
                    if re2.name == "" {
                        not_found.push(package.to_string());
                    } else {
                        packages_nodeps.push(re2);
                    }
                    
                    // recursively queries the database for the package's dependencies and pushes them to packages_deps until no more unique dependencies are found
                    let deps = re3.dependencies.clone().unwrap_or(vec![]);
                    for dep in deps.iter() {
                        let res = query(&dep, &db_path);
                        if res.name != "" {
                            let mut found = false;
                            for p in packages_deps.iter() {
                                if p.name == res.name {
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                packages_deps.push(res);
                            }
                        }
                    }
                    
                    // queries the repo for all of the package's dependencies and if any return "", push them to not_found
                    for dep in deps.iter() {
                        let res = query(&dep, &db_path);
                        if res.name == "" {
                            not_found.push(dep.to_string());
                        }
                    }
                }
                for repo in &repos.remote {
                    // breaks if package already found
                    if Path::new(&format!("/tmp/libdlta/pkg/{}.tar.zst", package)).exists() {
                        break;
                    }
                    // downloads the remote repository db and queries it for the package
                    let db_pathstr = format!("/tmp/dlta/repo_dbs/{}/db.sqlite", repo.name);
                    let db_path = Path::new(&db_pathstr);
                    let db_url = format!("{}/{}", repo.locations[0], "db.sqlite");
                    let mut db_file = fs::File::create(&db_path).unwrap_or_else(|err| {
                        eprintln!("Could not create database file: {}", err);
                        std::process::exit(1);
                    });
                    let mut db_response = reqwest::blocking::get(&db_url).unwrap_or_else(|err| {
                        eprintln!("Could not download database file: {}", err);
                        std::process::exit(1);
                    });
                    db_response.copy_to(&mut db_file).unwrap_or_else(|err| {
                        eprintln!("Could not copy database file: {}", err);
                        std::process::exit(1);
                    });
                    let res = query(&package, &db_path);
                    let re2 = query(&package, &db_path); // lousy way to do this
                    let re3 = query(&package, &db_path); // but it works

                    // only pushes the package to packages_nodeps if it's not already in packages_nodeps and if it already is, only if the parsed semver is higher
                    if res.name != "" {
                        let mut found = false;
                        for p in packages_nodeps.iter() {
                            if p.name == res.name {
                                if Version::parse(&p.version).unwrap() < Version::parse(&res.version).unwrap() {
                                    packages_nodeps.retain(|x| x.name != res.name);
                                    packages_nodeps.push(res);
                                }
                                found = true;
                                break;
                            }
                        }
                        if found {
                            packages_nodeps.push(re2);
                        } else {
                            not_found.push(package.to_string());
                        }
                    }

                    // recursively queries the database for the package's dependencies and pushes them to packages_deps until no more unique dependencies are found
                    let deps = re3.dependencies.clone().unwrap_or(vec![]);
                    for dep in deps.iter() {
                        let dep_res = query(&dep, &db_path);
                        if dep_res.name != "" {
                            let mut found = false;
                            for p in packages_deps.iter() {
                                if p.name == dep_res.name {
                                    found = true;
                                    break;
                                }
                            }
                            if found {
                                packages_deps.push(dep_res);
                            } else {
                                panic!("Package {} requested dependency {} but it was not found", package, dep);
                            }
                        }
                    }
                }
            }
            // DEBUG: prints out all vecs in raw format for testing
            println!("expl: {:?}\n", packages_nodeps);
            println!("deps: {:?}\n", packages_deps);
            println!("ntfd: {:?}", not_found);
        }
        "uninstall" => {
            for package in packages.iter() {
                uninstall(package);
            }
        }
        "query" => {
            let mut nonmatches: Vec<String> = Vec::new();
            for arg in packages {
                let pkg = query(&arg, Path::new(libdlta::database::initialise::DATABASE_PATH));
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
                let pkgs = search(&arg, Path::new(libdlta::database::initialise::DATABASE_PATH));
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