/*
 * lib.rs
 *
 * deepwell-rpc - RPC server to provide database management and migrations
 * Copyright (C) 2019-2020 Ammon Smith
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

//! Client for sending RPC calls to DEEPWELL

#![forbid(unsafe_code)]

extern crate deepwell_core;
extern crate futures;

#[macro_use]
extern crate log;
extern crate serde_json;
extern crate tarpc;
extern crate tokio;
extern crate tokio_serde;

mod api;
mod client;

pub use self::api::{Deepwell as Api, PROTOCOL_VERSION};
pub use self::client::Client;
pub use deepwell_core::error::SendableError;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, SendableError>;
