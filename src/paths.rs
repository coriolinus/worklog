use std::path::PathBuf;

pub fn database() -> PathBuf {
    dirs::data_local_dir()
        .expect("supported platforms have a data_local dir")
        .join("worklog")
        .join("db.sqlite3")
}

pub fn config() -> PathBuf {
    dirs::config_dir()
        .expect("supported platforms have a config dir")
        .join("worklog")
        .join("config.toml")
}
