#![feature(phase)]


extern crate acpe;
extern crate time;

extern crate client;
extern crate common;
extern crate test_tools_ng;


mod unit {
	mod client {
		mod server;
	}
	mod common {
		mod socket;
	}
}
mod component {
	mod client {
		mod protocol;
	}
	mod game_service {
		mod protocol;
	}
}
mod acceptance {
	mod basic;
}
