// Copyright 2021 The BMW Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use tor_util::Error;

use crate::ds_load::DSInfo;

use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Circuit {}

pub fn build_circuit(dsinfo: &DSInfo) -> Result<Circuit, Error> {
	let conn = format!("{}:{}", dsinfo.hosts[3].host, dsinfo.hosts[3].port);

	Ok(Circuit {})
}
