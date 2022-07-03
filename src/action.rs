use chrono::{DateTime, Local};
use sqlx::SqliteConnection;

use crate::{db, paths};

pub struct Event {
    pub timestamp: DateTime<Local>,
    pub message: String,
}

pub enum Action {
    Start(Event),
    Stop(Event),
    Report(),
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
            Self::Start(evt) => Self::handle_start_stop(conn, db::EvtType::Start, evt).await,
            Self::Stop(evt) => Self::handle_start_stop(conn, db::EvtType::Stop, evt).await,
            _ => unimplemented!(),
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
        db_evt.insert(conn).await?;

        // output for a start or stop event
        // TODO: return this instead of emitting it here in the library code
        let formatted_timestamp = timestamp.format("%Y-%m-%d %H%M");
        let n_evts_today = db::count_events_today(conn).await?;
        let evt_type_name = evt_type.name();
        println!("[{formatted_timestamp}] #{n_evts_today}: {evt_type_name} {truncated_message}");

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("executing database action")]
    Db(#[from] db::Error),
}
