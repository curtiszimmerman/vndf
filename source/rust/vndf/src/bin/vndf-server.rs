#![cfg_attr(test, allow(dead_code))]


extern crate env_logger;
#[macro_use]
extern crate log;
extern crate time;

extern crate vndf;


use std::env;
use std::thread::sleep_ms;

use time::precise_time_s;

use vndf::server::args::Args;
use vndf::server::clients::Clients;
use vndf::server::game::state::GameState;
use vndf::server::incoming_events::IncomingEvents;
use vndf::server::network::Network;
use vndf::server::outgoing_events::{
	OutgoingEvents,
	Recipients,
};
use vndf::shared::protocol::server::Event as ServerEvent;


fn main() {
	env_logger::init().unwrap_or_else(|e|
		panic!("Error initializing logger: {}", e)
	);

	let args = Args::parse(env::args());

	let mut game_state = GameState::new();
	let mut clients    = Clients::new();
	let mut network    = Network::new(args.port);

	info!("Listening on port {}", args.port);

	let mut incoming_events = IncomingEvents::new();
	let mut outgoing_events = OutgoingEvents::new();

	loop {
		trace!("Start server main loop iteration");

		let now_s = precise_time_s();

		incoming_events.receive(network.receive());
		incoming_events.handle(
			now_s,
			&mut clients,
			&mut game_state,
			&mut outgoing_events,
		);

		clients.remove_inactive(now_s, args.client_timeout_s, |client| {
			outgoing_events.push(
				ServerEvent::RemoveEntity(client.ship_id),
				Recipients::All,
			);
			game_state.on_leave(&client.ship_id);
		});

		game_state.on_update(now_s);

		for (id, entity) in game_state.export_entities() {
			outgoing_events.push(
				ServerEvent::UpdateEntity(id, entity),
				Recipients::All,
			)
		}

		outgoing_events.push(ServerEvent::Heartbeat(now_s), Recipients::All);
		outgoing_events.send(&mut clients, &mut network);

		// TODO(1oL33ljB): While physics will generally need to happen on a
		//                 fixed interval, there's not really a reason to delay
		//                 other kinds of logic by sleeping. For example,
		//                 broadcasts can be handled immediately.
		sleep_ms(args.sleep_ms);
	}
}