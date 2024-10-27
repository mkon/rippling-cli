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
