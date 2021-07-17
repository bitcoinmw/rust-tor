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
use tor_util::http::{build_connector_context, do_get, UrlContext};
use tor_util::logger::Log;
use tor_util::store::lmdb::Store;
use tor_util::Error;
use tor_util::StopState;

use std::convert::TryInto;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread;
use std::time::SystemTime;

use tor_util::core::ser::{self, Readable, Reader, Writeable, Writer};

const DB_NAME: &str = "ds_db";
const HOSTS_KEY: &[u8] = &[1];

pub struct DSContext {
	store: Store,
	http: UrlContext,
}

#[derive(Debug)]
pub struct HostInfo {
	pub host: String,
	pub port: u16,
}

impl Writeable for HostInfo {
	fn write<W: Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
		let items = self.host.split(".");
		let items = items.collect::<Vec<&str>>();
		for i in items {
			let val = u8::from_str(i);
			if val.is_err() {
				return Err(ser::Error::CorruptedData);
			}
			writer.write_u8(u8::from_str(i).unwrap())?;
		}
		writer.write_u16(self.port)?;
		Ok(())
	}
}

impl Readable for HostInfo {
	fn read<R: Reader>(reader: &mut R) -> Result<Self, ser::Error> {
		let mut items = vec![];
		for _ in 0..4 {
			let item = reader.read_u8()?;
			items.push(item);
		}
		let host = format!("{}.{}.{}.{}", items[0], items[1], items[2], items[3]);
		let port = reader.read_u16()?;
		Ok(HostInfo { host, port })
	}
}

#[derive(Debug)]
pub struct DSInfo {
	pub hosts: Vec<HostInfo>,
	pub load_time: u128,
}

impl Writeable for DSInfo {
	fn write<W: Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
		writer.write_u64(self.load_time.try_into()?)?;
		writer.write_u64(self.hosts.len() as u64)?;
		for host in &self.hosts {
			host.write(writer)?;
		}

		Ok(())
	}
}

impl Readable for DSInfo {
	fn read<R: Reader>(reader: &mut R) -> Result<Self, ser::Error> {
		let load_time = reader.read_u64()?;
		let count = reader.read_u64()?;
		let mut hosts = vec![];
		for _ in 0..count {
			let host = HostInfo::read(reader)?;
			hosts.push(host);
		}

		Ok(DSInfo {
			load_time: load_time.into(),
			hosts,
		})
	}
}

fn load_ds_info_from_ds(
	directory_servers: Vec<String>,
	context: &DSContext,
) -> Result<String, Error> {
	let mut count = 0;
	let len = directory_servers.len();
	loop {
		let response = do_get(
			&format!(
				"http://{}/tor/status-vote/current/consensus/",
				directory_servers[count % len]
			),
			&context.http,
		);
		if response.is_ok() {
			return Ok(response?);
		}
		std::thread::sleep(std::time::Duration::from_millis(100));
		count += 1;
	}
}

fn update_db(directory_servers: Vec<String>, context: &DSContext) -> Result<(), Error> {
	let response = load_ds_info_from_ds(directory_servers, context)?;
	let arr = response.split("\n");
	let mut hosts = vec![];
	for line in arr {
		if line.starts_with("r ") {
			// this is a server to add to our hosts
			let items = line.split(" ");
			let items = items.collect::<Vec<&str>>();
			let host_info = HostInfo {
				host: items[6].to_string(),
				port: u16::from_str(items[7])?,
			};
			hosts.push(host_info);
		}
	}
	let load_time = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.expect("time went backwards")
		.as_millis();
	let dsinfo = DSInfo { load_time, hosts };

	// load hosts into DB
	{
		let batch = context.store.batch()?;
		batch.put_ser(HOSTS_KEY, &dsinfo)?;
		batch.commit()?;
	}

	Ok(())
}

fn get_hosts_from_db(context: &DSContext) -> Result<Option<DSInfo>, Error> {
	let batch = context.store.batch()?;
	let res: Option<DSInfo> = batch.get_ser(HOSTS_KEY)?;
	Ok(res)
}

pub fn build_ds_context(config: &TorConfig) -> Result<DSContext, Error> {
	let store = Store::new(&config.db_root, None, Some(DB_NAME), None, true)?;
	let http = build_connector_context(20, 20, 20);

	Ok(DSContext { store, http })
}

pub fn start_dsinfo_refresh_thread(
	config: &TorConfig,
	stop_state: Arc<RwLock<StopState>>,
	mainlog: Arc<Mutex<Log>>,
) -> Result<(), Error> {
	let refresh_frequency = config.ds_refresh_frequency;
	let directory_servers = config.directory_servers.clone();
	let context = build_ds_context(config)?;
	thread::spawn(move || {
		let mut count = 0;
		loop {
			if count != 0 && (count * 100) % refresh_frequency == 0 {
				{
					let mut mainlog = mainlog.lock().unwrap();
					(*mainlog)
						.log("updating directory information to DB")
						.unwrap();
				}
				update_db(directory_servers.clone(), &context).unwrap();
				{
					let mut mainlog = mainlog.lock().unwrap();
					(*mainlog)
						.log("updating directory information to DB complete")
						.unwrap();
				}
			}
			std::thread::sleep(std::time::Duration::from_millis(100));
			let stop_state = stop_state.read().unwrap();
			if stop_state.is_stopped() {
				break;
			}
			count += 1;
		}
	});

	Ok(())
}

pub fn get_latest_valid_dsinfo(config: &TorConfig, context: &DSContext) -> Result<DSInfo, Error> {
	let now = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.expect("time went backwards")
		.as_millis();
	let mut hosts = get_hosts_from_db(context)?;
	if hosts.is_none() || now - hosts.as_ref().unwrap().load_time > config.ds_refresh_timeout.into()
	{
		update_db(config.directory_servers.clone(), context)?;
		hosts = get_hosts_from_db(context)?;
	}

	Ok(hosts.unwrap())
}
