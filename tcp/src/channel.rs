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

use futures::stream::SplitStream;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tor_linkspec::OwnedChanTarget;
use tor_proto::channel::reactor::Reactor;
use tor_proto::channel::Channel;
use tor_proto::channel::ChannelBuilder;
use tor_rtcompat::tls::TlsConnector;
use tor_rtcompat::CertifiedConn;
use tor_rtcompat::Runtime;
use tor_rtcompat::TlsProvider;
use tor_util::logger::Log;
use tor_util::{Error, ErrorKind};

use futures::task::SpawnExt;
use tokio::sync::mpsc;

pub struct TorChannel {
	pub channel: Option<Arc<Channel>>,
	pub error: Option<Error>,
}

impl TorChannel {
	pub fn new() -> TorChannel {
		TorChannel {
			channel: None,
			error: None,
		}
	}
}

/*
async fn process_rx(
	rx: &mut mpsc::Receiver<
		Option<
			(Arc<Channel>,
			Reactor<SplitStream<asynchronous_codec::framed::Framed<<impl Runtime as TlsProvider>::TlsStream, tor_proto::channel::codec::ChannelCodec>>>)>>, channel_wrapper: Arc<Mutex<TorChannel>>) {
	let res = rx.recv().await;
	if res.is_some() {
		let res = res.unwrap();
		if res.is_some() {
			let (channel, reactor) = res.unwrap();
			let mut channel_wrapper = channel_wrapper.lock().unwrap();
			channel_wrapper.channel = Some(channel);
		} else {
			let mut channel_wrapper = channel_wrapper.lock().unwrap();
			channel_wrapper.channel = None;
		}
	}

	{
		let mut channel_wrapper = channel_wrapper.lock().unwrap();
		if channel_wrapper.channel.is_none() && channel_wrapper.error.is_none() {
			// it means it was a timeout so mark error
			channel_wrapper.error =
				Some(ErrorKind::TcpConnectError("connect timeout".to_string()).into());
		}
	}
}
*/

fn connect(
	host: String,
	port: u16,
	runtime: &mut impl Runtime,
	mainlog: &'static Arc<Mutex<Log>>,
	timeout: u64,
	channel_wrapper: Arc<Mutex<TorChannel>>,
) -> Result<(), Error> {
	let addr: SocketAddr = format!("{}:{}", host, port,).parse::<SocketAddr>()?.into();
	let mut cb = ChannelBuilder::new();
	cb.set_declared_addr(addr);
	let tls_connector = runtime.tls_connector();
	let (tx, mut rx) = mpsc::channel(1);
	let tx_arc = Arc::new(tx);
	let tx_arc_clone = tx_arc.clone();

	runtime.spawn(async move {
		std::thread::sleep(std::time::Duration::from_millis(timeout));
		let _ = tx_arc.send(None).await;
	})?;

	let channel_wrapper_clone = channel_wrapper.clone();
	runtime.spawn(async move {
		{
			let mut mainlog = mainlog.lock().unwrap();
			let _ = mainlog
				.log(&format!("connecting to: {}", addr))
				.map_err(|e| {
					println!("Logging error: {}", e.to_string(),);
				});
		}
		let tls = tls_connector
			.connect_unvalidated(&addr, "ignored")
			.await
			.map_err(|e| {
				let mut mainlog = mainlog.lock().unwrap();
				let _ = mainlog
					.log(&format!("connect error: {}", e.to_string()))
					.map_err(|e| {
						println!("Logging error: {}", e.to_string(),);
					});
			})
			.ok();

		if tls.is_none() {
			{
				let mut channel_wrapper = channel_wrapper_clone.lock().unwrap();
				channel_wrapper.error =
					Some(ErrorKind::TcpConnectError("no tls connector".to_string()).into());
			}
			let _ = tx_arc_clone.send(None).await;
		} else {
			let tls = tls.unwrap();
			let peer_cert = tls.peer_certificate();
			if peer_cert.is_err() {
				{
					let mut channel_wrapper = channel_wrapper_clone.lock().unwrap();
					channel_wrapper.error =
						Some(ErrorKind::TcpConnectError("peer_cert error".to_string()).into());
				}
				let _ = tx_arc_clone.send(None).await;
			} else {
				let peer_cert = peer_cert.unwrap();
				if peer_cert.is_none() {
					{
						let mut channel_wrapper = channel_wrapper_clone.lock().unwrap();
						channel_wrapper.error = Some(
							ErrorKind::TcpConnectError("peer_cert is_none".to_string()).into(),
						);
					}
					let _ = tx_arc_clone.send(None).await;
				} else {
					let peer_cert = peer_cert.unwrap();
					let outboundhs = cb.launch(tls);
					let channel = outboundhs.connect();
					let channel = channel.await;
					if channel.is_err() {
						{
							let mut channel_wrapper = channel_wrapper_clone.lock().unwrap();
							channel_wrapper.error = Some(
								ErrorKind::TcpConnectError("channel error".to_string()).into(),
							);
						}
						let _ = tx_arc_clone.send(None).await;
					} else {
						let channel = channel.unwrap();
						let mut target = OwnedChanTarget::new(vec![addr], None, None);
						let channel = channel.check(&mut target, &peer_cert, None);
						if channel.is_err() {
							{
								let mut channel_wrapper = channel_wrapper_clone.lock().unwrap();
								channel_wrapper.error = Some(
									ErrorKind::TcpConnectError("channel error".to_string()).into(),
								);
							}
							let _ = tx_arc_clone.send(None).await;
						} else {
							let channel = channel.unwrap();
							let res = channel.finish().await;
							if res.is_err() {
								{
									let mut channel_wrapper = channel_wrapper_clone.lock().unwrap();
									channel_wrapper.error = Some(
										ErrorKind::TcpConnectError(
											"channel finish error".to_string(),
										)
										.into(),
									);
								}
								let _ = tx_arc_clone.send(None).await;
							} else {
								let (channel, reactor) = res.unwrap();
								let _ = tx_arc_clone.send(Some((channel, reactor))).await;
							}
						}
					}
				}
			}
		}
	})?;

	//runtime.block_on(process_rx(&mut rx, channel_wrapper));

	runtime.block_on(async {
		let res = rx.recv().await;
		if res.is_some() {
			let res = res.unwrap();
			if res.is_some() {
				let (channel, reactor) = res.unwrap();
				let mut channel_wrapper = channel_wrapper.lock().unwrap();
				channel_wrapper.channel = Some(channel);
			} else {
				let mut channel_wrapper = channel_wrapper.lock().unwrap();
				channel_wrapper.channel = None;
			}
		}
	});

	{
		let mut channel_wrapper = channel_wrapper.lock().unwrap();
		if channel_wrapper.channel.is_none() && channel_wrapper.error.is_none() {
			// it means it was a timeout so mark error
			channel_wrapper.error =
				Some(ErrorKind::TcpConnectError("connect timeout".to_string()).into());
		}
	}

	Ok(())
}

pub fn build_channel(
	host: String,
	port: u16,
	runtime: &mut impl Runtime,
	mainlog: &'static Arc<Mutex<Log>>,
	connect_timeout: u64,
	tor_channel: Arc<Mutex<TorChannel>>,
) -> Result<(), Error> {
	connect(host, port, runtime, mainlog, connect_timeout, tor_channel)?;

	Ok(())
}
