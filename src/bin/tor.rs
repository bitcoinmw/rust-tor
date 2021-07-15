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
use tor_util as util;
use util::Error;
//use util::http::{build_connector_context, do_get};
use tor_tcp::ds_load::build_ds_context;
use util::logger::Log;

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
	let config = get_config()?;
	let ds_context = build_ds_context(&config)?;

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

	let mut log = Log::new(
		"./test.log",
		true,
		100,
		1000 * 60,
		true,
		"header_url_ABC_DEF",
	)?;

	let mut count = 0;
	loop {
		println!("logging");
		log.log("test1")?;
		log.log("test2")?;
		std::thread::sleep(std::time::Duration::from_secs(5));
		count += 1;
		if count >= 1_000_000 {
			break;
		}
	}

	Ok(())
}
