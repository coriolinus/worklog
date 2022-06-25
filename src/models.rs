use chrono::{DateTime, Utc};
use diesel::Queryable;

#[derive(Queryable)]
pub struct EvtType {
    id: u64,
    name: String,
}

#[derive(Queryable)]
pub struct Event {
    id: u64,
    evt_type: u64,
    timestamp: DateTime<Utc>,
    message: String,
}
