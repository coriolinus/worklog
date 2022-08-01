use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use futures::TryStreamExt;
use sqlx::{
    query, query_file_as, query_scalar,
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
        // this is a very short-lived process, so force synchronicity
        .journal_mode(SqliteJournalMode::Truncate)
        .synchronous(SqliteSynchronous::Full);
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

    /// Create a function which converts numeric IDs back into instances of Self.
    ///
    /// This ideally be quite fast, if we're avoiding just doing the natural SQL thing.
    async fn unmap(conn: &mut SqliteConnection) -> Result<impl Fn(Id) -> Option<Self>, Error> {
        let start_id = Self::Start.id(conn).await?;
        let stop_id = Self::Stop.id(conn).await?;

        Ok(move |id| {
            if id == start_id {
                Some(Self::Start)
            } else if id == stop_id {
                Some(Self::Stop)
            } else {
                None
            }
        })
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

        // use a transaction to force this query to finalize
        let mut tx = conn.begin().await.map_err(Error::InsertEvent)?;

        let id = query!(
            "insert into events(evt_type, timestamp, message) values (?, ?, ?) returning id",
            evt_type_id,
            timestamp,
            message
        )
        .fetch_one(&mut tx)
        .await
        .map(|row| row.id)
        .map_err(Error::InsertEvent)?;

        // finalize the transaction
        tx.commit().await.map_err(Error::InsertEvent)?;

        Ok(id)
    }
}

#[derive(sqlx::FromRow)]
struct RawRetrieveEvent {
    id: Id,
    evt_type: Id,
    timestamp: NaiveDateTime,
    message: String,
}

pub struct RetrieveEvent {
    pub id: Id,
    pub evt_type: EvtType,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

impl RetrieveEvent {
    /// Retrieve the events between `start` (inclusive) and `end` (exclusive).
    // TODO: rethink this interface, we need to handle overnight explicitly-stopped events
    pub async fn events_between(
        conn: &mut SqliteConnection,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Self>, Error> {
        let unmap_evt = EvtType::unmap(conn).await?;

        let mut events = Vec::new();
        let mut raw_event_stream =
            query_file_as!(RawRetrieveEvent, "queries/events_between.sql", start, end).fetch(conn);

        while let Some(raw_event) = raw_event_stream
            .try_next()
            .await
            .map_err(Error::RetrieveEvents)?
        {
            let evt_type =
                unmap_evt(raw_event.evt_type).expect("only known event types appear here");
            let timestamp = Utc
                .from_local_datetime(&raw_event.timestamp)
                .single()
                .expect("roundtrip conversions to/from UTC should be unambiguous");

            events.push(Self {
                id: raw_event.id,
                evt_type,
                timestamp,
                message: raw_event.message,
            });
        }

        Ok(events)
    }
}

/// Delete an event from the database.
///
/// Return whether or not the event was deleted successfully.
/// Normally this will only be `Ok(false)` if an unused `Id` was entered.
pub async fn delete_event(conn: &mut SqliteConnection, event: Id) -> Result<bool, Error> {
    query!("DELETE FROM events WHERE id = ?", event)
        .execute(conn)
        .await
        .map(|query_result| query_result.rows_affected() != 0)
        .map_err(Error::DeleteEvent)
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
    #[error("retrieving events")]
    RetrieveEvents(#[source] sqlx::Error),
    #[error("deleting event")]
    DeleteEvent(#[source] sqlx::Error),
}
