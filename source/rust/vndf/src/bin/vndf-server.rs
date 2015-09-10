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
use vndf::shared::game::Attributes;

fn main() {
    env_logger::init().unwrap_or_else(|e|
                                      panic!("Error initializing logger: {}", e)
                                      );

    let args = Args::parse(env::args());

    let mut game_state = GameState::new();
    let mut clients    = Clients::new();
    let mut network    = Network::new(args.port);
    
    let planets = game_state.load_state();

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

        for ent in game_state.export_entities() {
            outgoing_events.push(
                ServerEvent::UpdateEntity(ent),
                Recipients::All,
                )
        }

	// check collisions
	// TODO: needs some notion of space-partitioning for efficiency
	let entities = game_state.get_entities();
	for (ship_id,ship_body) in entities.bodies.iter() {
	    // check only from the perspective of a ship
	    if let Some(attr) = entities.attributes.get(&ship_id) {
		if attr != &Attributes::Ship { continue }
	    } // if not found, likely a ship anyways
	    
	    let ship_coll = {
		if let Some (coll) = entities.colliders.get(&ship_id) { coll }
		else { warn!("No collider found for ship {}", ship_id);
		       continue }
	    };
	    for planet_id in planets.iter() {
		let planet_coll = {
		    if let Some (coll) = entities.colliders.get(&planet_id) { coll }
		    else { warn!("No collider found for planet {}", planet_id);
			   continue }
		};
		let planet_body = {
		    if let Some (body) = entities.bodies.get(&planet_id) { body }
		    else { warn!("No body found for planet {}", planet_id);
			   continue }
		};
		
		if ship_coll.check_collision(&ship_body.position,
					     (planet_coll,&planet_body.position)) {
		    outgoing_events.push(
			ServerEvent::Collision(*ship_id,*planet_id),
			Recipients::All);
		}
	    }

	    // TODO: ship-ship collision checks
	    //for (ship_id2,ship_body2) in frame.ships.iter() {
	    //    if ship_id == ship_id2 { continue }
	    //}
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
