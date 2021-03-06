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

use iron::{Request, Response, IronResult};
use iron::status;
use persistent::Read;
use plugin::Pluggable;
use params::{Params, Value};

use ::web::server::{ConnectionPoolKey, StorageKey};
use ::web::backend::db::{PackageVersion, User};
use ::utils::error::BackendError;

header! { (XNugetApiKey, "X-NuGet-ApiKey") => [String] }

pub fn upload(req: &mut Request) -> IronResult<Response> {
    let params = req.get_ref::<Params>().unwrap().clone();

    let apikey = req.headers.get::<XNugetApiKey>().cloned().unwrap().0;

    let storage = req.extensions.get::<Read<StorageKey>>().unwrap();
    let connection_pool = req.extensions.get::<Read<ConnectionPoolKey>>().unwrap();
    let connection = match connection_pool.get() {
        Ok(connection) => connection,
        Err(x) => {
            print!("{:?}", x);
            return Ok(Response::with((status::InternalServerError, "Database Error, please try again later")));
        }
    };

    match User::get_by_apikey(&*connection, &apikey) {
        Ok(user) => {
            match params.find(&["package"]) {
                Some(&Value::File(ref file)) => {
                    match PackageVersion::new(&*connection, &user, storage, file.open().unwrap()) {
                        Ok(_) => Ok(Response::with(status::Ok)),
                        Err(BackendError::PermissionDenied) => Ok(Response::with((status::Forbidden, "Only the maintainer or admin is allowed to update a package"))),
                        Err(err) => {
                            error!("{}", err);
                            Ok(Response::with((status::BadRequest, format!("{}", err))))
                        },
                    }
                },
               _ => Ok(Response::with((status::BadRequest, "package is no File"))),
            }
        },
        //TODO better match
        Err(_) => Ok(Response::with((status::InternalServerError, "No User with matching API-Key found"))),
    }
}
