//! Command line parsing.
//!
//! We're rolling our own CLI parsing here instead of using `structopt` or `clap` because this is a very informal,
//! pseudo-natural-language CLI which is heavy on subcommands and strings and light on parsed data.
//
// Any chance it gives me to explore a bunch of parser libraries is a purely incidental benefit.

use chrono::{Date, DateTime, Duration, Local};
use chrono_english::{Dialect, Interval};
use peg::{error::ParseError, str::LineCol};
use worklog::action::{Action, Event};

fn no_start_message(require_message: bool, msg: &Option<String>) -> bool {
    require_message && (msg.is_none() || msg.as_ref().map(|msg| msg.is_empty()).unwrap_or_default())
}

peg::parser! {
    grammar cli_parser() for str {
        rule ws() = quiet!{[' ' | '\t']}
        rule space() = quiet!{ws()+}
        rule space_then<T>(r: rule<T>) -> T
            = quiet!{space() r:r() { r }}

        // some basic components:
        // ---------------------
        // messages are essentially anything which fits on a single line
        // turns out that in ASCII, you can express this as the single range below
        rule message() -> String
            = quiet!{msg:$([' '..='~']*) { msg.trim().to_owned() }}
            / expected!("message")
        // time specs can't contain colons
        rule time_spec() -> &'input str
            = quiet!{ts:$((!(":" / "ago") [' '..='~'])*) { ts.trim() }}
            / expected!("time_spec")
        // interval might end with "ago"
        rule interval() -> Result<Interval, Error>
            = ts:time_spec() "ago"? {
                chrono_english::parse_duration(ts).map_err(|err| Error::ParseInterval(ts.into(), err))
            }
        // some special handling for specific time formats without the day
        rule timefragment(first: bool) -> u32
            = frag:$(['0'..='9']*<{if first {1} else {2}},2>) {
                frag.parse().expect("all two digit numbers can be parsed into u32")
            }
        rule colon_seconds() -> u32
            = ":" s:timefragment(false) { s }
        // returns 0 if "a", or 12 if "p".
        rule am_pm() -> u32
            = ws()* ap:$("a"/"p"/"A"/"P") ("m"/"M")? {
                if ap.eq_ignore_ascii_case("p") {
                    12
                } else {
                    0
                }
            }
        rule civilian_time() -> Result<DateTime<Local>, Error>
            = h:timefragment(true) ":" m:timefragment(false) s:colon_seconds()? pm_offset:am_pm()? {
                Ok(Local::today().and_hms(h + pm_offset.unwrap_or_default(), m, s.unwrap_or_default()))
            }
        rule military_time() -> Result<DateTime<Local>, Error>
            = h:timefragment(false) m:timefragment(false) s:timefragment(false)? {
                Ok(Local::today().and_hms(h, m, s.unwrap_or_default()))
            }
        rule english_date_time() -> Result<DateTime<Local>, Error>
            = ts:time_spec() {
                chrono_english::parse_date_string(ts, Local::now(), Dialect::Us)
                    .map_err(|err| Error::ParseDatetime(ts.into(), err))
        }
        rule datetime() -> Result<DateTime<Local>, Error>
             = dt:(
                military_time() /
                civilian_time() /
                english_date_time()
             ) { dt }

        // now build up a few higher-level constructs
        rule bare_message(require_message: bool) -> Result<BareMessage, Error>
            = msg:message()? {
                if no_start_message(require_message, &msg) {
                    Err(Error::NoStartMessage)
                } else {
                    let message = msg.unwrap_or_default();
                    Ok(BareMessage { message })
                }
             }
        rule colon_message() -> String
            = ":" ws()* message:message() { message }
        rule relative_message(require_message: bool) -> Result<RelativeMessage, Error>
            = interval:interval() msg:colon_message()? {
                let interval = interval?;

                if no_start_message(require_message, &msg) {
                    Err(Error::NoStartMessage)
                } else {
                    let message = msg.unwrap_or_default();
                    Ok(RelativeMessage { interval, message })
                }
            }
        rule absolute_message(require_message: bool) -> Result<AbsoluteMessage, Error>
            = timestamp:datetime() msg:colon_message()? {
                let timestamp = timestamp?;

                if no_start_message(require_message, &msg) {
                    Err(Error::NoStartMessage)
                } else {
                    let message = msg.unwrap_or_default();
                    Ok(AbsoluteMessage { timestamp, message })
                }
            }

        // now the parsers for each CLI variant
        // note the explicit whitespace; we want at least one space after the keyword
        rule start() -> Result<Cli, Error>
            = "start" m:bare_message(true) {
                Ok(Cli::Start(m?))
            }
        rule stop() -> Result<Cli, Error>
            = "stop" m:bare_message(false) {
                Ok(Cli::Stop(m?))
            }
        rule started() -> Result<Cli, Error>
            = "started" m:space_then(<relative_message(true)>) {
                Ok(Cli::Started(m?))
            }
        rule stopped() -> Result<Cli, Error>
            = "stopped" m:space_then(<relative_message(false)>) {
                Ok(Cli::Stopped(m?))
            }
        rule started_at() -> Result<Cli, Error>
            = "started at" m:space_then(<absolute_message(true)>) {
                Ok(Cli::StartedAt(m?))
            }
        rule stopped_at() -> Result<Cli, Error>
            = "stopped at" m:space_then(<absolute_message(false)>) {
                Ok(Cli::StoppedAt(m?))
            }

        // path commands
        rule path_database() -> Result<Cli, Error>
            = "path" "s"? space() ("database" / "db") {
                Ok(Cli::PathDatabase)
            }
        rule path_config() -> Result<Cli, Error>
            = "path" "s"? space() "conf" "ig"? {
                Ok(Cli::PathConfig)
            }

        // we need to be able to create reports for particular days
        rule for_when() -> Result<Date<Local>, Error>
            = "for"? when:time_spec() {
                chrono_english::parse_date_string(when.trim(), Local::now(), Dialect::Us)
                    .map(|dt| dt.date())
                    .map_err(|err| Error::ParseDatetime(when.into(), err))
            }
        rule report() -> Result<Cli, Error>
            = "report" date:space_then(<for_when()>)? {
                let date = date.transpose()?.unwrap_or_else(|| Local::today());
                Ok(Cli::Report(date))
            }

        // catchall for better error messages
        rule catch_command() -> Result<Cli, Error>
            = quiet!{cmd:$((!ws() [' '..='~'])+) message() {
                Err(Error::UnknownCommand(cmd.trim().to_owned()))
            }}

        // now the actual top-level parser
        pub rule cli() -> Result<Cli, Error>
            = c:(
                started_at() /
                started() /
                start() /
                stopped_at() /
                stopped() /
                stop() /
                path_database() /
                path_config() /
                report() /
                // note: this catchall should always be last in the command list
                catch_command()
            ) { c }
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
    timestamp: DateTime<Local>,
    message: String,
}

impl AbsoluteMessage {
    /// Create a new Absolute Message at the specified time today
    #[cfg(test)]
    fn new<'a>(h: u32, m: u32, message: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        let message = message.into().into_owned();
        let timestamp = Local::today().and_hms(h, m, 0);
        Self { timestamp, message }
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
    Report(Date<Local>),
    PathDatabase,
    PathConfig,
}

impl Cli {
    pub fn parse(input: &str) -> Result<Self, Error> {
        cli_parser::cli(input)
            .or_else(|err| Err(Error::UnexpectedParseError(err)))
            .and_then(std::convert::identity)
    }
}

fn interval2duration(interval: Interval) -> Duration {
    match interval {
        Interval::Seconds(s) => Duration::seconds(s.into()),
        Interval::Days(d) => Duration::days(d.into()),
        Interval::Months(m) => Duration::days(Into::<i64>::into(m) * 30),
    }
}

impl From<BareMessage> for Event {
    fn from(BareMessage { message }: BareMessage) -> Self {
        Event {
            timestamp: Local::now(),
            message,
        }
    }
}

impl From<RelativeMessage> for Event {
    fn from(RelativeMessage { interval, message }: RelativeMessage) -> Self {
        Event {
            timestamp: Local::now() - interval2duration(interval),
            message,
        }
    }
}

impl From<AbsoluteMessage> for Event {
    fn from(AbsoluteMessage { timestamp, message }: AbsoluteMessage) -> Self {
        Event { timestamp, message }
    }
}

impl From<Cli> for Action {
    fn from(cli: Cli) -> Self {
        match cli {
            Cli::Start(msg) => Action::Start(msg.into()),
            Cli::Stop(msg) => Action::Stop(msg.into()),
            Cli::Started(msg) => Action::Start(msg.into()),
            Cli::Stopped(msg) => Action::Stop(msg.into()),
            Cli::StartedAt(msg) => Action::Start(msg.into()),
            Cli::StoppedAt(msg) => Action::Stop(msg.into()),
            Cli::PathDatabase => Action::PathDatabase,
            Cli::PathConfig => Action::PathConfig,
            Cli::Report(date) => Action::Report(date),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parsing human interval from \"{0}\"")]
    ParseInterval(String, #[source] chrono_english::DateError),
    #[error("parsing human absolute timestamp from \"{0}\"")]
    ParseDatetime(String, #[source] chrono_english::DateError),
    #[error("message is required for start variants")]
    NoStartMessage,
    #[error("unknown command: \"{0}\"")]
    UnknownCommand(String),
    #[error("parsing cli arguments")]
    UnexpectedParseError(#[source] ParseError<LineCol>),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[cfg(test)]
mod example_tests {
    use chrono::{TimeZone, Timelike};

    use super::*;

    fn expect_ok(msg: &str, expect: Cli) {
        assert_eq!(Cli::parse(msg).unwrap(), expect);
    }

    macro_rules! expect_bad {
        ($msg:expr => $pattern:pat_param) => {
            let err = Cli::parse($msg).unwrap_err();
            dbg!(&err);
            assert!(matches!(err, $pattern))
        };
    }

    #[test]
    fn glorb() {
        expect_bad!("glorb" => Error::UnknownCommand(_));
    }

    #[test]
    fn start_1234() {
        expect_ok("start #1234", Cli::Start(BareMessage::new("#1234")));
    }

    #[test]
    fn bare_start() {
        expect_bad!("start" => Error::NoStartMessage);
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
        expect_bad!("started 15m ago" => Error::NoStartMessage);
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

    #[test]
    fn started_at_0901_foo() {
        expect_ok(
            "started at 0901: foo",
            Cli::StartedAt(AbsoluteMessage::new(9, 1, "foo")),
        );
    }

    #[test]
    fn started_at_1234_bar() {
        expect_ok(
            "started at 12:34: bar",
            Cli::StartedAt(AbsoluteMessage::new(12, 34, "bar")),
        );
    }

    #[test]
    fn started_at_123456_bat() {
        let mut expect_msg = AbsoluteMessage::new(12, 34, "bat");
        expect_msg.timestamp = expect_msg
            .timestamp
            .with_second(56)
            .expect("56 is legal seconds");
        expect_ok("started at 12:34:56: bat", Cli::StartedAt(expect_msg));
    }

    #[test]
    fn started_at_123p_ampm() {
        expect_ok(
            "started at 1:23p: ampm",
            Cli::StartedAt(AbsoluteMessage::new(13, 23, "ampm")),
        );
    }

    #[test]
    fn started_at_0926pm_yem() {
        expect_ok(
            "started at 09:26 PM: yem",
            Cli::StartedAt(AbsoluteMessage::new(12 + 9, 26, "yem")),
        );
    }

    #[test]
    fn started_at_123_p_ampm() {
        expect_ok(
            "started at 1:23 p: ampm",
            Cli::StartedAt(AbsoluteMessage::new(13, 23, "ampm")),
        );
    }

    #[test]
    fn report_bare() {
        expect_ok("report", Cli::Report(Local::today()))
    }

    #[test]
    fn report_today() {
        expect_ok("report today", Cli::Report(Local::today()))
    }

    #[test]
    fn report_yesterday() {
        expect_ok("report yesterday", Cli::Report(Local::today().pred()))
    }

    #[test]
    fn report_2022_07_04() {
        expect_ok(
            "report 2022-07-04",
            Cli::Report(
                Local
                    .from_local_date(&chrono::NaiveDate::from_ymd(2022, 07, 04))
                    .single()
                    .expect("date is unambiguous"),
            ),
        )
    }
}
