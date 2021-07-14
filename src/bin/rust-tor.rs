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

use rust_tor_error::Error;
use rust_tor_util::http::{build_connector_context, do_get};

fn main() {
	let exit_code = real_main();
	std::process::exit(exit_code);
}

fn real_main() -> i32 {
	match main_with_result() {
		Ok(_) => 0,
		Err(e) => {
			println!("Error: {}", e);
			-1
		}
	}
}

fn main_with_result() -> Result<(), Error> {
	let context = build_connector_context(20, 20, 20);
	let response = do_get(
		"http://86.59.21.38/tor/status-vote/current/consensus/",
		context,
	)?;

	println!("response={}", response);

	Ok(())
}
