use std::comm::TryRecvError;
use std::io::stdin;

use self::command_kinds::{
	CommandKind,
	CommandKinds,
};


mod command_kinds;


pub struct InputReader {
	receiver     : Receiver<char>,
	current      : String,
	command_kinds: CommandKinds,
	start_with   : Vec<String>,
}

impl InputReader {
	pub fn new() -> InputReader {
		let (sender, receiver) = channel();

		spawn(proc() {
			let mut stdin = stdin();

			loop {
				// TODO(83541252): This operation should time out to ensure
				//                 panic propagation between tasks.
				match stdin.read_char() {
					Ok(c) =>
						sender.send(c),
					Err(error) =>
						panic!("Error reading from stdint: {}", error),
				}
			}
		});

		InputReader {
			receiver     : receiver,
			current      : String::new(),
			command_kinds: CommandKinds::new(),
			start_with   : Vec::new(),
		}
	}

	pub fn read_commands(&mut self) -> Vec<CommandResult> {
		let mut commands = Vec::new();

		loop {
			match self.receiver.try_recv() {
				Ok(c) => {
					if c == '\x7f' { // Backspace
						self.current.pop();
					}
					else if c == '\x09' { // Tab
						if self.start_with.len() == 1 {
							self.current = self.start_with[0].clone();
							self.current.push(' ');
						}
					}
					else if c == '\n' {
						commands.push(Command::parse(
							&self.command_kinds,
							self.current.clone(),
						));
						self.current.clear();
					}
					else if c.is_control() {
						// ignore other control characters
					}
					else {
						self.current.push(c);
					}
				},

				Err(error) => match error {
					TryRecvError::Empty =>
						break,
					TryRecvError::Disconnected =>
						panic!("Channel disconnected"),
				}
			}
		}

		self.start_with = self.command_kinds
			.start_with(self.current.as_slice())
			.iter()
			.map(|kind|
				kind.name().to_string()
			)
			.collect();

		commands.push(Err(CommandError::Incomplete(
			self.current.clone(),
			self.start_with.clone(),
		)));

		commands
	}
}


#[deriving(Show)]
pub enum Command {
	Help(&'static str),
	Broadcast(String),
	StopBroadcast,
}

impl Command {
	fn parse(kinds: &CommandKinds, full_command: String) -> CommandResult {
		let mut splits = full_command.splitn(1, ' ');
		
		let command = match splits.next() {
			Some(command) =>
				command,
			None =>
				return Err(CommandError::Invalid(
					"Invalid command",
					full_command.clone(),
				)),
		};

		let args = splits.next();

		let kind = match kinds.get(command) {
			Some(kind) =>
				kind,
			None =>
				return Err(CommandError::Invalid(
					"Unknown command",
					full_command.clone()
				)),
		};

		match kind.parse(args, kinds) {
			Ok(command) =>
				Ok(command),
			Err(error) =>
				return Err(CommandError::Invalid(
					error,
					full_command.clone()
				)),
		}
	}
}


pub type CommandResult = Result<Command, CommandError>;


#[deriving(Show)]
pub enum CommandError {
	Incomplete(String, Vec<String>),
	Invalid(&'static str, String),
}
