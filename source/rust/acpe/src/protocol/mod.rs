use std::io::{
	BufReader,
	IoResult,
};

use self::buf_writer::BufWriter;
use super::MAX_PACKET_SIZE;


pub type Seq = u64;


pub struct Encoder {
	buffer: [u8, ..MAX_PACKET_SIZE],
}

impl Encoder {
	pub fn new() -> Encoder {
		Encoder {
			buffer: [0, ..MAX_PACKET_SIZE],
		}
	}

	pub fn message<P: MessagePart>(&mut self, seq: Seq) -> MessageEncoder<P> {
		MessageEncoder::new(&mut self.buffer, seq)
	}
}


pub struct MessageEncoder<'r, Part> {
	writer: BufWriter<'r>,
}

impl<'r, Part: MessagePart> MessageEncoder<'r, Part> {
	pub fn new(buffer: &mut [u8], confirmed_seq: Seq) -> MessageEncoder<Part> {
		let mut writer = BufWriter::new(buffer);

		match write!(&mut writer, "{}\n", confirmed_seq) {
			Ok(()) =>
				(),
			Err(error) =>
				panic!("Error writing message header: {}", error),
		}

		MessageEncoder {
			writer: writer,
		}
	}

	pub fn add(&mut self, part: &Part) -> bool {
		let mut buffer = [0, ..MAX_PACKET_SIZE];

		let len = {
			let mut writer = BufWriter::new(&mut buffer);
			match part.write(&mut writer) {
				Ok(())  => (),
				Err(_)  => return false,
			}

			writer.tell().unwrap_or_else(|_|
				panic!(
					"I/O operation on BufWriter that cannot possibly fail \
					still managed to fail somehow."
				)
			)
		};
		let addition = buffer[.. len as uint];

		match self.writer.write(addition) {
			Ok(()) => (),
			Err(_) => return false,
		}

		true
	}

	pub fn encode(self, buffer: &mut [u8]) -> IoResult<&[u8]> {
		let len = {
			let len = self.writer.tell().unwrap_or_else(|_|
				panic!(
					"I/O operation on BufWriter that cannot possibly fail \
					still managed to fail somehow."
				)
			);

			let mut writer = BufWriter::new(buffer);
			match writer.write(self.writer.into_slice()[.. len as uint]) {
				Ok(())     => (),
				Err(error) => return Err(error),
			};

			len
		};

		Ok(buffer[.. len as uint])
	}
}


// TODO: A decode method in an encoder module. Something has to change.
pub fn decode<P: MessagePart>(
	message: &[u8],
	parts  : &mut Vec<P>
) -> Result<Seq, String> {
	let mut reader = BufReader::new(message);

	let message = match reader.read_to_string() {
		Ok(message) =>
			message,
		Err(error) => {
			return Err(
				format!("Error converting message to string: {}\n", error)
			);
		},
	};

	let mut lines: Vec<&str> = message.split('\n').collect();

	let header = match lines.remove(0) {
		Some(header) =>
			header,
		None => {
			return Err(format!("Header line is missing\n"));
		},
	};

	let confirmed_seq = match from_str(header) {
		Some(confirmed_seq) =>
			confirmed_seq,
		None => {
			return Err(format!("Header is not a number\n"));
		},
	};

	for line in lines.into_iter() {
		if line.len() == 0 {
			continue;
		}

		match MessagePart::read(line) {
			Ok(part) =>
				parts.push(part),
			Err(error) =>
				return Err(error),

		}
	}

	Ok(confirmed_seq)
}


pub trait MessagePart {
	fn write<W: Writer>(&self, writer: &mut W) -> IoResult<()>;

	// TODO: This interface doesn't allow for an allocation-free implementation,
	//       when the type contains a String, Vec, or similar.
	fn read(line: &str) -> Result<Self, String>;
}


mod buf_writer {
	// This is code from the Rust standard library. I copied it because I needed
	// the BufWriter::into_slice method that I implemented here.

	// TODO(83622128): Send PR to Rust project.


	use std::io::{
		mod,
		IoError,
		IoResult,
		SeekStyle,
	};
	use std::slice;


	fn combine(seek: SeekStyle, cur: uint, end: uint, offset: i64) -> IoResult<u64> {
		// compute offset as signed and clamp to prevent overflow
		let pos = match seek {
			io::SeekSet => 0,
			io::SeekEnd => end,
			io::SeekCur => cur,
		} as i64;

		if offset + pos < 0 {
			Err(IoError {
				kind: io::InvalidInput,
				desc: "invalid seek to a negative offset",
				detail: None
			})
		} else {
			Ok((offset + pos) as u64)
		}
	}


	/// Writes to a fixed-size byte slice
	///
	/// If a write will not fit in the buffer, it returns an error and does not
	/// write any data.
	///
	/// # Example
	///
	/// ```rust
	/// # #![allow(unused_must_use)]
	/// use std::io::BufWriter;
	///
	/// let mut buf = [0, ..4];
	/// {
	///     let mut w = BufWriter::new(&mut buf);
	///     w.write(&[0, 1, 2]);
	/// }
	/// assert!(buf == [0, 1, 2, 0]);
	/// ```
	pub struct BufWriter<'a> {
		buf: &'a mut [u8],
		pos: uint
	}

	impl<'a> BufWriter<'a> {
		/// Creates a new `BufWriter` which will wrap the specified buffer. The
		/// writer initially starts at position 0.
		#[inline]
		pub fn new<'a>(buf: &'a mut [u8]) -> BufWriter<'a> {
			BufWriter {
				buf: buf,
				pos: 0
			}
		}

		/// Consumes the `BufWriter`, returning the slice it was originally
		/// created with.
		#[inline]
		pub fn into_slice(self) -> &'a mut [u8] {
			self.buf
		}
	}

	impl<'a> Writer for BufWriter<'a> {
		#[inline]
		fn write(&mut self, buf: &[u8]) -> IoResult<()> {
			// return an error if the entire write does not fit in the buffer
			let cap = if self.pos >= self.buf.len() { 0 } else { self.buf.len() - self.pos };
			if buf.len() > cap {
				return Err(IoError {
					kind: io::OtherIoError,
					desc: "Trying to write past end of buffer",
					detail: None
				})
			}

			slice::bytes::copy_memory(self.buf[mut self.pos..], buf);
			self.pos += buf.len();
			Ok(())
		}
	}

	impl<'a> Seek for BufWriter<'a> {
		#[inline]
		fn tell(&self) -> IoResult<u64> { Ok(self.pos as u64) }

		#[inline]
		fn seek(&mut self, pos: i64, style: SeekStyle) -> IoResult<()> {
			let new = try!(combine(style, self.pos, self.buf.len(), pos));
			self.pos = new as uint;
			Ok(())
		}
	}
}