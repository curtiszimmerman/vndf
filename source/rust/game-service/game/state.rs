use collections::HashMap;
use std::comm::{
	Disconnected,
	Empty
};

use common::ecs::Components;
use common::physics::{
	Body,
	Radians,
	Vec2
};
use common::protocol::{
	Action,
	Perception
};

use events::{
	Action,
	Enter,
	GameEvent,
	Init,
	Leave,
	Message,
	NetworkEvent,
	Update
};
use game::data::{
	Player,
	Ship
};
use network::ClientId;


pub struct GameState {
	pub events: Sender<GameEvent>,

	incoming: Receiver<GameEvent>,
	network : Sender<NetworkEvent>,

	bodies  : Components<Body>,
	missiles: Components<Body>,
	ships   : Components<Ship>,
	players : Components<Player>
}

impl GameState {
	pub fn new(network: Sender<NetworkEvent>) -> GameState {
		let (sender, receiver) = channel();

		GameState {
			events  : sender,

			incoming: receiver,
			network : network,

			bodies  : HashMap::new(),
			missiles: HashMap::new(),
			ships   : HashMap::new(),
			players : HashMap::new()
		}
	}

	pub fn update(&mut self) {
		loop {
			match self.incoming.try_recv() {
				Ok(event) => {
					print!("Incoming event: {}\n", event);

					match event {
						Init =>
							(), // nothing do do, it just exists for the logging
						Enter(client_id) =>
							self.on_enter(client_id),
						Leave(client_id) =>
							self.on_leave(client_id),
						Update(frame_time_in_s) =>
							self.on_update(frame_time_in_s),
						Action(client_id, action) =>
							self.on_action(client_id, action)
					}
				},

				Err(error) => match error {
					Empty        => break,
					Disconnected => fail!("Unexpected error: {}", error)
				}
			}
		}
	}

	fn on_enter(&mut self, id: ClientId) {
		let velocity = Vec2(30.0, 10.0);
		self.bodies.insert(id, Body {
			position: Vec2::zero(),
			velocity: velocity,
			attitude: Radians::from_vec(velocity)
		});

		self.ships.insert(id, Ship);

		self.players.insert(id, Player {
			missile_index: 0
		});
	}

	fn on_leave(&mut self, id: ClientId) {
		self.ships.remove(&id);
		self.players.remove(&id);
	}

	fn on_update(&mut self, delta_time_in_s: f64) {
		for (_, body) in self.bodies.mut_iter() {
			integrate(body, delta_time_in_s);
		}

		for (_, missile) in self.missiles.mut_iter() {
			integrate(missile, delta_time_in_s);
		}

		for &id in self.players.keys() {
			let update = Perception {
				self_id : id,
				ships   : self.bodies
					.iter()
					.filter(|&(id, _)| self.ships.contains_key(id))
					.map(|(&id, &body)| (id, body))
					.collect(),
				missiles: self.missiles.clone()
			};

			self.network.send(Message(vec!(id), update));
		}
	}

	fn on_action(&mut self, id: ClientId, action: Action) {
		let ship = match self.bodies.find_mut(&id) {
			Some(ship) => ship,
			None       => return
		};

		ship.attitude = action.attitude;

		let player = self.players
			.find_mut(&id)
			.expect("expected control");

		if action.missile > player.missile_index {
			let mut body = Body::default();
			body.position = ship.position;
			body.attitude = ship.attitude;

			self.missiles.insert(
				(id * 1000) as ClientId + action.missile as ClientId,
				body);
		}
		player.missile_index = action.missile;
	}
}


fn integrate(body: &mut Body, delta_time_in_s: f64) {
	body.velocity = body.attitude.to_vec() * 30.0;
	body.position = body.position + body.velocity * delta_time_in_s;
}
