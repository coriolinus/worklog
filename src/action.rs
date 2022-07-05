use std::fmt;

use chrono::{Date, DateTime, Duration, Local, Utc};
use sqlx::SqliteConnection;

use crate::{
    db::{self, EvtType, Id, RetrieveEvent},
    paths,
};

pub struct Event {
    pub timestamp: DateTime<Local>,
    pub message: String,
}

pub enum Action {
    Start(Event),
    Stop(Event),
    Report(Date<Local>),
    PathDatabase,
    PathConfig,
}

impl Action {
    pub async fn execute(self, conn: &mut SqliteConnection) -> Result<(), Error> {
        match self {
            Self::PathDatabase => {
                let path = paths::database();
                let path = path.display();
                println!("{path}");
                Ok(())
            }
            Self::PathConfig => {
                let path = paths::config();
                let path = path.display();
                println!("{path}");
                Ok(())
            }
            Self::Start(evt) => handle_start_stop(conn, db::EvtType::Start, evt).await,
            Self::Stop(evt) => handle_start_stop(conn, db::EvtType::Stop, evt).await,
            Self::Report(date) => handle_report(conn, date).await,
        }
    }
}

async fn handle_start_stop(
    conn: &mut SqliteConnection,
    evt_type: db::EvtType,
    Event { timestamp, message }: Event,
) -> Result<(), Error> {
    let truncated_message = {
        let mut t = message.clone();
        if message.len() > 40 {
            t.truncate(39);
            t.push('…');
        }
        t
    };

    let db_evt = db::InsertEvent {
        evt_type,
        timestamp: timestamp.into(),
        message,
    };
    let record_number = db_evt.insert(conn).await?;

    // output for a start or stop event
    // TODO: return this instead of emitting it here in the library code
    let formatted_timestamp = timestamp.format("%Y-%m-%d %H%M");
    let evt_type_name = evt_type.name();
    println!("[{formatted_timestamp}] #{record_number}: {evt_type_name} {truncated_message}");

    Ok(())
}

struct Task {
    start: DateTime<Local>,
    stop: Option<DateTime<Local>>,
    id: Id,
    message: String,
}

impl Task {
    fn duration(&self) -> Option<Duration> {
        self.stop.map(|stop| stop - self.start)
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = self.start.format("%H%M");
        let stop = self
            .stop
            .map(|stop| stop.format("%H%M").to_string())
            .unwrap_or(String::from("…   "));
        let duration = self.duration().unwrap_or(Duration::zero());
        let minutes = duration.num_minutes();
        let hours = minutes / 60;
        let minutes = minutes % 60;
        let id = self.id;
        let message = &self.message;

        write!(
            f,
            "[{start}–{stop}] ({hours}:{minutes:02}) #{id}: {message}"
        )
    }
}

async fn handle_report(conn: &mut SqliteConnection, date: Date<Local>) -> Result<(), Error> {
    // get the list of events for the report period
    let local_midnight: DateTime<Utc> = date.and_hms(0, 0, 0).into();
    let next_day = local_midnight + Duration::days(1);
    let events = RetrieveEvent::events_between(conn, local_midnight, next_day).await?;

    // transform into a list of events for the report period
    let mut tasks = Vec::with_capacity(events.len());

    let mut in_progress: Option<Task> = None;
    for event in events {
        if let Some(mut in_progress) = in_progress.take() {
            in_progress.stop = Some(event.timestamp.into());
            tasks.push(in_progress);
        }
        if let EvtType::Start = event.evt_type {
            in_progress = Some(Task {
                start: event.timestamp.into(),
                stop: None,
                id: event.id,
                message: event.message,
            });
        }
    }
    // we might have a final event in progress
    if let Some(in_progress) = in_progress {
        tasks.push(in_progress);
    }

    // now emit all tasks
    println!("{}:", date.format("%Y-%m-%d"));
    println!("-----------");
    for task in &tasks {
        println!("{task}");
    }
    println!("-----------");
    let n = tasks.len();
    let total: Duration = tasks
        .iter()
        .map(|task| task.duration().unwrap_or(Duration::zero()))
        .fold(Duration::zero(), |total, item| total + item);
    let minutes = total.num_minutes();
    let hours = minutes / 60;
    let minutes = minutes % 60;
    println!(" {n:2} tasks   {hours:2}:{minutes:02}");

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("executing database action")]
    Db(#[from] db::Error),
}
