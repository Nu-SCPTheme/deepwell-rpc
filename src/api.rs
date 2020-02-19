/*
 * api.rs
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

use crate::Result;
use deepwell_core::*;

pub const PROTOCOL_VERSION: &str = "0";

#[tarpc::service]
pub trait Deepwell {
    // Misc
    async fn protocol() -> String;
    async fn ping() -> String;
    async fn time() -> f64;

    // Session
    async fn login(
        username_or_email: String,
        password: String,
        remote_address: Option<String>,
    ) -> Result<Session>;

    async fn logout(session_id: SessionId, user_id: UserId) -> Result<()>;
    async fn logout_others(session_id: SessionId, user_id: UserId) -> Result<Vec<Session>>;
    async fn check_session(session_id: SessionId, user_id: UserId) -> Result<()>;

    // User
    async fn create_user(name: String, email: String, password: String) -> Result<UserId>;
    async fn edit_user(user_id: UserId, changes: UserMetadataOwned) -> Result<()>;
    async fn get_user_from_id(user_id: UserId) -> Result<Option<User>>;
    async fn get_users_from_ids(user_ids: Vec<UserId>) -> Result<Vec<Option<User>>>;
    async fn get_user_from_name(name: String) -> Result<Option<User>>;
    async fn get_user_from_email(email: String) -> Result<Option<User>>;

    // TODO
}
