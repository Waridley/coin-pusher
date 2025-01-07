use avian3d::{collision::Collider, dynamics::rigid_body::RigidBody};
use bevy::prelude::*;
use currency::Currency;

#[derive(Component, Debug, Clone)]
#[require(RigidBody, Collider(|| Collider::cylinder(1.0, 0.25)), Mesh3d, MeshMaterial3d<StandardMaterial>)]
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
