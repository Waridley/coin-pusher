use crate::coins::Coin;
use crate::Winnings;
use avian3d::collision::{Collider, Sensor};
use avian3d::math::FRAC_PI_2;
use avian3d::prelude::{CoefficientCombine, Friction, LinearVelocity, Restitution, RigidBody};
use bevy::prelude::EaseFunction::SineInOut;
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_8;
use std::sync::Arc;

pub struct MachinePlugin;

impl Plugin for MachinePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, spawn_machine)
			.add_systems(FixedUpdate, move_piston)
			.add_systems(Update, collect);
	}
}

pub fn spawn_machine(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
) {
	let floor_collider = Collider::cuboid(20.0, 40.0, 5.0);
	let floor_mesh = meshes.add(Cuboid::new(20.0, 40.0, 5.0));
	let machine_mat = mats.add(StandardMaterial {
		base_color: Color::linear_rgb(0.1, 0.07, 0.15),
		metallic: 1.0,
		// anisotropy_strength: 0.8,
		..default()
	});
	let walls_and_floor_shared_bundle = (
		RigidBody::Static,
		MeshMaterial3d(machine_mat.clone()),
		Restitution::new(0.9),
	);
	// Floor
	cmds.spawn((
		walls_and_floor_shared_bundle.clone(),
		floor_collider.clone(),
		Mesh3d(floor_mesh.clone()),
	));
	let walls_collider = Collider::cuboid(50.0, 60.0, 5.0);
	let walls_mesh = meshes.add(Cuboid::new(50.0, 60.0, 5.0));
	let wall_bundle = (
		walls_and_floor_shared_bundle.clone(),
		walls_collider,
		Mesh3d(walls_mesh),
	);
	// Left wall
	cmds.spawn((
		wall_bundle.clone(),
		Transform {
			translation: Vec3::new(-12.5, 10.0, 22.5),
			rotation: Quat::from_rotation_y(FRAC_PI_2),
			..default()
		},
	));
	// Right wall
	cmds.spawn((
		wall_bundle.clone(),
		Transform {
			translation: Vec3::new(12.5, 10.0, 22.5),
			rotation: Quat::from_rotation_y(-FRAC_PI_2),
			..default()
		},
	));

	// Rear
	cmds.spawn((
		RigidBody::Static,
		Collider::cuboid(20.0, 5.0, 45.0),
		Mesh3d(meshes.add(Cuboid::new(20.0, 5.0, 45.0))),
		MeshMaterial3d(mats.add(StandardMaterial {
			base_color: Color::linear_rgb(0.02, 0.004, 0.03),
			reflectance: 0.01,
			..default()
		})),
		Transform {
			translation: Vec3::new(0.0, 15.0, 22.5),
			rotation: Quat::from_rotation_x(-FRAC_PI_8 * 0.5),
			..default()
		},
		Friction {
			dynamic_coefficient: 0.1,
			static_coefficient: 0.0,
			combine_rule: CoefficientCombine::Min,
		},
		Restitution::new(0.0).with_combine_rule(CoefficientCombine::Min),
	))
	.with_children(|cmds| {
		let peg_collider = Collider::cylinder(0.25, 0.5);
		let peg_mesh = meshes.add(Cylinder::new(0.25, 0.5));
		let peg_bundle = (
			RigidBody::Static,
			peg_collider,
			Mesh3d(peg_mesh.clone()),
			MeshMaterial3d(machine_mat.clone()),
		);
		for v in 0..7 {
			for h in 0..5 {
				let h_offset = 2.0 * (v % 2) as f32;
				cmds.spawn((
					peg_bundle.clone(),
					Transform {
						translation: Vec3::new(
							(h as f32 * 4.0) - 9.0 + h_offset,
							-2.75,
							(v as f32 * 4.0) - 10.0,
						),
						..default()
					},
					Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
					Restitution::new(0.9),
				));
			}
		}

		// Glass in front of pegs to prevent coins escaping plinko
		cmds.spawn((
			RigidBody::Static,
			Collider::cuboid(20.0, 1.0, 35.0),
			Transform {
				translation: Vec3::new(0.0, -3.375, 5.0),
				..default()
			},
			Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
		));

		cmds.spawn((
			DropZone,
			Collider::cuboid(20.0, 0.5, 2.0),
			Sensor,
			Transform {
				translation: Vec3::new(0.0, -2.625, 20.0),
				..default()
			},
		));
	});

	// Sliding platform
	cmds.spawn((
		RigidBody::Kinematic,
		Collider::cuboid(20.0, 30.0, 5.0),
		Mesh3d(meshes.add(Cuboid::new(20.0, 30.0, 5.0))),
		MeshMaterial3d(machine_mat.clone()),
		Transform {
			translation: Vec3::new(0.0, 10.0, 5.0),
			..default()
		},
		Piston {
			curve: Arc::new(
				EasingCurve::new(
					Vec3::new(0.0, 15.0, 5.0),
					Vec3::new(0.0, 0.0, 5.0),
					SineInOut,
				)
				.ping_pong()
				.unwrap()
				.forever()
				.unwrap(),
			),
			speed: 0.1,
		},
		Friction::new(1.0),
	));
}

#[derive(Component, Clone, Debug)]
#[require(Collider, Sensor)]
pub struct DropZone;

#[derive(Component, Clone)]
#[require(RigidBody(|| RigidBody::Kinematic), Collider)]
pub struct Piston {
	pub curve: Arc<dyn Curve<Vec3> + Send + Sync + 'static>,
	pub speed: f32,
}

pub fn move_piston(
	mut q: Query<(&mut Transform, &mut LinearVelocity, &Piston)>,
	t: Res<Time<Fixed>>,
) {
	let dt = t.timestep();
	for (mut xform, mut vel, piston) in q.iter_mut() {
		let Some((a, b)) = piston.curve.sample(t.elapsed_secs() * piston.speed).zip(
			piston
				.curve
				.sample((t.elapsed() + dt).as_secs_f32() * piston.speed),
		) else {
			error!(
				t = t.elapsed_secs(),
				speed = piston.speed,
				"Failed to sample curve"
			);
			return;
		};
		// Prevents drift from floating point imprecision
		xform.translation = a;
		// Velocity is needed, not just position, for friction to move coins.
		vel.0 = (b - a) / dt.as_secs_f32();
	}
}

pub fn collect(
	mut cmds: Commands,
	coins: Query<(Entity, &GlobalTransform, &Coin)>,
	mut winnings: ResMut<Winnings>,
) {
	for (id, xform, coin) in coins.iter() {
		if xform.translation().z < -20.0 {
			info!("Collecting {coin:?}");
			cmds.entity(id).despawn_recursive();
			winnings.0 = winnings.0.clone() + coin.value.clone();
			info!("Score: {}", winnings.0);
		}
	}
}
