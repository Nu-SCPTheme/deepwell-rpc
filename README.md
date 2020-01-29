## deepwell-rpc

[![Build Status](https://travis-ci.org/Nu-SCPTheme/deepwell-rpc.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/deepwell-rpc)

An RPC server and client for [DEEPWELL](https://github.com/Nu-SCPTheme/deepwell) calls.
See the relevant crate documentation for more information about what services it provides.

### Compilation
This crate targets the latest stable Rust. At time of writing, that is 1.40.0

```sh
$ cargo build --release
$ cargo run --release -- [arguments] # server
```

If you wish to use its client, import the crate and use it as a library.

### API

The current API provided by the RPC server is as follows:

__Miscellaneous:__

```rust
async fn protocol() -> String;
async fn ping() -> String;
async fn time() -> f64;
```

__Session management:__

```
async fn login(
    username_or_email: String,
    password: String,
    remote_address: Option<String>,
) -> Result<()>;
```
