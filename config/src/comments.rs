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
version = \"REPLACE_VERSION\"\n\
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
			let toml = DEFAULT_TOML.replace("REPLACE_VERSION", &config.version);
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
