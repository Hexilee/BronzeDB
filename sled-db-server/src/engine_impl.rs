use sled::Db;

#[derive(Clone)]
pub struct EngineImpl {
    inner: Db,
}

impl EngineImpl {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            inner: Db::start_default(path).expect(&format!("cannot open db {}", path)),
        }
    }
}
