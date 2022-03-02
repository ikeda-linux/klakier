mod install;
mod query;

pub fn query(packages: Vec<String>) {
    query::query(packages);
}

pub fn search(packages: Vec<String>) {
    query::search(packages);
}

pub fn install(packages: Vec<String>) {
    install::install(packages);
}
