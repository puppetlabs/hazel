// Copyright (C) 2016  Max Planck Institute for Human Development
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use iron::{Request, Response, IronResult, Plugin};
use iron::headers::SetCookie;
use iron::modifiers::Header;
use cookie::*;
use iron::status::{self, Status};
use persistent::{Read, Write};
use uuid::Uuid;
use params::{Params, Value};
use chrono::*;

use std::str::FromStr;

use ::web::backend::db::User;
use ::web::server::{ConnectionPoolKey, SessionStoreKey};
use ::utils::CONFIG;
use ::utils::middleware::SessionInfo;

pub fn login(req: &mut Request) -> IronResult<Response> {

    let params = req.get_ref::<Params>().unwrap().clone();

    let username = match params.find(&["username"]) {
        Some(&Value::String(ref name)) => name.clone(),
        _ => return Ok(Response::with(Status::BadRequest)),
    };
    let password = match params.find(&["password"]) {
        Some(&Value::String(ref pass)) => pass.clone(),
        _ => return Ok(Response::with(Status::BadRequest)),
    };
    let remember = match params.find(&["remember"]) {
        Some(&Value::String(ref rem)) => match bool::from_str(rem) {
            Ok(boolean) => boolean,
            Err(_) => return Ok(Response::with(Status::BadRequest)),
        },
        _ => false
    };

    let connection_pool = req.extensions.get::<Read<ConnectionPoolKey>>().unwrap();
    let connection = match connection_pool.get() {
        Ok(connection) => connection,
        Err(err) => {
            error!("{:?}", err);
            return Ok(Response::with((status::InternalServerError, "Database Error, please try again later")));
        }
    };

    match User::login(&*connection, &username, &password) {
        Ok(success) => {
            if success {
                let session_store_mutex = req.extensions.get::<Write<SessionStoreKey>>().unwrap().clone();
                let mut session_store = session_store_mutex.lock().unwrap();
                let session_id = Uuid::new_v4().simple().to_string();
                let session_info = SessionInfo {
                    expires: match remember {
                        true  => UTC::now() + Duration::weeks(1),
                        false => UTC::now() + Duration::hours(1),
                    },
                    session_id: session_id.clone(),
                    remember: remember,
                };
                let root_jar = CookieJar::new(&*CONFIG.auth.cookie_key.as_bytes());
                let jar = root_jar.encrypted();
                let mut user_cookie = Cookie::new(String::from("hazel_username"), username.clone());
                let mut session_cookie = Cookie::new(String::from("hazel_sessionid"), session_id);

                session_cookie.max_age = Some((session_info.expires - UTC::now()).num_seconds() as u64);
                user_cookie.max_age = Some((session_info.expires - UTC::now()).num_seconds() as u64);
                session_cookie.path = Some(String::from("/"));
                user_cookie.path = Some(String::from("/"));
                session_cookie.domain = Some(req.url.host().to_string());
                user_cookie.domain = Some(req.url.host().to_string());

                jar.add(user_cookie);
                jar.add(session_cookie);

                session_store.insert(username, session_info);
                Ok(Response::with((Status::Ok, Header(SetCookie::from_cookie_jar(&root_jar)), "success")))
            } else {
                Ok(Response::with(Status::Unauthorized))
            }
        },
        Err(_) => {
            Ok(Response::with((Status::Unauthorized)))
        }
    }
}
