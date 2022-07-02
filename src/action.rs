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
            Self::Start(_) | Self::Stop(_) => {
                if let Some(evt) = self.into_db_event() {
                    evt.insert(conn).await?;
                    return Ok(());
                }
                unreachable!("duplicate check of action variant")
            }
            _ => unimplemented!(),
        }
    }

    fn into_db_event(self) -> Option<db::InsertEvent> {
        let (evt_type, Event { timestamp, message }) = match self {
            Action::Start(evt) => (db::EvtType::Start, evt),
            Action::Stop(evt) => (db::EvtType::Stop, evt),
            _ => return None,
        };
        Some(db::InsertEvent {
            evt_type,
            timestamp: timestamp.into(),
            message,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("executing database action")]
    Db(#[from] db::Error),
}
