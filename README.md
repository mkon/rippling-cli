# rippling-cli
Command line interface to Rippling HR.

## Installation

You can either compile it yourself, but it requires rust ecosystem with `cargo`.

```bash
cargo install --git https://github.com/mkon/rippling-cli.git
```

Alternatively you can download a binary [release](https://github.com/mkon/rippling-cli/releases). At least on MacOS you probably will have to clear the quarantine flag after downloading to make it executable:

```
xattr -d com.apple.quarantine rippling-cli
```

## Usage

> **Disclaimer:** This tool persists access token readable on your disk, so it can run commands without authenticating every time. Use it only on a machine that is always under your control.

Rippling uses a weird client-side password hashing which I do not want to replicate. Thus the only way to make this tool work is to extract your access token from the Rippling web application. This is fairly simple with web inspector. Simply log in into Rippling Web UI, and check in local storage for the access token. It should be valid for a fairly long time. Once you have the token, simply run:
```bash
rippling-cli configure access-token <your-access-token>
```

Afterwards you should be able to use this CLI for around a month from my experience.

See all available commands with `rippling-cli help`:

### Available Commands

```
Usage: rippling-cli <COMMAND>

Commands:
  configure    Configure this client
  guard        Warn about and enforce break and work time limits
  status       Clock-in Status
  clock-in     Clock In
  clock-out    Clock Out
  start-break  Start a break
  end-break    Continue after a break
  manual       Manually add entry for a day
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

#### Command manual

```
Manually add entry for a day

Usage: rippling-cli manual [OPTIONS] [RANGES]...

Arguments:
  [RANGES]...  

Options:
  -d, --days-ago <DAYS_AGO>  Defaults to 0 (today)
  -c, --check                Before submitting check for overlap with holidays, weekends or PTO
  -y, --yes                  Bypass prompt with a yes answer
  -h, --help                 Print help
```

Example: `rippling-cli manual 8:30-17`

Will add an entry from **8:30** to **17:00** with the German statutory breaks in the middle, in this case a 30min break from **12:30** to **13:00**. The statutory break is 30min when working over 6hrs, and 45min when working over 9hrs. The minimum valid break is 15min, so when adding an entry like `8-14:05` it will use a 15min break and not 5min.`

#### Command guard

```
Warn about and enforce break and work time limits

Usage: rippling-cli guard [OPTIONS] --max-break <MAX_BREAK> --max-work <MAX_WORK> --warn <WARN>

Options:
      --max-break <MAX_BREAK>  Maximum length of a break in minutes
      --max-work <MAX_WORK>    Maximum worked time in minutes, excluding breaks
      --warn <WARN>            Minutes before a limit is reached to start warning
      --enforce                End the break or clock out when a limit is exceeded
      --ttl <TTL>              Max age of the cached time entry in seconds. Always refetched before warning or enforcing [default: 300]
  -h, --help                   Print help
```

Checks the current break or the worked time against the given limits. It is meant to be run periodically, e.g. every few minutes via `launchctl` (see below).

* `--max-break` - Maximum length of a break in minutes.
* `--max-work` - Maximum worked time in minutes. Worked time is calculated from the clock-in time and excludes all breaks.
* `--warn` - Minutes before a limit is reached to start warning. Must be smaller than both limits.
* `--enforce` - When a limit is exceeded, end the break or clock out automatically. Without it, an exceeded limit is only printed to stdout.
* `--ttl` - To reduce requests, the current time entry is cached for this many seconds. Before any warning or action the entry is always fetched fresh.

Example: `rippling-cli guard --max-break 60 --max-work 540 --warn 15 --enforce`

* While the break is between 45 and 60min long, it shows a macOS notification on every run to end the break soon.
* Once the break is longer than 60min, it ends the break and shows a notification.
* The same applies to the worked time, with a warning from 8:45h on and a clock-out after 9h.
* When not clocked in, it does nothing.

Notifications are shown with `osascript`, so they appear as coming from *Script Editor*. If you do not see them, check that notifications for Script Editor are allowed in *System Settings → Notifications*.

### Run guard automatically on macOS

`launchctl` can run the guard periodically in the background. Create a launch agent at `~/Library/LaunchAgents/rs.rippling-cli.guard.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>rs.rippling-cli.guard</string>

    <key>ProgramArguments</key>
    <array>
        <string>/Users/YOUR_USER/.cargo/bin/rippling-cli</string>
        <string>guard</string>
        <string>--max-break</string><string>60</string>
        <string>--max-work</string><string>540</string>
        <string>--warn</string><string>15</string>
        <string>--enforce</string>
    </array>

    <!-- Run every 3 minutes -->
    <key>StartInterval</key>
    <integer>180</integer>

    <key>RunAtLoad</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/Users/YOUR_USER/Library/Logs/rippling-guard.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/YOUR_USER/Library/Logs/rippling-guard.log</string>
</dict>
</plist>
```

Replace `YOUR_USER` with your user name and adjust the path to the `rippling-cli` binary if you did not install it with `cargo`. `launchctl` does not expand `~` or `$HOME`, so all paths must be absolute. You may want to leave out `--enforce` at first, to see if the warnings match your expectations.

Load the agent:

```bash
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/rs.rippling-cli.guard.plist
```

Check that it is running and see the last exit code:

```bash
launchctl print gui/$(id -u)/rs.rippling-cli.guard
```

Run it immediately, without waiting for the interval:

```bash
launchctl kickstart -k gui/$(id -u)/rs.rippling-cli.guard
```

`launchctl` does not pick up changes to the plist by itself. After editing it, unload and load the agent again:

```bash
launchctl bootout gui/$(id -u)/rs.rippling-cli.guard
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/rs.rippling-cli.guard.plist
```

Loading an agent that is already loaded fails with `Input/output error`. Updating the `rippling-cli` binary does not require reloading, the next run uses the new version.

Errors, e.g. an expired access token, are written to `~/Library/Logs/rippling-guard.log`.

Things to know:

* `StartInterval` does not run while the Mac is asleep. After waking up, the guard runs once and catches up, but a break that already took too long while sleeping will still be recorded with its full length.
* macOS may show a *Background Items Added* notification when loading the agent. It can be managed in *System Settings → General → Login Items & Extensions*.
