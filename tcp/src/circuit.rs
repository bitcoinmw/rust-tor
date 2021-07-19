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
use crate::channel::TorChannel;
use crate::ds_load::DSInfo;
use std::sync::Arc;
use std::sync::Mutex;
use tor_cell::chancell::{msg, ChanCell};
use tor_rtcompat::Runtime;

use tor_util::logger::Log;
use tor_util::Error;

pub struct Circuit {}

pub fn build_circuit(
	dsinfo: &DSInfo,
	mainlog: &'static Arc<Mutex<Log>>,
	runtime: &mut impl Runtime,
) -> Result<Circuit, Error> {
	let tor_channel = Arc::new(Mutex::new(TorChannel::new()));
	let res = build_channel(
		dsinfo.hosts[5].host.clone(),
		dsinfo.hosts[5].port,
		runtime,
		mainlog,
		5_000_000_000,
		tor_channel.clone(),
	);

	match res {
		Err(e) => {
			let mut mainlog = mainlog.lock()?;
			mainlog
				.log(&format!(
					"Error occurred while connecting to channel: {}",
					e
				))
				.map_err(|e| {
					println!("logging error occurred: {}", e);
				})
				.ok();
		}
		Ok(_) => {
			let tor_channel = tor_channel.lock().unwrap();
			let channel = tor_channel.channel.as_ref();
			let error = tor_channel.error.as_ref();

			{
				let mut mainlog = mainlog.lock().unwrap();
				mainlog
					.log(&format!(
						"connect complete chan={:?}, err={:?}",
						channel, error
					))
					.map_err(|e| {
						println!("logging error occurred: {}", e);
					})
					.ok();
			}

			// send a cell
			let cell = ChanCell::new(5.into(), msg::Create2::new(2, &b"abc"[..]).into());
			runtime.block_on(async move {
				let e = channel.unwrap().send_cell(cell).await;
				println!("result of e: {:?}", e);
			});
		}
	}

	Ok(Circuit {})
}
