## deepwell-rpc

[![Build Status](https://travis-ci.org/Nu-SCPTheme/deepwell-rpc.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/deepwell-rpc)

An RPC server and client for [DEEPWELL](https://github.com/Nu-SCPTheme/deepwell) calls.
See the relevant crate documentation for more information about what services it provides.

The lint `#![forbid(unsafe_code)]` is set, and therefore this crate has only safe code. However dependencies may have `unsafe` internals.

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

### Server Execution

If you want to expose a new RPC method, a few changes are needed. Firstly, the RPC prototype in `api.rs` must be adjusted.

Once that is set, you implement client and server calls for it in in `client.rs` and `server.rs` respectively. The client
call is a simple pass-through for the generated tarpc method. Somewhat similarly, the server call proxies to the corresponding
DEEPWELL method.

However, because `deepwell::Server` is not thread-safe, it is not actually kept in the tarpc instance. Instead it is run in
a separate async task, with tasks fed into it via an enum in a provided input channel. Each request passes in a onceshot
output channel, which is then awaited to get the result.
