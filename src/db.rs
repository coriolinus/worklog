use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    Connection, SqliteConnection,
};

pub async fn establish_connection() -> Result<SqliteConnection, Error> {
    let path = crate::paths::database();
    std::fs::create_dir_all(path.parent().expect("DB path is never the root"))?;
    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        // use write-ahead-log and "normal" mode synchronicity for performance
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    sqlx::migrate!().run(&mut connection).await?;
    Ok(connection)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create the database parent directory")]
    CreateDatabasePath(#[from] std::io::Error),
    #[error("failed to connect to database")]
    Connect(#[from] sqlx::Error),
    #[error("failed to apply migrations")]
    Migrations(#[from] sqlx::migrate::MigrateError),
}
