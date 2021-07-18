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

use crate::channel::build_channel;
use crate::ds_load::DSInfo;
use std::sync::Arc;
use std::sync::Mutex;
use tor_rtcompat::Runtime;
use tor_util::logger::Log;
use tor_util::Error;

pub struct Circuit {}

pub fn build_circuit(
	dsinfo: &DSInfo,
	mainlog: &Arc<Mutex<Log>>,
	runtime: &mut impl Runtime,
) -> Result<Circuit, Error> {
	let _channel = build_channel(
		dsinfo.hosts[3].host.clone(),
		dsinfo.hosts[3].port,
		runtime,
		mainlog,
	);
	Ok(Circuit {})
}
