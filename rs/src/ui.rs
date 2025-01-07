use crate::cam::{CamSwivel, CamTilter};
use crate::coins::Coin;
use crate::machine::DropZone;
use crate::Winnings;
use avian3d::prelude::*;
use bevy::color::palettes::css::GOLD;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::input::ButtonState;
use bevy::prelude::*;
use rand::random;

pub struct UiPlugin;

impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup_ui)
			.add_systems(FixedUpdate, on_click)
			.add_systems(Update, (dev_cam, update_winnings_text));
	}
}

pub fn setup_ui(mut cmds: Commands) {
	cmds.spawn((
		BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.7)),
		Node {
			bottom: Val::Px(20.0),
			justify_self: JustifySelf::End,
			align_self: AlignSelf::End,
			..default()
		},
	))
	.with_child((
		WinningsText,
		Text("$0.00".into()),
		TextFont {
			font_size: 60.0,
			..default()
		},
		TextColor(GOLD.into()),
	));
}

pub fn on_click(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	mut mouse_input: EventReader<MouseButtonInput>,
	drop_zone: Single<(Entity, &GlobalTransform, &ColliderAabb), With<DropZone>>,
	collisions: Res<Collisions>,
	coins: Query<Entity, With<Coin>>,
	diags: Res<DiagnosticsStore>,
	mut auto: Local<bool>,
) {
	let dz = drop_zone.2.size();
	let coin_dia = 2.0;
	let h_range = dz.x - coin_dia;
	let v_range = dz.y - coin_dia;
	let h = random::<f32>() * h_range - (0.5 * h_range);
	let v = random::<f32>() * v_range - (0.5 * v_range);
	let dz_center = drop_zone.2.center(); // AABB is in global coords

	let mut spawn_coin = || {
		if let Some(fps) = diags.get(&FrameTimeDiagnosticsPlugin::FPS) {
			if let Some(fps) = fps.smoothed() {
				if fps < 24.0 {
					warn!(fps, "Performance is too low, not spawning another coin");
					return;
				}
			}
		}
		for col in collisions.collisions_with_entity(drop_zone.0) {
			if coins.contains(col.entity1) || coins.contains(col.entity2) {
				// Another coin might overlap, wait for it to clear
				//     *alternatively, we could  try to spawn beside any coins in the drop zone
				trace!("Not spawning because another coin is in the drop zone");
				return;
			}
		}
		cmds.spawn((
			Coin::default(),
			Mesh3d(meshes.add(Cylinder::new(1.0, 0.25))),
			MeshMaterial3d(mats.add(StandardMaterial {
				base_color: Color::from(GOLD),
				metallic: 1.0,
				..default()
			})),
			Transform {
				translation: Vec3::new(dz_center.x + h, dz_center.y, dz_center.z + v),
				rotation: drop_zone.1.rotation(),
				..default()
			},
			LinearDamping(0.2),
			AngularDamping(0.2),
			Restitution::new(0.9),
		));
	};

	for click in mouse_input.read() {
		if click.button == MouseButton::Left && click.state == ButtonState::Pressed {
			if *auto {
				*auto = false;
			}
			spawn_coin();
		} else if click.button == MouseButton::Right && click.state == ButtonState::Pressed {
			*auto = !*auto;
		}
	}

	if *auto {
		spawn_coin();
	}
}

#[derive(Component, Debug)]
pub struct WinningsText;

pub fn update_winnings_text(mut q: Single<&mut Text, With<WinningsText>>, winnings: Res<Winnings>) {
	if winnings.is_changed() {
		q.0 = format!("{}", winnings.0);
	}
}

pub fn dev_cam(
	mut swivel: Single<&mut Transform, With<CamSwivel>>,
	mut tilt: Single<&mut Transform, (With<CamTilter>, Without<CamSwivel>)>,
	mut mouse_motion: EventReader<MouseMotion>,
	mouse_buttons: Res<ButtonInput<MouseButton>>,
	t: Res<Time>,
) {
	if mouse_buttons.pressed(MouseButton::Middle) {
		for ev in mouse_motion.read() {
			swivel.rotation *= Quat::from_rotation_z(-ev.delta.x * t.delta_secs());
			tilt.rotation *= Quat::from_rotation_x(-ev.delta.y * t.delta_secs());
		}
	}
}
