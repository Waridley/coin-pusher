use avian3d::math::Vector;
use avian3d::prelude::{Gravity, SubstepCount};
use avian3d::PhysicsPlugins;
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use currency::Currency;

pub mod cam;
pub mod coins;
pub mod env;
pub mod machine;
pub mod tools;
pub mod ui;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugins((FrameTimeDiagnosticsPlugin, TemporalAntiAliasPlugin))
		.add_plugins(PhysicsPlugins::default())
		.add_plugins((
			cam::CamPlugin,
			coins::CoinsPlugin,
			env::EnvPlugin,
			machine::MachinePlugin,
			tools::ToolsPlugin,
			ui::UiPlugin,
		))
		.init_resource::<Winnings>()
		// Realistic gravity (772.44 half-inches/s^2 !!) causes too many problems
		// with the simulation. This is slow and a little "floaty," but satisfying
		// to watch anyway.
		.insert_resource(Gravity(Vector::NEG_Z * 20.0))
		.insert_resource(SubstepCount(4))
		.run();
}

#[derive(Resource, Debug)]
pub struct Winnings(Currency);

impl Default for Winnings {
	fn default() -> Self {
		Self(Currency::from_str("$0.00").unwrap())
	}
}
