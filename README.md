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
- What did you do on a particular day: `worklog report for last Monday`. Lists all tasks started on Monday, ordered by start time.

## Model

- Starting a new task implicitly stops the old task.
- Manually stopping a task is therefore never mandatory. If you request a basic report, it will just list the tasks that you started. However, manually stopping tasks gives much more sensible output when requesting a time-tracking report.
- It's assumed that you're a software developer, so things that look like links to issues are linked in the reports, if `worklog` is appropriately configured. Patterns that look like links:
  - `#1234` looks like a link to `https://github.com/configured_default_org/configured_default_repo/issues/1234`.
  - `foo#1234` looks like a link to `https://github.com/configured_default_org/foo/issues/1234`.
  - `foo/bar#1234` looks like a link to `https://github.com/foo/bar/issues/1234`.
- Things enclosed in angle bracket pairs are also assumed to be links:
  - `<example.org>` looks like a link to `https://example.org`.
