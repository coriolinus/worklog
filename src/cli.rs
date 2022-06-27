//! Command line parsing.
//!
//! We're rolling our own CLI parsing here instead of using `structopt` or `clap` because this is a very informal,
//! pseudo-natural-language CLI which is heavy on subcommands and strings and light on parsed data.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_english::Interval;

lalrpop_mod!(pub cli_parser);

pub struct BareMessage {
    message: String,
}

pub struct RelativeMessage {
    interval: Interval,
    message: String,
}

pub struct AbsoluteMessage {
    timestamp: DateTime<Utc>,
    message: String,
}

/// This struct represents user input via the CLI.
pub enum Cli {
    Start(BareMessage),
    Stop(BareMessage),
    Started(RelativeMessage),
    Stopped(RelativeMessage),
    StartedAt(AbsoluteMessage),
    StoppedAt(AbsoluteMessage),
    // Report {
    //     relative_time: Duration,
    //     time_tracking: bool,
    // },
    // ReportFor {
    //     date: NaiveDate,
    //     time_tracking: bool,
    // },
    // ReportSpan {
    //     from: NaiveDate,
    //     to: NaiveDate,
    //     time_tracking: bool,
    // },
}
