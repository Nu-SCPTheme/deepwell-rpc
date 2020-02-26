/*
 * deepwell.rs
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

//! Helper struct to keep `deepwell::Server` in a fixed memory position,
//! and use `Send + Sync` future channels to communicate with it.

use crate::StdResult;
use deepwell::Error as DeepwellError;
use deepwell::Server as DeepwellServer;
use deepwell_core::*;
use futures::channel::{mpsc, oneshot};
use futures::prelude::*;
use ref_map::*;

const QUEUE_SIZE: usize = 64;

type DeepwellResult<T> = StdResult<T, DeepwellError>;

macro_rules! send {
    ($response:expr, $result:expr) => {
        match $response.send($result) {
            Ok(_) => trace!("Send response to method receiver"),
            Err(_) => warn!("Method receiver closed, could not send"),
        }
    };
}

#[derive(Debug)]
pub struct AsyncDeepwell {
    server: DeepwellServer,
    recv: mpsc::Receiver<AsyncDeepwellRequest>,
    send: mpsc::Sender<AsyncDeepwellRequest>,
}

impl AsyncDeepwell {
    #[inline]
    pub fn new(server: DeepwellServer) -> Self {
        let (send, recv) = mpsc::channel(QUEUE_SIZE);

        Self { server, recv, send }
    }

    #[inline]
    pub fn sender(&self) -> mpsc::Sender<AsyncDeepwellRequest> {
        mpsc::Sender::clone(&self.send)
    }

    pub async fn run(&mut self) {
        use AsyncDeepwellRequest::*;

        while let Some(request) = self.recv.next().await {
            match request {
                Ping { response, .. } => {
                    debug!("Received Ping request");

                    let result = self.server.ping().await;

                    send!(response, result);
                }
                TryLogin {
                    username_or_email,
                    password,
                    remote_address,
                    response,
                } => {
                    debug!("Received TryLogin request");

                    let result = self
                        .server
                        .try_login(
                            &username_or_email,
                            &password,
                            remote_address.ref_map(|s| s.as_str()),
                        )
                        .await;

                    send!(response, result);
                }
                CheckSession {
                    session_id,
                    user_id,
                    response,
                } => {
                    debug!("Received CheckSession request");

                    let result = self.server.check_session(session_id, user_id).await;
                    send!(response, result);
                }
                Logout {
                    session_id,
                    user_id,
                    response,
                } => {
                    debug!("Received Logout request");

                    let result = self.server.end_session(session_id, user_id).await;
                    send!(response, result);
                }
                LogoutOthers {
                    session_id,
                    user_id,
                    response,
                } => {
                    debug!("Received LogoutOthers request");

                    let result = self.server.end_other_sessions(session_id, user_id).await;
                    send!(response, result);
                }
                CreateUser {
                    name,
                    email,
                    password,
                    response,
                } => {
                    debug!("Received CreateUser request");

                    let result = self.server.create_user(&name, &email, &password).await;
                    send!(response, result);
                }
                EditUser {
                    user_id,
                    changes,
                    response,
                } => {
                    debug!("Received EditUser request");

                    let result = self.server.edit_user(user_id, changes.borrow()).await;
                    send!(response, result);
                }
                GetUserFromId { user_id, response } => {
                    debug!("Received GetUserFromId request");

                    let result = self.server.get_user_from_id(user_id).await;
                    send!(response, result);
                }
                GetUsersFromIds { user_ids, response } => {
                    debug!("Received GetUsersFromIds request");

                    let result = self.server.get_users_from_ids(&user_ids).await;
                    send!(response, result);
                }
                GetUserFromName { name, response } => {
                    debug!("Received GetUserFromName request");

                    let result = self.server.get_user_from_name(&name).await;
                    send!(response, result);
                }
                GetUserFromEmail { email, response } => {
                    debug!("Received GetUserFromEmail request");

                    let result = self.server.get_user_from_email(&email).await;
                    send!(response, result);
                }
            }
        }

        panic!("Receiver stream exhausted");
    }
}

#[derive(Debug)]
pub enum AsyncDeepwellRequest {
    Ping {
        data: (),
        response: oneshot::Sender<DeepwellResult<()>>,
    },
    TryLogin {
        username_or_email: String,
        password: String,
        remote_address: Option<String>,
        response: oneshot::Sender<DeepwellResult<Session>>,
    },
    CheckSession {
        session_id: SessionId,
        user_id: UserId,
        response: oneshot::Sender<DeepwellResult<()>>,
    },
    Logout {
        session_id: SessionId,
        user_id: UserId,
        response: oneshot::Sender<DeepwellResult<()>>,
    },
    LogoutOthers {
        session_id: SessionId,
        user_id: UserId,
        response: oneshot::Sender<DeepwellResult<Vec<Session>>>,
    },
    CreateUser {
        name: String,
        email: String,
        password: String,
        response: oneshot::Sender<DeepwellResult<UserId>>,
    },
    EditUser {
        user_id: UserId,
        changes: UserMetadataOwned,
        response: oneshot::Sender<DeepwellResult<()>>,
    },
    GetUserFromId {
        user_id: UserId,
        response: oneshot::Sender<DeepwellResult<Option<User>>>,
    },
    GetUsersFromIds {
        user_ids: Vec<UserId>,
        response: oneshot::Sender<DeepwellResult<Vec<Option<User>>>>,
    },
    GetUserFromName {
        name: String,
        response: oneshot::Sender<DeepwellResult<Option<User>>>,
    },
    GetUserFromEmail {
        email: String,
        response: oneshot::Sender<DeepwellResult<Option<User>>>,
    },
}
