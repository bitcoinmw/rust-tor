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
use std::fs::{canonicalize, metadata};
use std::path::Path;
use std::path::PathBuf;
use toml::Value;
use toml::Value::Table;

/// the default Tor directory (we use .tor2 not to collide with .tor)
const TOR_HOME: &str = ".tor2";
/// The default name for the Tor toml config file
const TOML_NAME: &str = "tor.toml";

/// This is the main configuration file for tor
#[derive(Debug)]
pub struct TorConfig {
	/// Location of the config file
	pub config_file: String,
	/// Version of the configuration file
	pub version: String,
	/// Directory Servers
	pub directory_servers: Vec<String>,
}

// include build information
pub mod built_info {
	include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// create the default toml file if it doesn't already exist
// return the toml Value.
pub fn try_create_toml(config: &TorConfig) -> Result<String, Error> {
	// create if specified
	if !Path::new(&config.config_file).exists() || metadata(&config.config_file)?.len() == 0 {
		build_toml(config)?;
	}
	let contents = fs::read_to_string(&config.config_file)?;
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

	let config_file = if args.is_present("config") {
		// if config specified use value passed in
		let file_name = args.value_of("config").unwrap().to_string();
		// we have to create it to use canonicalize
		if !fsutils::path_exists(&file_name) {
			fsutils::create_file(&file_name);
		}
		// return canonicalized path
		canonicalize(PathBuf::from(file_name))?
			.into_os_string()
			.into_string()
			.unwrap()
	} else {
		// use default, not specified
		let mut config_path = PathBuf::new();
		match dirs::home_dir() {
			Some(p) => {
				let home_dir_str = p.into_os_string().into_string().unwrap();
				config_path.push(home_dir_str);
				config_path.push(TOR_HOME);
			}
			_ => {
				config_path.push(TOR_HOME);
			}
		}
		// mkdir for default if it doesn't exist
		fsutils::mkdir(&config_path.clone().into_os_string().into_string().unwrap());
		config_path.push(TOML_NAME);
		let path = &config_path.clone().into_os_string().into_string().unwrap();
		// create the file if it's not there
		if !fsutils::path_exists(path) {
			fsutils::create_file(path);
		}
		let config_path = canonicalize(config_path)?;
		config_path.into_os_string().into_string().unwrap()
	};

	let directory_servers = vec![];

	// build preliminary tor config
	let mut config = TorConfig {
		config_file,
		version,
		directory_servers,
	};

	// try to get it, if not there, create it
	let toml_text = try_create_toml(&config)?;
	// update config based on toml file values
	update_config(&mut config, toml_text)?;
	Ok(config)
}

/// Update the config object based on the passed in values from config file
fn update_config(config: &mut TorConfig, value: String) -> Result<(), Error> {
	let value = match value.parse::<Value>()? {
		Table(value) => value,
		_ => {
			return Err(ErrorKind::TomlError("Invalid TOML File".to_string()).into());
		}
	};

	// make sure there's a general section
	let general = value.get("general");
	let general = match general {
		Some(general) => general,
		None => {
			return Err(
				ErrorKind::TomlError("general section must be specified".to_string()).into(),
			)
		}
	};

	// get the version
	config.version = match general.get("version") {
		Some(version) => match version.as_str() {
			Some(version) => version.to_string(),
			None => {
				return Err(
					ErrorKind::TomlError("general.version must be a string".to_string()).into(),
				)
			}
		},
		None => {
			return Err(
				ErrorKind::TomlError("general.version must be specified".to_string()).into(),
			)
		}
	};

	config.directory_servers = match general.get("directory_servers") {
		Some(ds) => match ds.as_array() {
			Some(ds) => {
				let mut ret = vec![];
				for s in ds {
					let value = s.as_str();
					if value.is_none() {
						return Err(ErrorKind::TomlError(
							"general.directory_servers must be an array of strings".to_string(),
						)
						.into());
					}
					ret.push(value.unwrap().to_string());
				}
				ret
			}
			None => {
				return Err(ErrorKind::TomlError(
					"general.directory_servers must be an array".to_string(),
				)
				.into());
			}
		},
		None => {
			return Err(ErrorKind::TomlError("general.version must be a string".to_string()).into())
		}
	};

	Ok(())
}
