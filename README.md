# rippling-cli
Command line interface to Rippling HR.

## Installation

You will need rust ecosystem with `cargo`.

```bash
cargo install --path .
```

## Usage

First you need to configure client id & secret. Both id and secret can usually be found in the html code of the login page.
```bash
rippling-cli configure client-id some-client-id
rippling-cli configure client-secret some-client-secret
```

Afterwards you can authenticate. Access token will be saved locally and is valid for 30 days usually.
```bash
rippling-cli authenticate
```