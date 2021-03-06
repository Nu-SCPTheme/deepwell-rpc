/*
 * main.rs
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

//! Server for DEEPWELL via RPC.

#![forbid(unsafe_code)]

extern crate async_std;
extern crate color_backtrace;
extern crate deepwell;
extern crate deepwell_core;

#[macro_use]
extern crate futures;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate ref_map;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate str_macro;
extern crate tarpc;
extern crate tokio;
extern crate tokio_serde;

mod api;
mod async_deepwell;
mod config;
mod server;

use self::async_deepwell::*;
use self::config::Config;
use self::server::Server;
use ref_map::*;
use std::io;

pub use deepwell::{Config as DeepwellConfig, Server as DeepwellServer};
pub use deepwell_core::error::SendableError;

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, SendableError>;

#[tokio::main]
async fn main() -> io::Result<()> {
    color_backtrace::install();

    let Config {
        address,
        log_level,
        database_url,
        revisions_dir,
        password_blacklist,
    } = Config::parse_args();

    pretty_env_logger::formatted_builder()
        .filter_level(log_level)
        .init();

    debug!("Building DEEPWELL server configuration");
    let config = DeepwellConfig {
        database_url: &database_url,
        revisions_dir,
        password_blacklist: password_blacklist.ref_map(|p| p.as_path()),
    };

    info!("Initializing DEEPWELL server");
    let deepwell_server = DeepwellServer::new(config).expect("Unable to start DEEPWELL server");

    let mut deepwell = AsyncDeepwell::new(deepwell_server);
    let send = deepwell.sender();

    info!("Initializing RPC server on {}", address);
    let rpc = Server::init(send);

    // Run both in parallel, return RPC status at end
    join!(rpc.run(address), deepwell.run()).0
}
