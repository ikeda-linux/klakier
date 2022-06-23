#[derive(serde::Deserialize, Debug)]
pub struct ConfigFile {
    pub settings: Settings,
    #[serde(rename="repos")]
    pub repositories: Repositories
}

#[derive(serde::Deserialize, Debug)]
pub struct Repositories {
    pub local: Vec<LocalRepository>,
    pub remote: Vec<RemoteRepository>
}

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub fancy_output: bool,
    pub enforce_tls: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct LocalRepository {
    pub name: String,
    pub location: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RemoteRepository {
    pub name: String,
    pub locations: Vec<String>,
}
