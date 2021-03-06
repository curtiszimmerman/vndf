#![cfg_attr(test, allow(dead_code))]


extern crate env_logger;
#[macro_use]
extern crate log;
extern crate time;
extern crate nalgebra;

extern crate vndf;


use std::env;
use std::thread::sleep;
use std::time::Duration;

use time::precise_time_s;

use vndf::server::args::Args;
use vndf::server::clients::Clients;
use vndf::server::game::events;
use vndf::server::game::initial_state::InitialState;
use vndf::server::game::state::GameState;
use vndf::server::incoming_events::IncomingEvents;
use vndf::server::network::Network;
use vndf::server::outgoing_events::{
    OutgoingEvents,
    Recipients,
};
use vndf::shared::protocol::server::Event as ServerEvent;


fn main() {
    env_logger::init()
        .expect("Failed to initialize logger");

    let args = Args::parse(env::args())
        .expect("Failed to parse command-line arguments");

    let mut game_state = GameState::new(precise_time_s());
    let mut clients    = Clients::new();
    let mut network    = Network::new(args.port);

    let initial_state = match args.initial_state {
        Some(path) => InitialState::from_file(path),
        None       => InitialState::random(),
    };
    initial_state.apply(&mut game_state);

    info!("Listening on port {}", args.port);

    let mut incoming_events = IncomingEvents::new();
    let mut outgoing_events = OutgoingEvents::new();

    loop {
        trace!("Start server main loop iteration");

        let now_s = precise_time_s();

        outgoing_events.push(ServerEvent::Heartbeat(now_s), Recipients::All);

        incoming_events.receive(network.receive());
        incoming_events.handle(
            now_s,
            &mut clients,
            &mut game_state,
            &mut outgoing_events,
        );

        clients.remove_inactive(now_s, args.client_timeout_s, |client| {
            game_state
                .handle_event(events::Leave { ship_id: client.ship_id })
                .expect("Leave event should never fail to validate");
        });

        game_state
            .handle_event(events::Update { now_s: now_s })
            .expect("Update event should never fail to validate");

        for id in game_state.destroyed_entities() {
            // This sends the ids of all destroyed maneuvers to all clients,
            // even though a given maneuver is only visible to a single client
            // in the first place. Not very nice, but ok for now.
            outgoing_events.push(
                ServerEvent::RemoveEntity(id),
                Recipients::All,
            );
        }

        for entity in game_state.export_entities() {
            let recipients = match entity.maneuver {
                Some(maneuver) => {
                    // Not pretty, but works.
                    let mut address = None;
                    for (the_address, client) in &clients.clients {
                        if client.ship_id == maneuver.ship_id {
                            address = Some(the_address);
                        }
                    }

                    match address {
                        Some(address) => Recipients::One(*address),
                        None => {
                            warn!(
                                "Found maneuver with no matching ship: {:?}",
                                maneuver
                            );
                            continue;
                        },
                    }
                },

                None => Recipients::All,
            };

            outgoing_events.push(
                ServerEvent::UpdateEntity(entity),
                recipients,
            );
        }

        outgoing_events.send(&mut clients, &mut network);

        // TODO(1oL33ljB): While physics will generally need to happen on a
        //                 fixed interval, there's not really a reason to delay
        //                 other kinds of logic by sleeping. For example,
        //                 broadcasts can be handled immediately.
        sleep(Duration::from_millis(args.sleep_ms));
    }
}
