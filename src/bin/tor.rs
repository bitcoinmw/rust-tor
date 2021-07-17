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

use tor_config::config::{get_config, TorConfig};
use tor_tcp::circuit::build_circuit;
use tor_tcp::ds_load::{build_ds_context, get_latest_valid_dsinfo, start_dsinfo_refresh_thread};
use tor_util as util;
use util::logger::Log;
use util::Error;
use util::StopState;

use chrono::prelude::DateTime;
use chrono::Local;
use chrono::Utc;
use num_format::{Locale, ToFormattedString};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

// include build information
pub mod built_info {
	include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

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

fn show_param(key: &str, value: &str, mainlog: Arc<Mutex<Log>>) -> Result<(), Error> {
	let mut mainlog = mainlog.lock()?;
	(*mainlog).log(&format!("{:21}: '{}'", key, value,))?;

	Ok(())
}

fn print_config(config: &TorConfig, mainlog: Arc<Mutex<Log>>) -> Result<(), Error> {
	{
		let mut mainlog = mainlog.lock()?;
		(*mainlog).log(
			"\
------------------------------------------------------------------------------",
		)?;
		(*mainlog).log(&format!(
			"Starting Rust Tor Daemon version: {}",
			built_info::PKG_VERSION.to_string()
		))?;
		(*mainlog).log(
			"\
------------------------------------------------------------------------------",
		)?;
	}
	show_param("config_file", &config.config_file, mainlog.clone())?;

	show_param("config file version", &config.version, mainlog.clone())?;

	show_param("db_root", &config.db_root, mainlog.clone())?;

	show_param(
		"directory_servers.len",
		&format!("{}", &config.directory_servers.len()),
		mainlog.clone(),
	)?;

	show_param(
		"ds_refresh_timeout",
		&format!(
			"{} ms",
			&config.ds_refresh_timeout.to_formatted_string(&Locale::en)
		),
		mainlog.clone(),
	)?;

	show_param(
		"ds_refresh_frequency",
		&format!(
			"{} ms",
			&config.ds_refresh_frequency.to_formatted_string(&Locale::en)
		),
		mainlog.clone(),
	)?;

	show_param("mainlog", &config.mainlog, mainlog.clone())?;

	show_param(
		"mainlog_rotationsize",
		&format!(
			"{} bytes",
			&config.mainlog_rotationsize.to_formatted_string(&Locale::en)
		),
		mainlog.clone(),
	)?;

	show_param(
		"mainlog_rotationtime",
		&format!(
			"{} ms",
			&config.mainlog_rotationtime.to_formatted_string(&Locale::en)
		),
		mainlog.clone(),
	)?;

	show_param(
		"print debugging info",
		if config.debug { "ON" } else { "OFF" },
		mainlog.clone(),
	)?;

	let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
	let d = UNIX_EPOCH + Duration::from_secs(timestamp);
	let datetime = DateTime::<Utc>::from(d).with_timezone(&Local);
	let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S %z").to_string();

	{
		let mut mainlog = mainlog.lock()?;
		(*mainlog).log(
			"\
------------------------------------------------------------------------------",
		)?;
		(*mainlog).log(&format!("Rust Tor Daemon started at: {}", timestamp_str,))?;
		(*mainlog).log(
			"\
------------------------------------------------------------------------------",
		)?
	}

	Ok(())
}

fn main_with_result() -> Result<(), Error> {
	let stop_state = Arc::new(RwLock::new(StopState::new()));
	let config = get_config()?;
	let mainlog = Arc::new(Mutex::new(Log::new(
		&config.mainlog,
		config.mainlog_rotationsize,
		config.mainlog_rotationtime.into(),
		false,
		"MainLog - Tor (Rust)\n\
------------------------------------------------------------------------------",
	)?));

	print_config(&config, mainlog.clone())?;

	let ds_context = build_ds_context(&config)?;
	let ds_info = get_latest_valid_dsinfo(&config, &ds_context)?;
	build_circuit(&ds_info)?;
	start_dsinfo_refresh_thread(&config, stop_state.clone(), mainlog.clone())?;

	{
		let mut mainlog = mainlog.lock()?;
		if !config.debug {
			(*mainlog).update_show_stdout(false)?;
		}
		(*mainlog).update_show_timestamp(true)?;

		(*mainlog).log(&format!("Found {} hosts.", ds_info.hosts.len()))?;
		for i in 0..10 {
			(*mainlog).log(&format!("host[{}]={:?}.", i, ds_info.hosts[i]))?;
		}
	}

	let mut count = 0;
	loop {
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
