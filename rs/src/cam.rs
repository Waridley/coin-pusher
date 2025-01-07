use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

pub struct CamPlugin;

impl Plugin for CamPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, spawn_camera)
			.add_systems(Update, move_cam);
	}
}

pub fn spawn_camera(mut cmds: Commands) {
	cmds.spawn((
		CamSwivel,
		Transform::from_translation(Vec3::new(0.0, 15.0, 5.0)),
	))
	.with_children(|cmds| {
		cmds.spawn((
			CamTilter,
			Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_8)),
		))
		.with_child((
			Projection::Perspective(PerspectiveProjection {
				fov: FRAC_PI_2,
				..default()
			}),
			Camera3d::default(),
			Transform {
				translation: Vec3::new(0.0, -50.0, 0.0),
				rotation: Quat::from_rotation_arc(Vec3::NEG_Z, Vec3::Y),
				..default()
			},
		));
	});
}

#[derive(Component, Debug)]
#[require(Transform, Visibility)]
pub struct CamSwivel;

#[derive(Component, Debug)]
#[require(Transform, Visibility)]
pub struct CamTilter;

pub fn move_cam(
	mut cam: Single<&mut Transform, With<CamTilter>>,
	keys: Res<ButtonInput<KeyCode>>,
	t: Res<Time>,
) {
	if keys.pressed(KeyCode::ArrowUp) {
		cam.rotation = cam
			.rotation
			.rotate_towards(Quat::from_rotation_x(-FRAC_PI_2), t.delta_secs());
	}
	if keys.pressed(KeyCode::ArrowDown) {
		cam.rotation = cam
			.rotation
			.rotate_towards(Quat::from_rotation_x(FRAC_PI_8 * 0.25), t.delta_secs());
	}
}
