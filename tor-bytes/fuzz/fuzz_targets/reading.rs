#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tor_bytes::Reader;

#[derive(Clone, Debug, Arbitrary)]
enum Op {
	GetLen,
	GetRemaining,
	GetConsumed,
	Advance(usize),
	CheckExhausted,
	Truncate(usize),
	Peek(usize),
	Take(usize),
	TakeU8,
	TakeU16,
	TakeU32,
	TakeU64,
	TakeU128,
	TakeUntil(u8),
	ExtractU32,
	ExtractU32N(usize),
}

#[derive(Clone, Debug, Arbitrary)]
struct Example {
	input: Vec<u8>,
	ops: Vec<Op>,
}

impl Example {
	fn run(self) {
		let mut r = Reader::from_slice(&self.input[..]);
		for op in self.ops {
			op.run(&mut r);
		}
		let _ignore = r.into_rest();
	}
}

impl Op {
	fn run(self, r: &mut Reader) {
		use Op::*;
		match self {
			GetLen => {
				let _len = r.total_len();
			}
			GetRemaining => {
				let _rem = r.remaining();
			}
			GetConsumed => {
				let _cons = r.consumed();
			}
			Advance(n) => {
				let _ignore = r.advance(n);
			}
			CheckExhausted => {
				let _ignore = r.should_be_exhausted();
			}
			Truncate(n) => {
				r.truncate(n);
			}
			Peek(n) => {
				let _ignore = r.peek(n);
			}
			Take(n) => {
				let _ignore = r.take(n);
			}
			TakeU8 => {
				let _u = r.take_u8();
			}
			TakeU16 => {
				let _u16 = r.take_u16();
			}
			TakeU32 => {
				let _u32 = r.take_u32();
			}
			TakeU64 => {
				let _u64 = r.take_u64();
			}
			TakeU128 => {
				let _u128 = r.take_u128();
			}
			TakeUntil(byte) => {
				let _ignore = r.take_until(byte);
			}
			ExtractU32 => {
				let _ignore: Result<u32, _> = r.extract();
			}
			ExtractU32N(n) => {
				let _ignore: Result<Vec<u32>, _> = r.extract_n(n);
			}
		}
	}
}

fuzz_target!(|ex: Example| {
	ex.run();
});
