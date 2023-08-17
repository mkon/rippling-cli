# rippling-cli
Command line interface to Rippling HR.

## Installation

You can either compile it yourself, but it requires rust ecosystem with `cargo`.

```bash
cargo install --path cli
```

Alternatively you can download a binary [release](https://github.com/mkon/rippling-cli/releases). At least on MacOS you probably will have to clear the quarantine flag after downloading to make it executable:

```
xattr -d com.apple.quarantine rippling-cli
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

### Available Commands

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
  mfa           Multi Factor Authentication flows
  help          Print this message or the help of the given subcommand(s)

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

#### Command mfa

```
Multi Factor Authentication flows

Usage: rippling-cli mfa <COMMAND>

Commands:
  token   For use with a token generator like Google Authenticator
  email   Enter the code which will be sent to your email address
  mobile  Enter the code which will be sent to your Phone (SMS)
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

If you enabled MFA in your account you will probably have to use the token flow frequently. Otherwise it usually requires re-running a mfa flow every couple of weeks to unblock your IP address & API token. Try mfa if you get cryptic error messages.