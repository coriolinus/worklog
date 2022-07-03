use chrono::{DateTime, Utc};
use sqlx::{
    query, query_file_scalar, query_scalar,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
    Connection, SqliteConnection,
};

pub type Id = i64;
pub type Count = i32;

pub async fn establish_connection() -> Result<SqliteConnection, Error> {
    let path = crate::paths::database();
    std::fs::create_dir_all(path.parent().expect("DB path is never the root"))?;
    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        // use write-ahead-log and "normal" mode synchronicity for performance
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);
    let mut connection = SqliteConnection::connect_with(&options)
        .await
        .map_err(Error::Connect)?;
    sqlx::migrate!().run(&mut connection).await?;
    Ok(connection)
}

#[derive(Clone, Copy)]
pub enum EvtType {
    Start,
    Stop,
}

impl EvtType {
    pub fn name(self) -> &'static str {
        match self {
            EvtType::Start => "START",
            EvtType::Stop => "STOP",
        }
    }

    async fn id(self, conn: &mut SqliteConnection) -> Result<Id, Error> {
        let name = self.name();
        query_scalar!("select id from evt_type where name = ?", name)
            .fetch_optional(conn)
            .await
            .map(|maybe_id| maybe_id.expect("name is definitely in the db"))
            .map_err(Error::GetEvtId)
    }
}

/// This type can be inserted into the Event database.
pub struct InsertEvent {
    pub evt_type: EvtType,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

impl InsertEvent {
    /// Insert this event into the database, returning its id.
    pub async fn insert(self, conn: &mut SqliteConnection) -> Result<Id, Error> {
        let Self {
            evt_type,
            timestamp,
            message,
        } = self;
        let evt_type_id = evt_type.id(conn).await?;

        query!(
            "insert into events(evt_type, timestamp, message) values (?, ?, ?) returning id",
            evt_type_id,
            timestamp,
            message
        )
        .fetch_one(conn)
        .await
        .map(|row| row.id)
        .map_err(Error::InsertEvent)
    }
}

/// Return the count of start events which have happened today
pub async fn count_events_today(conn: &mut SqliteConnection) -> Result<Count, Error> {
    use chrono::{Duration, Local, Timelike};

    let start_of_day: DateTime<Utc> = Local::now()
        .with_hour(0)
        .expect("hour 0 is valid")
        .with_minute(0)
        .expect("minute 0 is valid")
        .with_second(0)
        .expect("second 0 is valid")
        .with_nanosecond(0)
        .expect("nanosecond 0 is valid")
        .into();

    let end_of_day = start_of_day + Duration::days(1);

    query_file_scalar!("queries/count_events_today.sql", start_of_day, end_of_day)
        .fetch_one(conn)
        .await
        .map_err(Error::CountEvents)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("creating the database parent directory")]
    CreateDatabasePath(#[from] std::io::Error),
    #[error("connecting to database")]
    Connect(#[source] sqlx::Error),
    #[error("applying migrations")]
    Migrations(#[from] sqlx::migrate::MigrateError),
    #[error("getting appropriate evt_type id")]
    GetEvtId(#[source] sqlx::Error),
    #[error("inserting event")]
    InsertEvent(#[source] sqlx::Error),
    #[error("counting events today")]
    CountEvents(#[source] sqlx::Error),
}
