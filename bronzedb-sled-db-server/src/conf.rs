#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub db_addr: String,
    pub db_path: String,
}

impl Config {
    pub fn new() -> Self {
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();
        settings.try_into().unwrap()
    }
}
