//! Command line parsing.
//!
//! We're rolling our own CLI parsing here instead of using `structopt` or `clap` because this is a very informal,
//! pseudo-natural-language CLI which is heavy on subcommands and strings and light on parsed data.
//!
//! Note that we're using Pest's idiomaticity guidelines which specify that we should make heavy use of
//! `unwrap` and `unreachable` based on the parser definitions; it's not a good feeling so far, but we'll see how it goes.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_english::Interval;
use pest::{iterators::Pair, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "cli_parser.pest"]
pub struct CliParser;

trait HasMessage {
    fn message(&self) -> &str;

    fn ensure_message<E>(self, or: E) -> Result<Self, E>
    where
        Self: Sized,
    {
        if self.message().is_empty() {
            Err(or)
        } else {
            Ok(self)
        }
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

    /// This is only safe within a context where you already know that cli is _definitely_
    /// a container for a `Self`.
    fn parse(cli: Pair<Rule>) -> Self {
        let message = cli
            .into_inner()
            .next()
            .map(|msg_pair| msg_pair.as_str().to_string())
            .unwrap_or_default();
        Self { message }
    }
}

impl HasMessage for BareMessage {
    fn message(&self) -> &str {
        &self.message
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

    /// This is only safe within a context where you already know that cli is _definitely_
    /// a container for a `Self`.
    fn parse(cli: Pair<Rule>) -> Result<Self, Error> {
        let mut relative_message = cli
            .into_inner()
            .next()
            .expect("1/1 guaranteed matches for started")
            .into_inner();

        let time_spec = relative_message
            .next()
            .expect("1/2 guaranteed inners of relative_message")
            .as_str();

        let interval = chrono_english::parse_duration(time_spec)
            .map_err(|err| Error::ParseInterval(time_spec.to_string(), err))?;

        let message = relative_message
            .next()
            .map(|message_node| message_node.as_str().to_string())
            .unwrap_or_default();

        Ok(RelativeMessage { interval, message })
    }
}

impl HasMessage for RelativeMessage {
    fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AbsoluteMessage {
    timestamp: DateTime<Utc>,
    message: String,
}

impl HasMessage for AbsoluteMessage {
    fn message(&self) -> &str {
        &self.message
    }
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
        use pest::iterators::Pair;

        let cli = CliParser::parse(Rule::cli_parser, input)?
            .next()
            .expect("1/1 guaranteed matches for CliParser outer")
            .into_inner()
            .next()
            .expect("1/1 guaranteed matches for CliParser inner");

        Ok(match cli.as_rule() {
            Rule::start => {
                Cli::Start(BareMessage::parse(cli).ensure_message(Error::NoStartMessage)?)
            }
            Rule::stop => Cli::Stop(BareMessage::parse(cli)),
            Rule::started => {
                Cli::Started(RelativeMessage::parse(cli)?.ensure_message(Error::NoStartMessage)?)
            }
            Rule::stopped => Cli::Stopped(RelativeMessage::parse(cli)?),
            Rule::started_at => todo!(),
            Rule::stopped_at => todo!(),
            Rule::report => todo!(),
            Rule::report_for => todo!(),
            Rule::EOI
            | Rule::ws
            | Rule::space
            | Rule::cli_parser
            | Rule::time_tracking
            | Rule::message_char
            | Rule::message
            | Rule::time_spec
            | Rule::interval
            | Rule::bare_message
            | Rule::relative_message
            | Rule::absolute_message => {
                unreachable!("cannot arrive at these rules from the top-level cli")
            }
        })
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
