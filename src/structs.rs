#[derive(serde::Deserialize, Debug)]
pub struct ConfigFile {
    pub settings: Settings,
    pub repositories: Repositories,
}

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub fancy_output: bool,
    pub enforce_tls: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct Repositories {
    pub remote: Vec<RemoteRepository>,
    pub local: Vec<LocalRepository>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RemoteRepository {
    pub name: String,
    pub locations: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct LocalRepository {
    pub name: String,
    pub location: String,
}
