use chrono::{DateTime, Local};

use crate::paths;

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
    pub fn execute(&self) -> Result<(), Error> {
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
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
