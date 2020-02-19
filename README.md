## deepwell-rpc

[![Build Status](https://travis-ci.org/Nu-SCPTheme/deepwell-rpc.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/deepwell-rpc)

An RPC server and client for [DEEPWELL](https://github.com/Nu-SCPTheme/deepwell) calls.
See the relevant crate documentation for more information about what services it provides.

The lint `#![forbid(unsafe_code)]` is set, and therefore this crate has only safe code. However dependencies may have `unsafe` internals.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).

### Compilation
This crate targets the latest stable Rust. At time of writing, that is 1.41.0

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
) -> Result<Session>;

/// Ends a user session, using the given session and user IDs.
async fn logout(session_id: SessionId, user_id: UserId) -> Result<()>;

/// Ends all sessions other than the current one for the current user.
/// Ensures that the given session ID is valid for the user.
/// Returns a list of sessions invalidated by this action.
async fn logout_others(session_id: SessionId, user_id: UserId) -> Result<Vec<Session>>;

/// Checks if the given session is currently valid for the current user.
async fn check_session(session_id: SessionId, user_id: UserId) -> Result<()>;
```

__User:__

```rust
/// Creates a new user with the given name, email, and password.
///
/// The username and email are checked for case-insensitive uniqueness among existing
/// users, and the password is checked against the configured blacklist of weak or common
/// passwords.
///
/// If successful, the user ID of the new user is returned.
async fn create_user(name: String, email: String, password: String) -> Result<UserId>;

/// Modifies the properties of a user, including name and email address.
/// If the email is modified it will need to be re-verified.
async fn edit_user(user_id: UserId, changes: UserMetadataOwned) -> Result<()>;

/// Retrieves information about a user from their ID.
async fn get_user_from_id(user_id: UserId) -> Result<Option<User>>;

/// Retrieves information about the given users by ID.
/// Returns users in the same order as the specified IDs.
/// If an ID is invalid that instance is `None`.
///
/// Can only fetch information from 100 users at once.
async fn get_users_from_ids(user_ids: Vec<UserId>) -> Result<Vec<Option<User>>>;

/// Retrieves information about a user from their username.
/// Returns `None` if no user with that username is found.
/// Searches case-insensitively.
async fn get_user_from_name(name: String) -> Result<Option<User>>;

/// Retrieves information about a user from their email.
/// Returns `None` if no user with that username is found.
/// Searches case-insensitively.
async fn get_user_from_email(email: String) -> Result<Option<User>>;
```

### Server Execution

If you want to expose a new RPC method, a few changes are needed. Firstly, the RPC prototype in `api.rs` must be adjusted.

Once that is set, you implement client and server calls for it in in `client.rs` and `server.rs` respectively. The client
call is a simple pass-through for the generated tarpc method. Somewhat similarly, the server call proxies to the corresponding
DEEPWELL method.

However, because `deepwell::Server` is not thread-safe, it is not actually kept in the tarpc instance. Instead it is run in
a separate async task, with tasks fed into it via an enum in a provided input channel. Each request passes in a onceshot
output channel, which is then awaited to get the result.
