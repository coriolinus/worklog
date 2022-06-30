//! Command line parsing.
//!
//! We're rolling our own CLI parsing here instead of using `structopt` or `clap` because this is a very informal,
//! pseudo-natural-language CLI which is heavy on subcommands and strings and light on parsed data.
//!
//! Note that we're using Pest's idiomaticity guidelines which specify that we should make heavy use of
//! `unwrap` and `unreachable` based on the parser definitions; it's not a good feeling so far, but we'll see how it goes.

use chrono::{DateTime, Utc};
use chrono_english::Interval;
use pest_consume::{match_nodes, Parser};

type Rule = <CliParser as Parser>::Rule;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "cli_parser.pest"]
pub struct CliParser;

#[pest_consume::parser]
impl CliParser {
    fn time_tracking(input: Node) -> Result<(), pest_consume::Error<Rule>> {
        Ok(())
    }
    fn message(input: Node) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_owned())
    }
    fn time_spec(input: Node) -> Result<&str, pest_consume::Error<Rule>> {
        Ok(input.as_str())
    }
    fn bare_message(input: Node) -> Result<BareMessage, pest_consume::Error<Rule>> {
        Ok(match_nodes!(input.into_children();
            [message(message)] => BareMessage { message }
        ))
    }
    fn relative_message(input: Node) -> Result<RelativeMessage, pest_consume::Error<Rule>> {
        Ok(match_nodes!(input.into_children();
            [time_spec(interval), message(message)] => {
                let interval = chrono_english::parse_duration(interval).map_err(|e| input.error(e))?;
                RelativeMessage { interval, message }
            }
        ))
    }
    fn absolute_message(input: Node) -> Result<AbsoluteMessage, pest_consume::Error<Rule>> {
        Ok(match_nodes!(input.into_children();
            [time_spec(timestamp), message(message)] => {
                let timestamp = chrono_english::parse_date_string(
                    timestamp,
                    Utc::now(),
                    chrono_english::Dialect::Us,
                ).map_err(|e| input.error(e))?;
                AbsoluteMessage { timestamp, message }
            }
        ))
    }
    fn start(input: Node) -> Result<Cli, pest_consume::Error<Rule>> {
        Ok(match_nodes!(input.into_children();
            [bare_message(message)] => Cli::Start(message)
        ))
    }
    fn stop(input: Node) -> Result<Cli, pest_consume::Error<Rule>> {
        Ok(match_nodes!(input.into_children();
            [bare_message(message)] => Cli::Stop(message)
        ))
    }
    fn started(input: Node) -> Result<Cli, pest_consume::Error<Rule>> {
        Ok(match_nodes!(input.into_children();
            [relative_message(message)] => Cli::Started(message)
        ))
    }
    fn stopped(input: Node) -> Result<Cli, pest_consume::Error<Rule>> {
        Ok(match_nodes!(input.into_children();
            [relative_message(message)] => Cli::Stopped(message)
        ))
    }
    fn started_at(input: Node) -> Result<Cli, pest_consume::Error<Rule>> {
        Ok(match_nodes!(input.into_children();
            [absolute_message(message)] => Cli::StartedAt(message)
        ))
    }
    fn stopped_at(input: Node) -> Result<Cli, pest_consume::Error<Rule>> {
        Ok(match_nodes!(input.into_children();
            [absolute_message(message)] => Cli::StoppedAt(message)
        ))
    }
    // fn report(input: Node) -> Result<Cli, pest_consume::Error<Rule>> {
    //     Ok(match_nodes!(input.into_children();
    //         [absolute_message(message)] => Cli::StartedAt(message)
    //     ))
    // }
    // fn report_for(input: Node) -> Result<Cli, pest_consume::Error<Rule>> {
    //     Ok(match_nodes!(input.into_children();
    //         [] => {
    //             let yesterday = Local::today().naive_local() - Duration::days(1);

    //         }
    //     ))
    // }
    fn cli_parser(input: Node) -> Result<Cli, pest_consume::Error<Rule>> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BareMessage {
    message: String,
}

impl BareMessage {
    #[cfg(test)]
    fn new(message: &str) -> Self {
        let message = message.to_string();
        Self { message }
    }
}

#[derive(Debug, PartialEq)]
pub struct RelativeMessage {
    interval: Interval,
    message: String,
}

impl RelativeMessage {
    #[cfg(test)]
    fn new(interval_secs: i32, message: &str) -> Self {
        let interval = Interval::Seconds(interval_secs);
        let message = message.to_string();
        Self { interval, message }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AbsoluteMessage {
    timestamp: DateTime<Utc>,
    message: String,
}

/// This struct represents user input via the CLI.
#[derive(Debug, PartialEq)]
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
        let inputs = CliParser::parse(Rule::cli_parser, input)?;
        let input = inputs.single()?;
        CliParser::cli_parser(input)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parsing human interval from \"{0}\"")]
    ParseInterval(String, #[source] chrono_english::DateError),
    #[error("message is required for start variants")]
    NoStartMessage,
    #[error("unknown command")]
    UnknownCommand,
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(err: pest::error::Error<Rule>) -> Self {
        if let pest::error::ErrorVariant::ParsingError {
            positives,
            negatives,
        } = &err.variant
        {
            if positives == &[Rule::cli_parser] && negatives.is_empty() {
                return Error::UnknownCommand;
            }
        }
        todo!()
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl Eq for Error {}

#[cfg(test)]
mod example_tests {
    use super::*;

    fn expect_ok(msg: &str, expect: Cli) {
        assert_eq!(Cli::parse(msg).unwrap(), expect);
    }

    fn expect_bad(msg: &str, expect: Error) {
        assert_eq!(Cli::parse(msg).unwrap_err(), expect);
    }

    #[test]
    fn glorb() {
        let err = Cli::parse("glorb").unwrap_err();
        dbg!(&err);
        assert!(matches!(err, Error::UnknownCommand));
    }

    #[test]
    fn start_1234() {
        expect_ok("start #1234", Cli::Start(BareMessage::new("#1234")));
    }

    #[test]
    fn bare_start() {
        expect_bad("start", Error::NoStartMessage);
    }

    #[test]
    fn bare_stop() {
        expect_ok("stop", Cli::Stop(BareMessage::new("")));
    }

    #[test]
    fn stop_1234() {
        expect_ok("stop #1234", Cli::Stop(BareMessage::new("#1234")));
    }

    #[test]
    fn started_15m_ago_2345() {
        expect_ok(
            "started 15m ago: #2345",
            Cli::Started(RelativeMessage::new(15 * 60, "#2345")),
        );
    }

    #[test]
    fn started_15m_ago() {
        expect_bad("started 15m ago", Error::NoStartMessage);
    }

    #[test]
    fn stopped_5m_ago() {
        expect_ok(
            "stopped 5m ago",
            Cli::Stopped(RelativeMessage::new(5 * 60, "")),
        );
    }

    #[test]
    fn stopped_5m_ago_2345() {
        expect_ok(
            "stopped 5m ago: #2345",
            Cli::Stopped(RelativeMessage::new(5 * 60, "#2345")),
        );
    }
}
