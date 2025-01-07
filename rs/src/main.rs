use avian3d::math::Vector;
use avian3d::prelude::Gravity;
use avian3d::PhysicsPlugins;
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
		.add_plugins(FrameTimeDiagnosticsPlugin)
		.add_plugins(PhysicsPlugins::default())
		.add_plugins((
			cam::CamPlugin,
			env::EnvPlugin,
			machine::MachinePlugin,
			tools::ToolsPlugin,
			ui::UiPlugin,
		))
		.init_resource::<Winnings>()
		.insert_resource(Gravity(Vector::NEG_Z * 9.81))
		.run();
}

#[derive(Resource, Debug)]
pub struct Winnings(Currency);

impl Default for Winnings {
	fn default() -> Self {
		Self(Currency::from_str("$0.00").unwrap())
	}
}
