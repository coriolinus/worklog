use diesel::prelude::*;
use diesel::{sqlite::SqliteConnection, ConnectionError};

embed_migrations!();

pub fn establish_connection() -> Result<SqliteConnection, Error> {
    let path = crate::paths::database();
    std::fs::create_dir_all(path.parent().expect("DB path is never the root"))
        .map_err(Error::CreateDatabasePath)?;
    let connection = SqliteConnection::establish(&path.display().to_string())?;
    embedded_migrations::run(&connection)?;
    Ok(connection)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create the database parent directory")]
    CreateDatabasePath(#[source] std::io::Error),
    #[error("failed to connecto to database")]
    Connect(#[from] ConnectionError),
    #[error("database could not be initialized properly")]
    Migrations(#[from] diesel_migrations::RunMigrationsError),
}
