use libc;

use client::Frame;
use termios::Termios;


pub trait Output {
	fn render(&mut self, frame: &Frame);
}


pub struct PlayerOutput;

impl PlayerOutput {
	pub fn new() -> PlayerOutput {
		let mut termios = Termios::get(libc::STDIN_FILENO);
		termios.echo(false);
		termios.canonical_input(false);
		termios.set(libc::STDIN_FILENO);

		PlayerOutput
	}
}

impl Output for PlayerOutput {
	fn render(&mut self, frame: &Frame) {
		print!("\x1b[2J\x1b[H");

		for broadcast in frame.broadcasts.iter() {
			print!("{}\n", broadcast);
		}
	}
}


pub struct HeadlessOutput;

impl HeadlessOutput {
	pub fn new() -> HeadlessOutput {
		HeadlessOutput
	}
}

impl Output for HeadlessOutput {
	fn render(&mut self, frame: &Frame) {
		print!("{}\n", frame.to_json());
	}
}
