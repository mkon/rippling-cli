# rippling-cli
Command line interface to Rippling HR.

## Installation

You will need rust ecosystem with `cargo`.

```bash
cargo install --path .
```

## Usage

> **Disclaimer:** This tool persists access token readable on your disk, so it can run commands without authenticating every time. Use it only on a machine that is always under your control.

You can configure your user name to avoid future prompts:
```bash
rippling-cli configure username me@example.com
```

Afterwards you can authenticate. Access token will be saved locally and is valid for 30 days usually.
```bash
rippling-cli authenticate
```

Once authenticated you can use the following sub-commands:
* `status`: Current clock in status.
* `clock-in` (`in`): Start tracking.
* `clock-out` (`out`): Stop tracking.
* `start-break` (`sb` | `break`): Start a break.
* `end-break` (`eb` | `continue`): End the break.
* `today`: Manual add an entry.

See all available commands with `rippling-cli help`.

### Today

Example: `rippling-cli today 8-17:30`

Will add an entry from **8:00** to **17:30** with the German statutory breaks in the middle, in this case a 30min break from **12:30** to **13:00**. The statutory break is 30min when working over 6hrs, and 45min when working over 9hrs. The minimum valid break is 15min, so when adding an entry like `8-14:05` it will use a 15min break and not 5min.