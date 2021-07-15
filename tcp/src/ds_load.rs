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

use tor_config::config::TorConfig;
use tor_error::Error;
use tor_util::store::lmdb::Store;

const DB_NAME: &str = "db";

pub struct DSContext {
	store: Store,
}

pub struct DSInfo {}

pub fn build_ds_context(config: &TorConfig) -> Result<DSContext, Error> {
	let store = Store::new(&config.db_root, None, Some(DB_NAME), None)?;

	Ok(DSContext { store })
}

pub fn get_latest_valid_dsinfo(config: &TorConfig, context: &DSContext) -> Result<DSInfo, Error> {
	Ok(DSInfo {})
}
