use std::time::Instant;
use avian3d::math::PI;
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
			.add_systems(FixedUpdate, drop_coins)
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

pub fn drop_coins(
	mut cmds: Commands,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	mut mouse_input: EventReader<MouseButtonInput>,
	drop_zone: Single<(Entity, &GlobalTransform, &ColliderAabb), With<DropZone>>,
	collisions: Res<Collisions>,
	coins: Query<Entity, With<Coin>>,
	diags: Res<DiagnosticsStore>,
	mut auto: Local<bool>,
	mut last_fps_warn: Local<Option<Instant>>,
	mut auto_drop_timer: Local<Timer>,
	t: Res<Time>,
) {
	let dz = drop_zone.2.size();
	let dz_center = drop_zone.2.center(); // AABB is in global coords
	let coin_dia = 2.0;
	let h_range = dz.x - coin_dia;
	let v_range = dz.y - coin_dia;

	let mut spawn_coin = |reason: &str| {
		let h = random::<f32>() * h_range - (0.5 * h_range);
		let v = random::<f32>() * v_range - (0.5 * v_range);

		if let Some(fps) = diags.get(&FrameTimeDiagnosticsPlugin::FPS) {
			if let Some(fps) = fps.smoothed() {
				if fps < 24.0 {
					let now = Instant::now();
					let last_warn = if let Some(last_fps_warn) = &*last_fps_warn {
						now.duration_since(*last_fps_warn)
					} else {
						std::time::Duration::MAX
					};
					if last_warn.as_secs() >= 1 {
						let last_warn = last_fps_warn.map(|_| last_warn);
						warn!(fps, ?last_warn, "Performance is too low, not spawning another coin.");
						*last_fps_warn = Some(now);
					}
					return false;
				}
			}
		}
		for col in collisions.collisions_with_entity(drop_zone.0) {
			if coins.contains(col.entity1) || coins.contains(col.entity2) {
				// Another coin might overlap, wait for it to clear
				//     *alternatively, we could  try to spawn beside any coins in the drop zone
				trace!("Not spawning because another coin is in the drop zone");
				return false;
			}
		}
		info!(?h, ?v, reason, "Dropping coin...");
		cmds.spawn((
			Coin::default(),
			SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("coin.glb"))),
			Transform {
				translation: Vec3::new(dz_center.x + h, dz_center.y, dz_center.z + v),
				rotation: drop_zone.1.rotation() * Quat::from_rotation_x(PI),
				..default()
			},
			Restitution::new(1.0),
			LinearDamping(0.05),
			AngularDamping(0.05),
		));
		true
	};

	for click in mouse_input.read() {
		if click.button == MouseButton::Left && click.state == ButtonState::Pressed {
			if *auto {
				info!("Auto: OFF");
				*auto = false;
			}
			spawn_coin("click");
		} else if click.button == MouseButton::Right && click.state == ButtonState::Pressed {
			*auto = !*auto;
			if *auto {
				auto_drop_timer.set_duration(std::time::Duration::from_secs(1));
				auto_drop_timer.set_elapsed(std::time::Duration::from_secs(1));
				info!("Auto: ON");
			} else {
				info!("Auto: OFF");
			}
		}
	}

	if *auto {
		auto_drop_timer.tick(t.delta());
		if auto_drop_timer.finished() {
			if spawn_coin("auto") {
				auto_drop_timer.reset();
			}
		}
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
