#![feature(phase)]


extern crate collections;
extern crate serialize;
extern crate time;

extern crate cgmath;

extern crate client;
extern crate game;
extern crate game_service;
extern crate platform;
extern crate protocol;
extern crate rustecs;
#[phase(plugin)] extern crate rustecs_macros;
#[phase(plugin)] extern crate test_infra;
extern crate test_tools;


mod unit {
	mod client_test {
		mod receiver;
	}

	mod common_test {
		mod protocol;
	}
}

mod component {
	mod client;
}

mod acceptance {
	mod basic;
	mod ship;
}
