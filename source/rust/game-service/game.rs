use collections::HashMap;
use libc::c_int;
use std::comm::{
	Disconnected,
	Empty
};

use common::physics::{Body, Radians, Vec2};
use common::net::Connection;
use common::protocol::{
	Action,
	Perception,
	Ship
};

use clients::{
	Client,
	Clients
};
use events::{
	Action,
	Close,
	Enter,
	GameEvent,
	Init,
	Leave,
	NetworkEvent,
	Update
};


pub struct Game {
	pub events: Sender<GameEvent>,

	incoming: Receiver<GameEvent>,
	network : Sender<NetworkEvent>,

	missiles: HashMap<uint, Body>
}


impl Game {
	pub fn new(network: Sender<NetworkEvent>) -> Game {
		let (sender, receiver) = channel();

		Game {
			events  : sender,

			incoming: receiver,
			network : network,

			missiles: HashMap::new()
		}
	}

	pub fn handle(&mut self, clients: &mut Clients) {
		loop {
			match self.incoming.try_recv() {
				Ok(event) => {
					print!("Incoming event: {}\n", event);

					match event {
						Init =>
							(), // nothing do do, it just exists for the logging
						Enter(connection) =>
							self.on_enter(connection, clients),
						Leave(clientId) =>
							self.on_leave(clientId, clients),
						Update(frame_time_in_s) =>
							self.on_update(clients, frame_time_in_s),
						Action(client_id, action) =>
							self.on_action(client_id, action, clients)
					}
				},

				Err(error) => match error {
					Empty        => break,
					Disconnected => fail!("Unexpected error: {}", error)
				}
			}
		}
	}

	fn on_enter(&mut self, connection: Connection, clients: &mut Clients) {
		let velocity = Vec2(30.0, 10.0);

		let ship = Body {
			position: Vec2::zero(),
			velocity: velocity,
			attitude: Radians::from_vec(velocity)
		};

		let new_client = Client::new(connection, ship);
		clients.add(new_client);
	}

	fn on_leave(&mut self, removed_id: uint, clients: &mut Clients) {
		clients.remove(removed_id);
	}

	fn on_update(&mut self, clients: &mut Clients, dTimeInS: f64) {
		clients.mut_each(|_, client| {
			client.ship.velocity = client.ship.attitude.to_vec() * 30.0;
			client.ship.position =
				client.ship.position + client.ship.velocity * dTimeInS;
		});

		for (_, missile) in self.missiles.mut_iter() {
			missile.velocity = missile.attitude.to_vec() * 30.0;
			missile.position =
				missile.position + missile.velocity * dTimeInS;
		};

		let mut ships = Vec::new();
		clients.each(|client_id, client| {
			ships.push(Ship {
				id  : client_id,
				body: client.ship
			});
		});

		let missiles: Vec<_> = self.missiles
			.iter()
			.map(|(&id, &body)|
				Ship {
					id  : id,
					body: body})
			.collect();

		clients.each(|client_id, client| {
			let update = Perception {
				self_id : client_id,
				ships   : ships.clone(),
				missiles: missiles.clone()
			};
			let message = update.to_str();

			match client.conn.send_message(message) {
				Err(_) => self.network.send(Close(client_id)),
				_      => ()
			};
		});
	}

	fn on_action(&mut self, fd: c_int, action: Action, clients: &mut Clients) {
		match clients.client_by_fd(fd) {
			Some((_, client)) => {
				client.ship.attitude = action.attitude;

				if action.missile > client.missile {
					let mut body = Body::default();
					body.position = client.ship.position;

					self.missiles.insert(
						(fd * 1000) as uint + action.missile as uint,
						body);
				}
				client.missile = action.missile;
			},
			None              => ()
		}
	}
}
