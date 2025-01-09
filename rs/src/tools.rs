use avian3d::debug_render::PhysicsGizmos;
use avian3d::prelude::PhysicsDebugPlugin;
use bevy::color::palettes::basic::YELLOW;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(PhysicsDebugPlugin::default())
			.add_systems(Update, (debug_toggles, update_fps.never_param_warn()));
	}

	fn finish(&self, app: &mut App) {
		app.world_mut()
			.resource_mut::<GizmoConfigStore>()
			.config_mut::<PhysicsGizmos>()
			.0
			.enabled = false;
	}
}

pub fn debug_toggles(
	mut cmds: Commands,
	fps_text: Option<Single<Entity, With<FpsText>>>,
	keys: Res<ButtonInput<KeyCode>>,
	mut gizmos: ResMut<GizmoConfigStore>,
) {
	// FPS
	if keys.just_pressed(KeyCode::F10) {
		if let Some(&fps) = fps_text.as_deref() {
			info!("FPS counter: OFF");
			cmds.entity(fps).despawn();
		} else {
			info!("FPS counter: ON");
			cmds.spawn((
				FpsText,
				Text::default(),
				TextFont::default(),
				TextColor(YELLOW.into()),
			));
		}
	}

	if keys.just_pressed(KeyCode::Backquote) {
		let (cfg, _) = gizmos.config_mut::<PhysicsGizmos>();
		cfg.enabled = !cfg.enabled;
	}
}

#[derive(Component)]
pub struct FpsText;

pub fn update_fps(mut fps_text: Single<&mut Text, With<FpsText>>, diags: Res<DiagnosticsStore>) {
	let Some(fps) = diags.get(&FrameTimeDiagnosticsPlugin::FPS) else {
		return;
	};
	let Some(fps) = fps.smoothed() else { return };
	fps_text.0 = format!("FPS: {fps:.2}");
}
