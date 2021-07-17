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

use crate::Error;
use chrono::{DateTime, Local, Utc};
use std::fs::{canonicalize, metadata, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

/// The main logging object
pub struct Log {
	data: Option<LogParams>,
}

/// The data that is held by the Log object
pub struct LogParams {
	file: File,
	file_path: String,
	cur_size: u64,
	max_size: u64,
	init_age_millis: u128,
	max_age_millis: u128,
	file_header: String,
	show_timestamp: bool,
	show_stdout: bool,
}

impl LogParams {
	/// This function rotates logs
	fn rotate(&mut self) -> Result<(), Error> {
		let now: DateTime<Utc> = Utc::now();
		let rotation_string = now.format(".r_%m_%e_%Y_%T").to_string().replace(":", "-");
		let file_path = match self.file_path.rfind(".") {
			Some(pos) => &self.file_path[0..pos],
			_ => &self.file_path,
		};
		let file_path = format!(
			"{}{}_{}.log",
			file_path,
			rotation_string,
			rand::random::<u64>(),
		);
		std::fs::rename(&self.file_path, file_path.clone())?;
		self.file = OpenOptions::new()
			.append(true)
			.create(true)
			.open(&self.file_path)?;
		Ok(())
	}

	/// The actual logging function, handles rotation if needed
	pub fn log(&mut self, line: &str) -> Result<(), Error> {
		let line_bytes = line.as_bytes(); // get line as bytes
		self.cur_size += line_bytes.len() as u64 + 1; // increment cur_size
		if self.show_timestamp {
			// timestamp is an additional 23 bytes
			self.cur_size += 23;
		}
		// get current time
		let time_now = SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.expect("Time went backwards")
			.as_millis();

		// check if rotation is needed
		if self.cur_size >= self.max_size
			|| time_now.saturating_sub(self.init_age_millis) > self.max_age_millis
		{
			self.rotate()?;
			let line_bytes = self.file_header.as_bytes();
			self.file.write(line_bytes)?;
			self.file.write(&[10u8])?; // new line
			self.init_age_millis = time_now;
			self.cur_size = self.file_header.len() as u64 + 1;
		}

		// if we're showing the timestamp, print it
		if self.show_timestamp {
			let date = Local::now();
			let formatted_ts = date.format("%Y-%m-%d %H:%M:%S");
			self.file
				.write(format!("[{}]: ", formatted_ts).as_bytes())?;
			if self.show_stdout {
				print!("[{}]: ", formatted_ts);
			}
		}
		// finally log the line followed by a newline.
		self.file.write(line_bytes)?;
		self.file.write(&[10u8])?; // newline

		// if stdout is specified log to stdout too
		if self.show_stdout {
			println!("{}", line);
		}

		Ok(())
	}
}

impl Log {
	/// create a new Log object to use based on specified values
	pub fn new(
		file_path: &str,
		max_size: u64,
		max_age_millis: u128,
		show_timestamp: bool,
		file_header: &str,
	) -> Result<Log, Error> {
		// create file with append option and create option
		let mut file = OpenOptions::new()
			.append(true)
			.create(true)
			.open(file_path)?;
		// get current size of the file
		let mut cur_size = metadata(file_path)?.len();
		// age is only relative to start logging time
		let init_age_millis = SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.expect("Time went backwards")
			.as_millis();
		let file_path = canonicalize(PathBuf::from(file_path))?
			.into_os_string()
			.into_string()?;
		let file_header = file_header.to_string();
		if cur_size == 0 {
			// add the header if the file is new
			let line_bytes = file_header.as_bytes();
			file.write(line_bytes)?;
			file.write(&[10u8])?; // new line
			cur_size = file_header.len() as u64 + 1;
		}

		// return Log object
		Ok(Log {
			data: Some(LogParams {
				max_size,
				cur_size,
				file,
				file_path,
				max_age_millis,
				init_age_millis,
				show_timestamp,
				file_header,
				show_stdout: true,
			}),
		})
	}

	/// Entry point for logging
	pub fn log(&mut self, line: &str) -> Result<(), Error> {
		let log_params = &mut *self.data.as_mut().unwrap();
		log_params.log(line)?;

		Ok(())
	}

	/// Update the show_timestamp parameter for this logger
	pub fn update_show_timestamp(&mut self, show: bool) -> Result<(), Error> {
		let log_params = &mut *self.data.as_mut().unwrap();
		log_params.show_timestamp = show;
		Ok(())
	}

	/// Update the show_stdout parameter for this logger
	pub fn update_show_stdout(&mut self, show: bool) -> Result<(), Error> {
		let log_params = &mut *self.data.as_mut().unwrap();
		log_params.show_stdout = show;
		Ok(())
	}
}
