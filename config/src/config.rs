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

use crate::comments::build_toml;
use crate::{Error, ErrorKind};
use clap::load_yaml;
use clap::App;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use toml::Value;
use toml::Value::Table;
const TOR_HOME: &str = ".tor";

const TOML_NAME: &str = "tor.toml";

pub struct TorConfig {
	/// Current dir optional parameter
	pub current_dir: Option<PathBuf>,
	/// Whether to create the directory if it doesn't exist
	pub create_path: bool,
}

// include build information
pub mod built_info {
	include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// create the default toml file if it doesn't already exist
// if it exists, return the toml Value.
pub fn try_create_toml(config: &TorConfig, create_path: bool) -> Result<String, Error> {
	let current_dir = &config.current_dir;

	// check if current directory has a toml
	let toml_location = if current_dir.is_some() {
		let current_dir = current_dir.as_ref().unwrap();
		let mut path_buf = PathBuf::new();
		path_buf.push(current_dir);
		path_buf.push(TOML_NAME);
		path_buf
	} else {
		// use default path
		let mut path_buf = PathBuf::new();
		get_tor_path(config.create_path, &mut path_buf)?;
		path_buf.push(TOML_NAME);
		path_buf
	}
	.into_os_string()
	.into_string()
	.unwrap();

	// create if specified
	if !Path::new(&toml_location).exists() && create_path {
		build_toml(toml_location.clone(), config)?;
	}

	let contents = fs::read_to_string(toml_location)?;

	Ok(contents)
}

/// Get a config object to use for all commands
pub fn get_config() -> Result<TorConfig, Error> {
	// config is based on tor.yml
	let yml = load_yaml!("tor.yml");
	let version = built_info::PKG_VERSION.to_string();

	let args = App::from_yaml(yml)
		.version(built_info::PKG_VERSION)
		.get_matches();

	let config = TorConfig {
		current_dir: None,
		create_path: true,
	};
	Ok(config)
}

/// Update the config object based on the passed in value
fn update_config(config: &mut TorConfig, value: String) -> Result<(), Error> {
	let value = match value.parse::<Value>()? {
		Table(value) => value,
		_ => {
			return Err(ErrorKind::TomlError("Invalid TOML File".to_string()).into());
		}
	};

	Ok(())
}

/// Get the tor path
pub fn get_tor_path(create_path: bool, tor_path: &mut PathBuf) -> Result<(), Error> {
	// Check if bmw dir exists

	match dirs::home_dir() {
		Some(p) => {
			let home_dir_str = p.into_os_string().into_string().unwrap();
			tor_path.push(home_dir_str);
			tor_path.push(TOR_HOME);
		}

		_ => {}
	}

	// Create if the default path doesn't exist
	if !tor_path.exists() && create_path {
		fs::create_dir_all(tor_path.clone())?;
	}

	if !tor_path.exists() {
		Err(ErrorKind::PathNotFoundError(String::from(tor_path.to_str().unwrap())).into())
	} else {
		Ok(())
	}
}
