use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

pub struct EnvPlugin;

impl Plugin for EnvPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup_env);
	}
}

pub fn setup_env(mut cmds: Commands) {
	cmds.spawn((
		DirectionalLight {
			shadows_enabled: true,
			..default()
		},
		Transform::from_rotation(Quat::from_rotation_arc(
			Vec3::NEG_Z,
			Vec3::new(0.0, 0.2, -1.0).normalize(),
		)),
		CascadeShadowConfigBuilder {
			num_cascades: 1,
			minimum_distance: 2.0,
			maximum_distance: 80.0,
			first_cascade_far_bound: 80.0,
			..default()
		}
		.build(),
	));
}
