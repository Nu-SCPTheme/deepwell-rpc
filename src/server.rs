/*
 * server.rs
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

use crate::api::{Deepwell as DeepwellApi, PROTOCOL_VERSION};
use crate::async_deepwell::AsyncDeepwellRequest;
use crate::Result;
use deepwell_core::*;
use futures::channel::{mpsc, oneshot};
use futures::future::{self, BoxFuture, Ready};
use futures::prelude::*;
use std::io;
use std::net::SocketAddr;
use std::time::SystemTime;
use tarpc::context::Context;
use tarpc::serde_transport::tcp;
use tarpc::server::{BaseChannel, Channel};
use tokio_serde::formats::Json;

// Prevent network socket exhaustion or related slowdown
const MAX_PARALLEL_REQUESTS: usize = 16;

macro_rules! forward {
    ($self:expr, $request:tt, [ $($field:ident),* , ] ) => {
        forward!($self, $request, [ $($field),* ])
    };

    ($self:expr, $request:tt, [ $($field:ident),* ] ) => {{
        let fut = async move {
            let (send, recv) = oneshot::channel();

            // Build request enum
            let request = AsyncDeepwellRequest::$request {
                $($field),*,
                response: send,
            };

            // Send to process
            $self.channel
                .send(request)
                .await
                .expect("Deepwell server channel closed");

            // Wait for result to arrive
            recv.await
                .expect("Oneshot closed before result")
                .map_err(|e| e.to_sendable())
        };

        fut.boxed()
    }};
}

#[derive(Debug, Clone)]
pub struct Server {
    channel: mpsc::Sender<AsyncDeepwellRequest>,
}

impl Server {
    #[inline]
    pub fn init(channel: mpsc::Sender<AsyncDeepwellRequest>) -> Self {
        Self { channel }
    }

    pub async fn run(&self, address: SocketAddr) -> io::Result<()> {
        tcp::listen(&address, Json::default)
            .await?
            // Log requests
            .filter_map(|conn| {
                async move {
                    match conn {
                        // Note incoming connection
                        Ok(conn) => {
                            match conn.peer_addr() {
                                Ok(addr) => info!("Accepted connection from {}", addr),
                                Err(error) => warn!("Unable to get peer address: {}", error),
                            }

                            Some(conn)
                        }
                        // Unable to accept connection
                        Err(error) => {
                            warn!("Error accepting connection: {}", error);

                            None
                        }
                    }
                }
            })
            // Create and fulfill channels for each request
            .map(BaseChannel::with_defaults)
            .map(|chan| {
                let resp = self.clone().serve();
                chan.respond_with(resp).execute()
            })
            .buffer_unordered(MAX_PARALLEL_REQUESTS)
            .for_each(|_| async {})
            .await;

        Ok(())
    }
}

impl DeepwellApi for Server {
    // Misc

    type ProtocolFut = Ready<String>;

    #[inline]
    fn protocol(self, _: Context) -> Self::ProtocolFut {
        info!("Method: protocol");

        future::ready(str!(PROTOCOL_VERSION))
    }

    type PingFut = Ready<String>;

    #[inline]
    fn ping(self, _: Context) -> Self::PingFut {
        info!("Method: ping");

        future::ready(str!("pong!"))
    }

    type TimeFut = Ready<f64>;

    #[inline]
    fn time(self, _: Context) -> Self::TimeFut {
        info!("Method: time");

        let now = SystemTime::now();
        let unix_time = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time before epoch")
            .as_secs_f64();

        future::ready(unix_time)
    }

    // Sessions
    type LoginFut = BoxFuture<'static, Result<Session>>;

    fn login(
        mut self,
        _: Context,
        username_or_email: String,
        password: String,
        remote_address: Option<String>,
    ) -> Self::LoginFut {
        info!("Method: login");

        forward!(self, TryLogin, [
            username_or_email,
            password,
            remote_address,
        ])
    }

    type LogoutFut = BoxFuture<'static, Result<()>>;

    fn logout(mut self, _: Context, session_id: SessionId, user_id: UserId) -> Self::LogoutFut {
        info!("Method: logout");

        forward!(self, Logout, [
            session_id,
            user_id,
        ])
    }

    type LogoutOthersFut = BoxFuture<'static, Result<Vec<Session>>>;

    fn logout_others(
        mut self,
        _: Context,
        session_id: SessionId,
        user_id: UserId,
    ) -> Self::LogoutOthersFut {
        info!("Method: logout_others");

        forward!(self, LogoutOthers, [
            session_id,
            user_id,
        ])
    }

    type CheckSessionFut = BoxFuture<'static, Result<()>>;

    fn check_session(
        mut self,
        _: Context,
        session_id: SessionId,
        user_id: UserId,
    ) -> Self::CheckSessionFut {
        info!("Method: check_session");

        forward!(self, CheckSession, [
            session_id,
            user_id,
        ])
    }

    type CreateUserFut = BoxFuture<'static, Result<UserId>>;

    fn create_user(mut self, _: Context, name: String, email: String, password: String) -> Self::CreateUserFut {
        info!("Method: create_user");

        forward!(self, CreateUser, [
            name,
            email,
            password,
        ])
    }

    type EditUserFut = BoxFuture<'static, Result<()>>;

    fn edit_user(mut self, _: Context, user_id: UserId, changes: UserMetadataOwned) -> Self::EditUserFut {
        info!("Method: edit_user");

        forward!(self, EditUser, [
            user_id,
            changes,
        ])
    }

    type GetUserFromIdFut = BoxFuture<'static, Result<Option<User>>>;

    fn get_user_from_id(mut self, _: Context, user_id: UserId) -> Self::GetUserFromIdFut {
        info!("Method: get_user_from_id");

        forward!(self, GetUserFromId, [user_id])
    }

    type GetUsersFromIdsFut = BoxFuture<'static, Result<Vec<Option<User>>>>;

    fn get_users_from_ids(mut self, _: Context, user_ids: Vec<UserId>) -> Self::GetUsersFromIdsFut {
        info!("Method: get_users_from_ids");

        forward!(self, GetUsersFromIds, [user_ids])
    }

    type GetUserFromNameFut = BoxFuture<'static, Result<Option<User>>>;

    fn get_user_from_name(mut self, _: Context, name: String) -> Self::GetUserFromNameFut {
        info!("Method: get_user_from_name");

        forward!(self, GetUserFromName, [name])
    }

    type GetUserFromEmailFut = BoxFuture<'static, Result<Option<User>>>;

    fn get_user_from_email(mut self, _: Context, email: String) -> Self::GetUserFromEmailFut {
        info!("Method: get_user_from_email");

        forward!(self, GetUserFromEmail, [email])
    }

    // TODO
}
