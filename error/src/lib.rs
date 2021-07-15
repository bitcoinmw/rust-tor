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

use failure::{Backtrace, Context, Fail};
use std::ffi::OsString;
use std::fmt::{self, Display};

#[derive(Debug, Fail)]
pub struct Error {
	inner: Context<ErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
/// ErrorKinds for BMW Node
pub enum ErrorKind {
	/// Config Error
	#[fail(display = "Config Error: {}", _0)]
	ConfigError(String),
	/// P2P Error
	#[fail(display = "Request Error: {}", _0)]
	RequestError(String),
	/// IO Error
	#[fail(display = "IOError: {}", _0)]
	IOError(String),
	/// Hyper Error
	#[fail(display = "Hyper: {}", _0)]
	Hyper(String),
	/// Poison Error
	#[fail(display = "OsString: {}", _0)]
	OsString(String),
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let cause = match self.cause() {
			Some(c) => format!("{}", c),
			None => String::from("Unknown"),
		};
		let backtrace = match self.backtrace() {
			Some(b) => format!("{}", b),
			None => String::from("Unknown"),
		};
		let output = format!(
			"{} \n Cause: {} \n Backtrace: {}",
			self.inner, cause, backtrace
		);
		Display::fmt(&output, f)
	}
}

impl Error {
	/// get kind
	pub fn kind(&self) -> ErrorKind {
		self.inner.get_context().clone()
	}
	/// get cause
	pub fn cause(&self) -> Option<&dyn Fail> {
		self.inner.cause()
	}
	/// get backtrace
	pub fn backtrace(&self) -> Option<&Backtrace> {
		self.inner.backtrace()
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Error {
		Error {
			inner: Context::new(kind),
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::IOError(format!("{}", e))),
		}
	}
}

impl From<hyper::http::Error> for Error {
	fn from(e: hyper::http::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::Hyper(format!("{}", e))),
		}
	}
}

impl From<hyper::Error> for Error {
	fn from(e: hyper::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::Hyper(format!("{}", e))),
		}
	}
}

impl From<OsString> for Error {
	fn from(e: OsString) -> Error {
		Error {
			inner: Context::new(ErrorKind::OsString(format!("{:?}", e))),
		}
	}
}
