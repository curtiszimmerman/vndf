use std::net::{
	SocketAddr,
	TcpListener,
};
use std::sync::mpsc::{
	channel,
	Receiver,
};
use std::sync::mpsc::TryRecvError::{
	Disconnected,
	Empty,
};
use std::thread::spawn;
use std::vec::Drain;

use rustc_serialize::Decodable;

use super::Connection;


pub struct Acceptor<R: Send> {
	receiver   : Receiver<(SocketAddr, Connection<R>)>,
	connections: Vec<(SocketAddr, Connection<R>)>,
}

impl<R> Acceptor<R> where R: Decodable + Send + 'static {
	pub fn new(port: u16) -> Acceptor<R> {
		let (connection_sender, connection_receiver) = channel();
		let (init_sender      , init_receiver      ) = channel();

		spawn(move || {
			let listener = match TcpListener::bind(&("::", port)) {
				Ok(listener) => listener,
				Err(error)   =>
					panic!(
						"Error binding listener to port {}: {}",
						port, error,
					),
			};

			// Notify the constructing thread that we're not listening.
			if let Err(_) = init_sender.send(()) {
				panic!("Init notifaction channel disconnected");
			}

			loop {
				trace!("Start acceptor loop iteration");

				let (stream, address) = match listener.accept() {
					Ok(result) => result,
					Err(error) => panic!("Error accepting stream: {}", error),
				};

				let connection = Connection::from_stream(stream);

				if let Err(_) = connection_sender.send((address, connection)) {
					panic!("Acceptor channel disconnected");
				}
			}
		});

		// Don't return until the listener thread is listening.
		if let Err(_) = init_receiver.recv() {
			panic!("Init notification channel disconnected")
		}

		Acceptor {
			connections: Vec::new(),
			receiver   : connection_receiver,
		}
	}

	pub fn accept(&mut self) -> Drain<(SocketAddr, Connection<R>)> {
		loop {
			match self.receiver.try_recv() {
				Ok(connection) =>
					self.connections.push(connection),
				Err(error) => match error {
					Empty        => break,
					Disconnected => panic!("Acceptor channel disconnected"),
				},
			}
		}

		self.connections.drain(..)
	}
}
