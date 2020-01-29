## deepwell-rpc

[![Build Status](https://travis-ci.org/Nu-SCPTheme/deepwell-rpc.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/deepwell-rpc)

An RPC server and client for [DEEPWELL](https://github.com/Nu-SCPTheme/deepwell) calls.
See the relevant crate documentation for more information about what services it provides.

The lint `#![forbid(unsafe_code)]` is set, and therefore this crate has only safe code. However dependencies may have unsafe internals.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).

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
/// Returns the static protocol version. Currently "0".
async fn protocol() -> String;

/// Determines if the server is reachable.
async fn ping() -> String;

/// Returns the system time on the server.
/// It may be in any timezone and is not monotonic.
async fn time() -> f64;
```

__Session management:__

```rust
/// Begin a user session, using the given username/email and password.
/// If known, `remote_address` refers to the client making the request.
async fn login(
    username_or_email: String,
    password: String,
    remote_address: Option<String>,
) -> Result<()>;
```
