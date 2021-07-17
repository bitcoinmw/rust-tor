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

use crate::config::TorConfig;
use crate::{Error, ErrorKind};
use std::fs::File;
use std::io::Write;

/// This is the default TOML file. It's modified based passed in params.
const DEFAULT_TOML: &str = "\
\n\
##############################################################################\n\
### TOR CONFIGURATION                                                      ###\n\
##############################################################################\n\
[general]\n\
\n\
##############################################################################\n\
### GENERAL CONFIGURATION                                                  ###\n\
##############################################################################\n\
\n\
version = \"REPLACE_VERSION\"\n\
\n\
db_root = \"REPLACE_DB_ROOT\"\n\
\n\
directory_servers = [\n\
\t\"45.66.33.45\",\n\
\t\"66.111.2.131\",\n\
\t\"128.31.0.34\",\n\
\t\"86.59.21.38\",\n\
\t\"204.13.164.118\",\n\
\t\"171.25.193.9\",\n\
\t\"193.23.244.244\",\n\
\t\"154.35.175.225\",\n\
\t\"131.188.40.189\",\n\
\t\"199.58.81.140\",\n\
]\n\
\n\
# Two weeks\n\
ds_refresh_timeout = 1209600000\n\
\n\
# Ten minutes\n\
ds_refresh_frequency = 600000\n\
\n\
[logging]\n\
\n\
##############################################################################\n\
### LOGGING CONFIGURATION                                                  ###\n\
##############################################################################\n\
\n\
# 1 hour\n\
mainlog_rotationtime = 3600000\n\
\n\
# 10 mb\n\
mainlog_rotationsize = 10485760\n\
\n\
";

/// This function builds the toml file based on the TorConfig argument
/// The toml file is saved in the specified location
pub fn build_toml(config: &TorConfig) -> Result<(), Error> {
	let toml_location = &config.config_file;
	let file = File::create(toml_location.clone());
	// make sure we can create the file.
	match file {
		Ok(mut file) => {
			let toml = DEFAULT_TOML
				.replace("REPLACE_VERSION", &config.version)
				.replace("REPLACE_DB_ROOT", &config.db_root);
			// write file
			file.write_all(toml.as_bytes())?;

			Ok(())
		}
		Err(_) => Err(ErrorKind::PathNotFoundError(format!(
			"directory: {} doesn't exist. Perhaps create it first.",
			toml_location
		))
		.into()),
	}
}
