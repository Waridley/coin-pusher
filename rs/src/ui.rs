use crate::cam::{CamSwivel, CamTilter};
use crate::coins::{AutoDrop, AutoDropTimer, CoinCount, DropCoin};
use crate::Winnings;
use bevy::color::palettes::basic::{LIME, RED, YELLOW};
use bevy::color::palettes::css::GOLD;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::input::ButtonState;
use bevy::prelude::*;
use std::time::Duration;

pub struct UiPlugin;

impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup_ui)
			.add_systems(FixedUpdate, drop_coins)
			.add_systems(
				Update,
				(
					dev_cam,
					update_winnings_text,
					update_auto_text,
					update_coin_count_text,
					adjust_auto_timer,
				),
			);
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

	cmds.spawn(Node {
		justify_self: JustifySelf::End,
		flex_direction: FlexDirection::Column,
		..default()
	})
	.with_children(|cmds| {
		cmds.spawn((
			Text("Left click: Drop coin".into()),
			TextFont::from_font_size(24.0),
			TextColor::WHITE,
			Node {
				align_self: AlignSelf::End,
				..default()
			},
		));
		cmds.spawn((
			Text("Right click: Toggle auto".into()),
			TextFont::from_font_size(24.0),
			TextColor::WHITE,
			Node {
				align_self: AlignSelf::End,
				..default()
			},
		));
		cmds.spawn((
			Text("Up/Down: Move camera".into()),
			TextFont::from_font_size(24.0),
			TextColor::WHITE,
			Node {
				align_self: AlignSelf::End,
				..default()
			},
		));
		cmds.spawn((
			Text("+/-: Adjust auto timer".into()),
			TextFont::from_font_size(24.0),
			TextColor::WHITE,
			Node {
				align_self: AlignSelf::End,
				..default()
			},
		));

		cmds.spawn((
			AutoText {
				enabled: false,
				duration: Duration::from_secs(2),
			},
			Text("Auto: OFF".into()),
			TextFont::from_font_size(24.0),
			TextColor(RED.into()),
			Node {
				align_self: AlignSelf::End,
				..default()
			},
		));

		cmds.spawn((
			CoinCountText,
			Text("Coins: 0".into()),
			TextFont::from_font_size(24.0),
			TextColor(YELLOW.into()),
			Node {
				align_self: AlignSelf::End,
				..default()
			},
		));
	});
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

#[derive(Component, Debug)]
pub struct AutoText {
	enabled: bool,
	duration: Duration,
}

pub fn update_auto_text(
	mut q: Single<(&mut Text, &mut TextColor, &mut AutoText)>,
	auto_enabled: Res<AutoDrop>,
	auto_timer: Res<AutoDropTimer>,
) {
	if **auto_enabled == q.2.enabled && auto_timer.duration() == q.2.duration {
		return;
	}

	if **auto_enabled {
		q.2.enabled = true;
		let duration = auto_timer.duration();
		q.2.duration = duration;
		q.0 .0 = format!("Auto: {duration:?}");
		q.1 .0 = LIME.into();
	} else {
		q.2.enabled = false;
		q.0 .0 = "Auto: OFF".into();
		q.1 .0 = RED.into();
	}
}

#[derive(Component, Debug)]
pub struct CoinCountText;

pub fn update_coin_count_text(
	mut q: Single<&mut Text, With<CoinCountText>>,
	count: Res<CoinCount>,
) {
	if count.is_changed() {
		q.0 = format!("Coins: {}", count.0);
	}
}

pub fn adjust_auto_timer(mut events: EventReader<KeyboardInput>, mut timer: ResMut<AutoDropTimer>) {
	for ev in events.read() {
		if ev.state == ButtonState::Pressed {
			let curr = timer.duration();
			let curr_idx = TIMER_VALUES.binary_search(&curr).unwrap_or(2);
			let new = match ev.key_code {
				KeyCode::Equal | KeyCode::NumpadAdd => {
					TIMER_VALUES[(curr_idx + 1).min(TIMER_VALUES.len() - 1)]
				}
				KeyCode::Minus | KeyCode::NumpadSubtract => {
					TIMER_VALUES[curr_idx.saturating_sub(1)]
				}
				_ => continue,
			};
			timer.set_duration(new);
		}
	}
}

pub const TIMER_VALUES: &[Duration] = &[
	Duration::from_millis(500),
	Duration::from_secs(1),
	Duration::from_secs(2),
	Duration::from_secs(3),
	Duration::from_secs(4),
	Duration::from_secs(5),
	Duration::from_secs(10),
	Duration::from_secs(15),
	Duration::from_secs(20),
	Duration::from_secs(25),
	Duration::from_secs(30),
	Duration::from_secs(45),
	Duration::from_secs(60),
	Duration::from_secs(90),   // 1m 30s
	Duration::from_secs(120),  // 2m
	Duration::from_secs(300),  // 5m
	Duration::from_secs(600),  // 10m
	Duration::from_secs(900),  // 15m
	Duration::from_secs(1800), // 30m
	Duration::from_secs(2700), // 45m
	Duration::from_secs(3600), // 1h
];
