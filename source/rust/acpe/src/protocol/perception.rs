use std::io::IoResult;

use rustc_serialize::{
	json,
	Encodable,
};

use super::{
	Message,
	Part,
	Seq,
};


pub type Perception<T> = Message<PerceptionHeader, T>;


#[deriving(Clone, PartialEq, RustcDecodable, RustcEncodable, Show)]
pub struct PerceptionHeader {
	pub confirm_action: Seq,
	pub self_id       : Option<String>,
}

impl Part for PerceptionHeader {
	fn assemble<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
		try!(self.encode(&mut json::Encoder::new(writer)));
		try!(writer.write_char('\n'));

		Ok(())
	}

	fn disassemble(line: &str) -> Result<PerceptionHeader, String> {
		json::decode(line)
			.map_err(|error|
				format!("Error decoding perception header: {}", error)
			)
	}
}
