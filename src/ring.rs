use color::rgba;
use glam::Quat;
use mint::Vector3;
use stardust_xr_molecules::{
	fusion::{client::LogicStepInfo, drawable::Lines, spatial::Spatial},
	lines::circle,
};
use std::f32::consts::PI;
use tween::{QuadIn, QuadOut, Tweener};

pub enum State {
	Rezzing {
		rez_height_tweener: Tweener<QuadIn<f32, f64>>,
		rez_scale_tweener: Tweener<QuadOut<f32, f64>>,
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
		let circle_points = circle(128, 1.0, 0.01, rgba!(0.392156863, 0.0, 1.0, 1.0));
		let lines = Lines::builder()
			.spatial_parent(parent)
			.points(&circle_points)
			.rotation(Quat::from_rotation_x(PI * 0.5))
			.scale(Vector3::from([0.02; 3]))
			.cyclic(true)
			.build()
			.unwrap();

		let state = State::Rezzing {
			rez_height_tweener: Tweener::new(QuadIn::new(0.0..=height, 0.25)),
			rez_scale_tweener: Tweener::new(QuadOut::new(0.02..=radius, 0.25)),
		};
		Ring { lines, state }
	}
	pub fn logic_step(&mut self, info: LogicStepInfo) -> &State {
		match &mut self.state {
			State::Rezzing {
				rez_height_tweener,
				rez_scale_tweener,
			} => {
				if let Some(height) = rez_height_tweener.update(info.delta) {
					let _ = self
						.lines
						.set_position(None, Vector3::from([0.0, height, 0.0]));
				} else if let Some(scale) = rez_scale_tweener.update(info.delta) {
					let _ = self.lines.set_scale(None, Vector3::from([scale; 3]));
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
