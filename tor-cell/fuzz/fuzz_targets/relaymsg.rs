#![no_main]
use libfuzzer_sys::fuzz_target;
use tor_bytes::Reader;
use tor_cell::relaycell::RelayCell;

fuzz_target!(|data: &[u8]| {
	let mut r = Reader::from_slice(data);
	let _ = RelayCell::decode_from_reader(&mut r);
});
