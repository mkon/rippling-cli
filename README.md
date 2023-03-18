# rippling-cli
Command line interface to Rippling HR.

## Installation

You will need rust ecosystem with `cargo`.

```bash
cargo install --path .
```

## Usage

> **Disclaimer:** This tool persists access token readable on your disk, so it can run commands without authenticating every time. Use it only on a machine that is always under your control.

You can configure your user name to avoid being prompted when authenticating:
```bash
rippling-cli configure username me@example.com
```

Afterwards you can authenticate. Access token will be saved locally and is valid for 30 days usually.
```bash
rippling-cli authenticate
```

Once authenticated you can use the following sub-commands, See all available commands with `rippling-cli help`:

```
Usage: rippling-cli <COMMAND>

Commands:
  configure     Configure this client
  authenticate  Authenticate against rippling
  status        Clock-in Status
  clock-in      Clock In
  clock-out     Clock Out
  start-break   Start a break
  end-break     Continue after a break
  manual        Manually add entry for a day
  mfa           Request MFA
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Command manual

```
Manually add entry for a day

Usage: rippling-cli manual [OPTIONS] [RANGES]...

Arguments:
  [RANGES]...

Options:
  -d, --days-ago <DAYS_AGO>  Defaults to 0 (today)
  -c, --check                Before submitting check for overlap with holidays, weekends or PTO
  -h, --help                 Print help
```

Example: `rippling-cli manual 8:30-17`

Will add an entry from **8:30** to **17:00** with the German statutory breaks in the middle, in this case a 30min break from **12:30** to **13:00**. The statutory break is 30min when working over 6hrs, and 45min when working over 9hrs. The minimum valid break is 15min, so when adding an entry like `8-14:05` it will use a 15min break and not 5min.`

### Command mfa

Sometimes 2FA flow must be completed again to unblock your IP address & API token.

```
rippling-cli mfa request EMAIL

rippling-cli mfa submit EMAIL 123456
```