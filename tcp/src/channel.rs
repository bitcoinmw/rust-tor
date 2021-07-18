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

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tor_proto::channel::ChannelBuilder;
use tor_rtcompat::tls::TlsConnector;
use tor_rtcompat::Runtime;
use tor_util::logger::Log;
use tor_util::Error;

pub struct Channel {}

fn connect(
	host: String,
	port: u16,
	runtime: &mut impl Runtime,
	mainlog: &Arc<Mutex<Log>>,
) -> Result<(), Error> {
	let addr: SocketAddr = format!("{}:{}", host, port,)
		.parse::<SocketAddr>()
		.unwrap()
		.into();
	let mut cb = ChannelBuilder::new();
	cb.set_declared_addr(addr);
	let tls_connector = runtime.tls_connector();
	runtime.block_on(async move {
		{
			let mut mainlog = mainlog.lock().unwrap();
			mainlog.log("about to connect").ok();
		}
		let tls = tls_connector
			.connect_unvalidated(&addr, "ignored")
			.await
			.unwrap();
		cb.launch(tls);
		{
			let mut mainlog = mainlog.lock().unwrap();
			mainlog.log("channel created").ok();
		}
	});
	Ok(())
}

pub fn build_channel(
	host: String,
	port: u16,
	runtime: &mut impl Runtime,
	mainlog: &Arc<Mutex<Log>>,
) -> Result<Channel, Error> {
	connect(host, port, runtime, mainlog)?;
	{
		let mut mainlog = mainlog.lock().unwrap();
		mainlog.log("connect complete").ok();
	}
	Ok(Channel {})
}
