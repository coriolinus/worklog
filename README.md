# `worklog`: keep track of what you were working on

Ever had the sinking feeling, at a standup or a sprint review, of "What the hell did I _do_ yesterday? There were 11 tickets, but which were they?" Some people solve this with a notepad and a pen, but I'm a nerd, so I'm going to solve it with a program.

`worklog` is a simple, fast tool for keeping track of what you've been doing, and querying it later.

## Usage

- Start working on a task: `worklog start #1234`. Logs that you started working on #1234 now.
- Stop working on a task: `worklog stop`. Logs that you stopped working on your current task.
- Start working on a task with an offset: `worklog started 15m ago: #2345`. Logs that you started working on #2345 15 minutes ago. The colon is syntactically significant and cannot be omitted.
- Start working on a task at a particular time: `worklog started at 0845: #2345`. Logs that you started working on #2345 at 0845 this morning. The colon is syncactically significant and cannot be omitted.
- Stopping work has `stopped` and `stopped at` variants also with equivalent syntax for logging stopping work.
- What did you do yesterday: `worklog report yesterday`. Lists all tasks started yesterday, ordered by start time.
- What did you do on a particular day: `worklog report for Monday`. Lists all tasks started on Monday, ordered by start time.
- How much time did you spend on each task yesterday: `worklog report --time-tracking yesterday`. Lists all tasks worked on yesterday, ordered by duration of work.

## Model

- Starting a new task implicitly stops the old task.
- Manually stopping a task is therefore never mandatory. If you request a basic report, it will just list the tasks that you started. However, manually stopping tasks gives much more sensible output when requesting a time-tracking report.
- It's assumed that you're a software developer, so things that look like links to issues are linked in the reports, if `worklog` is appropriately configured. Patterns that look like links:
  - `#1234` looks like a link to `https://github.com/configured_default_org/configured_default_repo/issues/1234`.
  - `foo#1234` looks like a link to `https://github.com/configured_default_org/foo/issues/1234`.
  - `foo/bar#1234` looks like a link to `https://github.com/foo/bar/issues/1234`.
- Things enclosed in angle bracket pairs are also assumed to be links:
  - `<example.org>` looks like a link to `https://example.org`.
- If an assumed end-of-work time is configured, then a trailing orphan task (which is neither explicitly nor implicitly stopped on the same day), if started before that time, will be assumed for time-tracking purposes to end at that time.
  - Trailing orphan tasks started after the assumed end-of-work time are assumed to end at midnight.
  - Explicitly stopped tasks respect the explicit stop time; they are not trailing orphans.
  - Explicitly stopped tasks do not have to be stopped on the same day; midnight is no artificial barrier.
