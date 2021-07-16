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

use tor_config::config::get_config;
use tor_tcp::ds_load::{build_ds_context, get_latest_valid_dsinfo, start_dsinfo_refresh_thread};
use tor_util as util;
use util::logger::Log;
use util::Error;
use util::StopState;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

fn main() {
	let exit_code = real_main();
	std::process::exit(exit_code);
}

fn real_main() -> i32 {
	match main_with_result() {
		Ok(_) => 0,
		Err(e) => {
			println!("Startup Error: {}", e);
			-1
		}
	}
}

fn main_with_result() -> Result<(), Error> {
	let stop_state = Arc::new(RwLock::new(StopState::new()));
	let config = get_config()?;
	let mainlog = Arc::new(Mutex::new(Log::new(
		"./logs/mainlog.log",
		100,
		1000 * 60,
		true,
		"header_url_ABC_DEF",
	)?));
	let ds_context = build_ds_context(&config)?;
	let ds_info = get_latest_valid_dsinfo(&config, &ds_context)?;
	start_dsinfo_refresh_thread(&config, stop_state.clone(), mainlog.clone())?;

	println!("ds_infohostlen={}", ds_info.hosts.len());

	println!("config={:?}", config);
	println!("config.version = {}", config.version);
	/*
		let context = build_connector_context(20, 20, 20);
		let response = do_get(
			"http://86.59.21.38/tor/status-vote/current/consensus/",
			context,
		)?;

		println!("response={}", response);
	*/

	let mut count = 0;
	loop {
		println!("logging");
		{
			let mut mainlog = mainlog.lock()?;
			(*mainlog).log("test1")?;
			(*mainlog).log("test2")?;
		}
		std::thread::sleep(std::time::Duration::from_secs(5));
		count += 1;
		if count >= 3000 {
			break;
		}
	}

	{
		let stop_state = stop_state.write()?;
		stop_state.stop();
	}
	std::thread::sleep(std::time::Duration::from_secs(5));
	Ok(())
}
