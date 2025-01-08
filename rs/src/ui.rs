use crate::cam::{CamSwivel, CamTilter};
use crate::coins::{AutoDrop, DropCoin};
use crate::Winnings;
use bevy::color::palettes::css::GOLD;
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::input::ButtonState;
use bevy::prelude::*;

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
	mut events: EventWriter<DropCoin>,
	mut mouse_input: EventReader<MouseButtonInput>,
	mut auto: ResMut<AutoDrop>,
) {
	for click in mouse_input.read() {
		if click.button == MouseButton::Left && click.state == ButtonState::Pressed {
			if **auto {
				info!("Auto: OFF");
				**auto = false;
			}
			events.send(DropCoin::manual("$1.00").unwrap());
		} else if click.button == MouseButton::Right && click.state == ButtonState::Pressed {
			*auto = !*auto;
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
