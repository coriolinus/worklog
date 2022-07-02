use crate::db::establish_connection;

use chrono::{DateTime, Utc};

// fn get_evt_type_named(name: &str) -> EvtType {
//     use self::evt_type::dsl::*;

//     evt_type
//         .filter(name.eq(name))
//         .first(&CONNECTION)
//         .expect("name should exist")
// }

// lazy_static::lazy_static! {
//     pub static ref CONNECTION: SqliteConnection = establish_connection().expect("can establish connection to local sqlite");

//     pub static ref EVT_START: EvtType = get_evt_type_named("START");
//     pub static ref EVT_STOP: EvtType = get_evt_type_named("STOP");
// }

pub struct EvtType {
    id: u64,
    name: String,
}

pub struct Event {
    id: u64,
    evt_type: u64,
    timestamp: DateTime<Utc>,
    message: String,
}
