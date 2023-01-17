# rippling-cli
Command line interface to Rippling HR.

## Installation

You will need rust ecosystem with `cargo`.

```bash
cargo install --path .
```

## Usage

> **Disclaimer:** This tool persists access token readable on your disk, so it can run commands without authenticating every time. So it is not recommended to use it on public computers.

You can configure your user name to avoid future prompts:
```bash
rippling-cli configure username me@example.com
```

Afterwards you can authenticate. Access token will be saved locally and is valid for 30 days usually.
```bash
rippling-cli authenticate
```

Once authenticated you can use the following sub-commands:
* `status`: Current clock in status
* `clock-in` (`ci`): Start the clock

See all available commands with `rippling-cli help`.
