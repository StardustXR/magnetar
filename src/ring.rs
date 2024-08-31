use std::f32::consts::PI;

use glam::Quat;
use mint::Vector3;
use stardust_xr_fusion::{
	core::values::color::rgba_linear,
	drawable::Lines,
	root::FrameInfo,
	spatial::{Spatial, SpatialAspect, Transform},
};
use stardust_xr_molecules::lines::{circle, LineExt};
use tween::{QuadIn, QuadOut, Tweener};

pub enum State {
	Rezzing {
		rez_height_tweener: Tweener<f32, f64, QuadIn>,
		rez_scale_tweener: Tweener<f32, f64, QuadOut>,
	},
	Idle,
	Derezzing,
	Derezzed,
}

pub struct Ring {
	lines: Lines,
	state: State,
}
impl Ring {
	pub fn new_from_point(parent: &Spatial, height: f32, radius: f32) -> Self {
		let circle =
			circle(128, 0.0, 1.0)
				.thickness(0.01)
				.color(rgba_linear!(0.392156863, 0.0, 1.0, 1.0));
		let lines = Lines::create(
			parent,
			Transform::from_rotation_scale(
				Quat::from_rotation_x(PI * 0.5),
				Vector3::from([0.02; 3]),
			),
			&[circle],
		)
		.unwrap();

		let state = State::Rezzing {
			rez_height_tweener: Tweener::quad_in(0.0, height, 0.25),
			rez_scale_tweener: Tweener::quad_out(0.02, radius, 0.25),
		};
		Ring { lines, state }
	}
	pub fn logic_step(&mut self, info: &FrameInfo) -> &State {
		match &mut self.state {
			State::Rezzing {
				rez_height_tweener,
				rez_scale_tweener,
			} => {
				if !rez_height_tweener.is_finished() {
					let height = rez_height_tweener.move_by(info.delta.into());
					let _ = self
						.lines
						.set_local_transform(Transform::from_translation([0.0, height, 0.0]));
				} else if !rez_scale_tweener.is_finished() {
					let scale = rez_scale_tweener.move_by(info.delta.into());
					let _ = self
						.lines
						.set_local_transform(Transform::from_scale([scale; 3]));
				} else {
					self.state = State::Idle
				}
			}
			State::Idle => (),
			State::Derezzing => (),
			_ => (),
		}
		&self.state
	}
}
