use super::{
	MAX_PACKET_SIZE,
	PerceptionEnc,
	Seq,
};


pub struct Encoder {
	buffer: [u8, ..MAX_PACKET_SIZE],
}

impl Encoder {
	pub fn new() -> Encoder {
		Encoder {
			buffer: [0, ..MAX_PACKET_SIZE],
		}
	}

	pub fn perception(&mut self, last_action: Seq) -> PerceptionEnc {
		PerceptionEnc::new(&mut self.buffer, last_action)
	}
}


pub mod buf_writer {
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