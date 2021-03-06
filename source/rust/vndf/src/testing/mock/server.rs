use std::net::SocketAddr;

use time::precise_time_s;

use server::network::Network;
use shared::protocol::{
	client,
	server,
};
use testing::util::random_port;


pub struct Server {
	port    : u16,
	network : Network,
	incoming: Vec<(SocketAddr, client::Event)>,
}

impl Server {
	pub fn start() -> Server {
		let port    = random_port(40000, 50000);
		let network = Network::new(port);

		Server {
			port    : port,
			network : network,
			incoming: Vec::new(),
		}
	}

	pub fn port(&self) -> u16 {
		self.port
	}

	pub fn send(&mut self, address: SocketAddr, event: server::Event) {
		self.network.send(
			Some(address).into_iter(),
			Some(event).into_iter(),
		);
	}

	// TODO(5rKZ3HPd): Make generic and move into a trait called Mock.
	pub fn expect_event(&mut self) -> Option<(SocketAddr, client::Event)> {
		let start_s = precise_time_s();

		while self.incoming.len() == 0 && precise_time_s() - start_s < 0.5 {
			self.incoming.extend(self.network.receive());
		}

		if self.incoming.len() > 0 {
			let event = self.incoming.remove(0);

			Some(event)
		}
		else {
			None
		}
	}

	// TODO(5rKZ3HPd): Make generic and move into a trait called Mock.
	pub fn wait_until<F>(&mut self, condition: F)
		-> Option<(SocketAddr, client::Event)>
		where
			F: Fn(&mut Option<(SocketAddr, client::Event)>) -> bool,
	{
		let start_s = precise_time_s();

		let mut event = self.expect_event();

		while !condition(&mut event) {
			if precise_time_s() - start_s > 0.5 {
				panic!("Condition not satisfied after waiting");
			}

			event = self.expect_event();
		}

		event
	}
}
