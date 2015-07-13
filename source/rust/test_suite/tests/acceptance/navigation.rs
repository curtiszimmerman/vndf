use std::f64::consts::PI;

use nalgebra::{
	Dot,
	Norm,
	Vec2,
};

use client::interface::InputEvent;
use shared::util::is_point_on_line;
use test_suite::rc;


#[test]
fn it_should_send_navigation_data() {
	let     server = rc::Server::start();
	let mut client = rc::Client::start(server.port());

	let frame_1 = client.wait_until(|frame|
		frame.ship_id.is_some() && frame.ships.len() == 1
	);
	let ship_id = match frame_1.ship_id {
		Some(ship_id) => ship_id,
		None          => panic!("Expected ship id"),
	};

	let frame_1 = client.wait_until(|frame|
		frame.ships[&ship_id].position != frame_1.ships[&ship_id].position
	);
	let frame_2 = client.wait_until(|frame|
		frame.ships[&ship_id].position != frame_1.ships[&ship_id].position
	);

	assert!(is_point_on_line(
		frame_2.ships[&ship_id].position,
		frame_1.ships[&ship_id].position, frame_1.ships[&ship_id].velocity,
	));
}

#[test]
fn it_should_display_other_players_ships() {
	let     server   = rc::Server::start();
	let mut client_a = rc::Client::start(server.port());

	client_a.wait_until(|frame|
		frame.ships.len() == 1
	);

	let mut client_b = rc::Client::start(server.port());

	client_a.wait_until(|frame|
		frame.ships.len() == 2
	);
	client_b.wait_until(|frame|
		frame.ships.len() == 2
	);

	drop(client_b);

	client_a.wait_until(|frame|
		frame.ships.len() == 1
	);
}

#[test]
fn it_should_schedule_maneuvers() {
	let     server = rc::Server::start();
	let mut client = rc::Client::start(server.port());

	let frame_1 = client.wait_until(|frame| {
		frame.ship_id.is_some() && frame.ships.len() == 1
	});
	let ship_id = match frame_1.ship_id {
		Some(ship_id) => ship_id,
		None          => panic!("Expected ship id"),
	};

	let velocity_direction_rad   = direction(frame_1.ships[&ship_id].velocity);
	let maneuver_1_direction_rad = velocity_direction_rad + PI / 2.0;
	let maneuver_2_direction_rad = velocity_direction_rad + PI;

	client.input(InputEvent::ScheduleManeuver(0.01, maneuver_1_direction_rad));
	client.input(InputEvent::ScheduleManeuver(0.02, maneuver_2_direction_rad));

	client.wait_until(|frame| {
		maneuver_1_direction_rad == direction(frame.ships[&ship_id].velocity)
	});
	client.wait_until(|frame| {
		maneuver_2_direction_rad == direction(frame.ships[&ship_id].velocity)
	});


	fn direction(v: Vec2<f64>) -> f64 {
		angle_between(Vec2::new(1.0, 0.0), v)
	}

	fn angle_between(v1: Vec2<f64>, v2: Vec2<f64>) -> f64 {
		(v1.dot(&v2) / (v1.norm() * v2.norm())).acos()
	}
}
