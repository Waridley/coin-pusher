use crate::machine::DropZone;
use avian3d::collision::{ColliderAabb, Collisions};
use avian3d::math::PI;
use avian3d::prelude::{AngularDamping, LinearDamping, Restitution};
use avian3d::{collision::Collider, dynamics::rigid_body::RigidBody};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::component::ComponentId;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use currency::{Currency, ParseCurrencyError};
use rand::random;
use std::collections::VecDeque;
use std::ops::Not;
use std::time::{Duration, Instant};

pub struct CoinsPlugin;

impl Plugin for CoinsPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<DropCoin>()
			.init_resource::<AutoDrop>()
			.init_resource::<AutoDropTimer>()
			.init_resource::<CoinQueue>()
			.init_resource::<CoinCount>()
			.add_systems(Startup, setup_coins)
			.add_systems(
				FixedUpdate,
				(drop_coins, auto_drop_coins.run_if(AutoDrop::is_enabled)),
			);
	}
}

pub fn setup_coins(mut cmds: Commands, asset_server: Res<AssetServer>) {
	let handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("coin.glb"));
	cmds.insert_resource(CoinScene(handle));
}

#[derive(Component, Debug, Clone)]
#[require(RigidBody, Collider(|| Collider::cylinder(1.0, 0.25)), Mesh3d, MeshMaterial3d<StandardMaterial>)]
#[component(on_add = increment_coin_count, on_remove = decrement_coin_count)]
pub struct Coin {
	pub value: Currency,
}

impl Default for Coin {
	fn default() -> Self {
		Self {
			value: Currency::from_str("$1.00").unwrap(),
		}
	}
}

#[derive(Resource, Debug, Clone)]
pub struct CoinScene(Handle<Scene>);

#[derive(Event, Debug, Clone)]
pub struct DropCoin {
	pub coin: Coin,
	pub reason: CoinDropReason,
}

impl DropCoin {
	pub fn auto(value: &str) -> Result<Self, ParseCurrencyError> {
		Ok(Self {
			coin: Coin {
				value: Currency::from_str(value)?,
			},
			reason: CoinDropReason::Auto,
		})
	}

	pub fn manual(value: &str) -> Result<Self, ParseCurrencyError> {
		Ok(Self {
			coin: Coin {
				value: Currency::from_str(value)?,
			},
			reason: CoinDropReason::Manual,
		})
	}
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum CoinDropReason {
	Auto,
	Manual,
}

fn drop_coins(
	mut cmds: Commands,
	mut events: ResMut<Events<DropCoin>>,
	drop_zone: Single<(Entity, &GlobalTransform, &ColliderAabb), With<DropZone>>,
	collisions: Res<Collisions>,
	coins: Query<Entity, With<Coin>>,
	coin_scene: Res<CoinScene>,
	diags: Res<DiagnosticsStore>,
	mut queue: ResMut<CoinQueue>,
	mut last_fps_warn: Local<Option<Instant>>,
	mut auto_drop_timer: ResMut<AutoDropTimer>,
	auto_drop: Res<AutoDrop>,
) {
	if auto_drop.is_changed() && !**auto_drop {
		// Would be confusing to keep auto-dropping after it is disabled.
		queue.retain(|ev| ev.reason != CoinDropReason::Auto);
	}

	let dz = drop_zone.2.size();
	let dz_center = drop_zone.2.center(); // AABB is in global coords
	let coin_dia = 2.0;
	let h_range = dz.x - coin_dia;
	let v_range = dz.y - coin_dia;

	let mut events = events.drain();

	let ev = events.next();
	let mut to_spawn = queue.drain(..).chain(ev);
	let failed_coin = (|| {
		let DropCoin { coin, reason } = to_spawn.next()?;
		let h = random::<f32>() * h_range - (0.5 * h_range);
		let v = random::<f32>() * v_range - (0.5 * v_range);

		if let Some(fps) = diags.get(&FrameTimeDiagnosticsPlugin::FPS) {
			if let Some(fps) = fps.smoothed() {
				if fps < 24.0 {
					let now = Instant::now();
					let last_warn = if let Some(last_fps_warn) = &*last_fps_warn {
						now.duration_since(*last_fps_warn)
					} else {
						Duration::MAX
					};
					if last_warn.as_secs() >= 1 {
						let last_warn = last_fps_warn.map(|_| last_warn);
						warn!(
							fps,
							?last_warn,
							"Performance is too low, not spawning another coin."
						);
						*last_fps_warn = Some(now);
					}
					return Some(DropCoin { coin, reason });
				}
			}
		}
		for col in collisions.collisions_with_entity(drop_zone.0) {
			if coins.contains(col.entity1) || coins.contains(col.entity2) {
				// Another coin might overlap, wait for it to clear
				//     *alternatively, we could  try to spawn beside any coins in the drop zone
				trace!("Not spawning because another coin is in the drop zone");
				return Some(DropCoin { coin, reason });
			}
		}
		info!(?h, ?v, ?reason, "Dropping coin...");
		cmds.spawn((
			coin,
			SceneRoot(coin_scene.0.clone()),
			Transform {
				translation: Vec3::new(dz_center.x + h, dz_center.y, dz_center.z + v),
				rotation: drop_zone.1.rotation() * Quat::from_rotation_x(PI),
				..default()
			},
			Restitution::new(1.0),
			LinearDamping(0.05),
			AngularDamping(0.05),
		));
		auto_drop_timer.reset();
		None
	})();

	drop(to_spawn);

	if let Some(failed) = failed_coin {
		queue.push_front(failed);
	}

	for ev in events {
		queue.push_back(ev);
	}
}

#[derive(Resource, Debug, Clone, Default, Deref, DerefMut)]
pub struct CoinQueue(VecDeque<DropCoin>);

#[derive(Resource, Default, Debug, Copy, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct AutoDrop(bool);

impl AutoDrop {
	pub fn is_enabled(this: Res<Self>) -> bool {
		this.0
	}
}

impl Not for AutoDrop {
	type Output = Self;
	fn not(self) -> Self::Output {
		Self(!self.0)
	}
}

#[derive(Resource, Debug, Clone, Deref, DerefMut)]
pub struct AutoDropTimer(Timer);

impl Default for AutoDropTimer {
	fn default() -> Self {
		let mut timer = Timer::new(Duration::from_secs(2), TimerMode::Once);
		timer.set_elapsed(Duration::from_secs(2));
		Self(timer)
	}
}

pub fn auto_drop_coins(
	mut events: EventWriter<DropCoin>,
	mut timer: ResMut<AutoDropTimer>,
	t: Res<Time>,
) {
	timer.tick(t.delta());
	if timer.finished() {
		events.send(DropCoin::auto("$1.00").unwrap());
	}
}

#[derive(Resource, Debug, Default)]
pub struct CoinCount(pub(crate) usize);

pub fn increment_coin_count(mut world: DeferredWorld, _: Entity, _: ComponentId) {
	world.resource_mut::<CoinCount>().0 += 1;
}

pub fn decrement_coin_count(mut world: DeferredWorld, _: Entity, _: ComponentId) {
	world.resource_mut::<CoinCount>().0 -= 1;
}
