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

use iron::{Url, BeforeMiddleware, IronError, IronResult, Request};

pub struct PathNormalizer;
impl BeforeMiddleware for PathNormalizer
{
    fn before(&self, req: &mut Request) -> IronResult<()>
    {
        let mut url = req.url.clone().into_generic_url();
        url.path_segments_mut().unwrap().pop_if_empty();
        req.url = Url::from_generic_url(url).unwrap();
        Ok(())
    }
    fn catch(&self, _: &mut Request, _: IronError) -> IronResult<()>
    {
        Ok(())
    }
}
