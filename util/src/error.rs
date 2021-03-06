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

use crate::logger::Log;

use grin_util::StopState;
use std::sync::MutexGuard;
use std::sync::PoisonError;
use std::sync::RwLockWriteGuard;

use failure::{Backtrace, Context, Fail};
use futures::task::SpawnError;
use std::ffi::OsString;
use std::fmt::{self, Display};
use std::net::AddrParseError;
use std::num::ParseIntError;
use std::num::TryFromIntError;
use std::time::SystemTimeError;

#[derive(Debug, Fail)]
pub struct Error {
	inner: Context<ErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
/// ErrorKinds for Tor
pub enum ErrorKind {
	/// Config Error
	#[fail(display = "Config Error: {}", _0)]
	ConfigError(String),
	/// P2P Error
	#[fail(display = "Request Error: {}", _0)]
	RequestError(String),
	/// IO Error
	#[fail(display = "IO Error: {}", _0)]
	IOError(String),
	/// Hyper Error
	#[fail(display = "Hyper Error: {}", _0)]
	Hyper(String),
	/// OsString Error
	#[fail(display = "OsString Error: {}", _0)]
	OsString(String),
	/// Log not configured Error
	#[fail(display = "Log not configured Error: {}", _0)]
	LogNotConfigured(String),
	/// Path not found
	#[fail(display = "Path not found Error: {}", _0)]
	PathNotFoundError(String),
	/// Invalid TOML File
	#[fail(display = "TOML Error: {}", _0)]
	TomlError(String),
	/// Store Error
	#[fail(display = "Store Error: {}", _0)]
	StoreError(String),
	/// AddrParseError
	#[fail(display = "AddrParseError: {}", _0)]
	AddrParseError(String),
	/// Parse Int Error
	#[fail(display = "ParseInt Error: {}", _0)]
	ParseIntError(String),
	#[fail(display = "Poison Error: {}", _0)]
	PoisonError(String),
	/// SystemTimeError
	#[fail(display = "SystemTimeError Error: {}", _0)]
	SystemTimeError(String),
	/// SpawnError
	#[fail(display = "Spawn Error: {}", _0)]
	SpawnError(String),
	#[fail(display = "TcpConnectError: {}", _0)]
	TcpConnectError(String),
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

impl From<toml::de::Error> for Error {
	fn from(e: toml::de::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::TomlError(format!("{}", e))),
		}
	}
}

impl From<grin_store::Error> for Error {
	fn from(e: grin_store::Error) -> Error {
		Error {
			inner: Context::new(ErrorKind::StoreError(format!("{}", e))),
		}
	}
}

impl From<ParseIntError> for Error {
	fn from(e: ParseIntError) -> Error {
		Error {
			inner: Context::new(ErrorKind::ParseIntError(format!("{}", e))),
		}
	}
}

impl From<TryFromIntError> for Error {
	fn from(e: TryFromIntError) -> Error {
		Error {
			inner: Context::new(ErrorKind::ParseIntError(format!("{}", e))),
		}
	}
}

impl From<PoisonError<RwLockWriteGuard<'_, StopState>>> for Error {
	fn from(e: PoisonError<RwLockWriteGuard<'_, StopState>>) -> Error {
		Error {
			inner: Context::new(ErrorKind::PoisonError(format!("{}", e))),
		}
	}
}

impl From<std::sync::PoisonError<MutexGuard<'_, Log>>> for Error {
	fn from(e: PoisonError<MutexGuard<'_, Log>>) -> Error {
		Error {
			inner: Context::new(ErrorKind::PoisonError(format!("{}", e))),
		}
	}
}

impl From<SystemTimeError> for Error {
	fn from(e: SystemTimeError) -> Error {
		Error {
			inner: Context::new(ErrorKind::SystemTimeError(format!("{}", e))),
		}
	}
}

impl From<SpawnError> for Error {
	fn from(e: SpawnError) -> Error {
		Error {
			inner: Context::new(ErrorKind::SpawnError(format!("{}", e))),
		}
	}
}

impl From<AddrParseError> for Error {
	fn from(e: AddrParseError) -> Error {
		Error {
			inner: Context::new(ErrorKind::AddrParseError(format!("{}", e))),
		}
	}
}
