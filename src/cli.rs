//! Command line parsing.
//!
//! We're rolling our own CLI parsing here instead of using `structopt` or `clap` because this is a very informal,
//! pseudo-natural-language CLI which is heavy on subcommands and strings and light on parsed data.
//!
//! Note that we're using Pest's idiomaticity guidelines which specify that we should make heavy use of
//! `unwrap` and `unreachable` based on the parser definitions; it's not a good feeling so far, but we'll see how it goes.

use chrono::{DateTime, Local};
use chrono_english::{Dialect, Interval};
use peg::{error::ParseError, str::LineCol};

peg::parser! {
    grammar cli_parser() for str {
        rule ws() = quiet!{[' ' | '\t']}
        rule space() = quiet!{ws()+}
        rule space_then<T>(r: rule<T>) -> T
            = quiet!{space() r:r() { r }}

        // some basic components:
        // ---------------------
        // time tracking flag isn't part of other messages
        rule time_tracking() -> ()
            = "--time-tracking" / "--tt" { () }
        // messages are essentially anything which fits on a single line
        // turns out that in ASCII, you can express this as the single range below
        rule message() -> String
            = quiet!{msg:$([' '..='~']*) { msg.trim().to_owned() }}
            / expected!("message")
        // time specs can't contain colons
        rule time_spec() -> &'input str
            = quiet!{ts:$((!(":" / "ago" / time_tracking()) [' '..='~'])*) { ts.trim() }}
            / expected!("time_spec")
        // interval might end with "ago"
        rule interval() -> Interval
            = ts:time_spec() "ago"? {?
                chrono_english::parse_duration(ts).or(Err("interval"))
            }
        rule datetime() -> DateTime<Local>
            = ts:time_spec() {?
                chrono_english::parse_date_string(ts, Local::now(), Dialect::Us).or(Err("date"))
        }

        // now build up a few higher-level constructs
        rule bare_message() -> BareMessage
            = message:message() { BareMessage { message } }
        rule colon_message() -> String
            = ":" ws()* message:message() { message }
        rule relative_message(require_message: bool) -> RelativeMessage
            = interval:interval() msg:colon_message()? {?
                if require_message && msg.is_none() {
                    Err("message was required but not present")
                } else {
                    Ok(RelativeMessage { interval, message: msg.unwrap_or_default() })
                }
            }
        rule absolute_message(require_message: bool) -> AbsoluteMessage
            = timestamp:datetime() msg:colon_message()? {?
                if require_message && msg.is_none() {
                    Err("message was required but not present")
                } else {
                    Ok(AbsoluteMessage { timestamp, message: msg.unwrap_or_default() })
                }
            }

        // now the parsers for each CLI variant
        // note the explicit whitespace; we want at least one space after the keyword
        rule start() -> Cli
            = "start" m:space_then(<bare_message()>) {
                Cli::Start(m)
            }
        rule stop() -> Cli
            = "stop" m:space_then(<bare_message()>)? {
                Cli::Stop(m.unwrap_or_default())
            }
        rule started() -> Cli
            = "started" m:space_then(<relative_message(true)>) {
                Cli::Started(m)
            }
        rule stopped() -> Cli
            = "stopped" m:space_then(<relative_message(false)>) {
                Cli::Stopped(m)
            }
        rule started_at() -> Cli
            = "started at" m:space_then(<absolute_message(true)>) {
                Cli::StartedAt(m)
            }
        rule stopped_at() -> Cli
            = "stopped at" m:space_then(<absolute_message(false)>) {
                Cli::StoppedAt(m)
            }

        // TODO later
        // report = { "report" ~ (space ~ interval)? ~ (space ~ time_tracking)? }
        // report_for = { "report for" ~ space ~ time_spec ~ time_tracking? }

        // now the actual top-level parser
        pub rule cli() -> Cli
            = c:(
                started_at() /
                started() /
                start() /
                stopped_at() /
                stopped() /
                stop()
            ) { c }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
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
    timestamp: DateTime<Local>,
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
        cli_parser::cli(input).map_err(Into::into)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parse error")]
    Peg(#[from] ParseError<LineCol>),
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

    macro_rules! expect_bad {
        ($msg:expr => $pattern:pat_param) => {
            assert!(matches!(Cli::parse($msg).unwrap_err(), $pattern))
        };
    }

    #[test]
    fn glorb() {
        expect_bad!("glorb" => Error::Peg(_));
    }

    #[test]
    fn start_1234() {
        expect_ok("start #1234", Cli::Start(BareMessage::new("#1234")));
    }

    #[test]
    fn bare_start() {
        expect_bad!("start" => Error::Peg(_));
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
        expect_bad!("started 15m ago" => Error::Peg(_));
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
