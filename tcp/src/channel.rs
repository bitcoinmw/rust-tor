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

use hex_literal::hex;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tor_linkspec::OwnedChanTarget;
use tor_llcrypto::pk::ed25519::Ed25519Identity;
use tor_llcrypto::pk::rsa::RsaIdentity;
use tor_proto::channel::ChannelBuilder;
use tor_rtcompat::tls::TlsConnector;
use tor_rtcompat::CertifiedConn;
use tor_rtcompat::Runtime;
use tor_util::logger::Log;
use tor_util::Error;

use crate::channel::mpsc::UnboundedReceiver;
use crate::channel::mpsc::UnboundedSender;
use futures::channel::mpsc;
use futures::SinkExt;

pub struct Channel {}

const ED_ID: [u8; 32] = hex!("b9f1a5e4f6c3106d7a8bfd622b6bb2b532f999309cd598ab3e527f35db88547a");
const RSA_ID: [u8; 20] = hex!("5492e760648aabb7bec08bc87e8f72f55fbba90d");

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

	let (mut tx, _rx): (
		UnboundedSender<Arc<tor_proto::channel::Channel>>,
		UnboundedReceiver<Arc<tor_proto::channel::Channel>>,
	) = mpsc::unbounded();

	runtime.block_on(async move {
		{
			let mut mainlog = mainlog.lock().unwrap();
			mainlog.log(&format!("about to connect: {}", addr)).ok();
		}
		let tls = tls_connector
			.connect_unvalidated(&addr, "ignored")
			.await
			.unwrap();
		let peer_cert = tls.peer_certificate().unwrap().unwrap();
		let outboundhs = cb.launch(tls);
		{
			let mut mainlog = mainlog.lock().unwrap();
			mainlog.log("channel created").ok();
		}
		let channel = outboundhs.connect();
		let channel = channel.await.unwrap();
		let ed: Ed25519Identity = ED_ID.into();
		let rsa: RsaIdentity = RSA_ID.into();
		let target = OwnedChanTarget::new(vec![addr], ed, rsa);
		let channel = channel.check(&target, &peer_cert, None).unwrap();
		let (channel, _reactor) = channel.finish().await.unwrap();
		tx.send(channel).await.unwrap();
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
