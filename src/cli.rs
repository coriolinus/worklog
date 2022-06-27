//! Command line parsing.
//!
//! We're rolling our own CLI parsing here instead of using `structopt` or `clap` because this is a very informal,
//! pseudo-natural-language CLI which is heavy on subcommands and strings and light on parsed data.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_english::Interval;
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "cli_parser.pest"]
pub struct CliParser;

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

impl Cli {
    fn parse(input: &str) -> Result<Self, Error> {
        use pest::iterators::Pair;

        let cli = CliParser::parse(Rule::cli_parser, input)?
            .next()
            .expect("a good match should return Some");

        match cli.as_rule() {
            Rule::start => todo!(),
            Rule::stop => todo!(),
            Rule::started => todo!(),
            Rule::stopped => todo!(),
            Rule::started_at => todo!(),
            Rule::stopped_at => todo!(),
            Rule::report => todo!(),
            Rule::report_for => todo!(),
            Rule::EOI
            | Rule::WHITESPACE
            | Rule::cli_parser
            | Rule::time_tracking
            | Rule::message
            | Rule::time_spec
            | Rule::interval
            | Rule::bare_message
            | Rule::relative_message
            | Rule::absolute_message => {
                unreachable!("cannot arrive at these rules from the top-level cli")
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parse error")]
    Parse(#[from] pest::error::Error<Rule>),
}
