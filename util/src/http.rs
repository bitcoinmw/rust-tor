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

use crate::{Error, ErrorKind};
use hyper::client::HttpConnector;
use hyper::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use hyper::http::request::Builder;
use hyper::{body, Body, Client, Request};
use hyper_rustls::HttpsConnector;
use hyper_timeout::TimeoutConnector;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::{Builder as TokioBuilder, Runtime};

// Global Tokio runtime.
// Needs a `Mutex` because `Runtime::block_on` requires mutable access.
// Tokio v0.3 requires immutable self, but we are waiting on upstream
// updates before we can upgrade.
// See: https://github.com/seanmonstar/reqwest/pull/1076
lazy_static! {
	pub static ref RUNTIME: Arc<Mutex<Runtime>> = Arc::new(Mutex::new(
		TokioBuilder::new()
			.threaded_scheduler()
			.enable_all()
			.build()
			.unwrap()
	));
}

pub struct UrlContext {
	client: Client<TimeoutConnector<HttpsConnector<HttpConnector>>>,
	builder: Builder,
}

pub fn build_connector_context(
	connect_timeout_secs: u64,
	read_timeout_secs: u64,
	write_timeout_secs: u64,
) -> UrlContext {
	let https = hyper_rustls::HttpsConnector::new();
	let mut connector = TimeoutConnector::new(https);
	connector.set_connect_timeout(Some(Duration::from_secs(connect_timeout_secs)));

	connector.set_read_timeout(Some(Duration::from_secs(read_timeout_secs)));

	connector.set_write_timeout(Some(Duration::from_secs(write_timeout_secs)));

	let client = Client::builder().build::<_, Body>(connector);
	let builder = Request::builder();

	UrlContext { client, builder }
}

pub async fn async_do_get(url: &str, context: UrlContext) -> Result<String, Error> {
	let req = context
		.builder
		.method("get")
		.uri(url)
		.header(USER_AGENT, "rust-tor-client")
		.header(ACCEPT, "application/json")
		.header(CONTENT_TYPE, "application/json")
		.body(Body::empty())?;

	let resp = context.client.request(req).await?;

	let raw = body::to_bytes(resp)
		.await
		.map_err(|e| ErrorKind::RequestError(format!("Cannot read response body: {}", e)))?;

	Ok(String::from_utf8_lossy(&raw).to_string())
}

pub fn do_get(url: &str, context: UrlContext) -> Result<String, Error> {
	RUNTIME.lock().unwrap().block_on(async_do_get(url, context))
}
