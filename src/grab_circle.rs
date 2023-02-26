use color::rgba;
use glam::{vec2, Quat};
use map_range::MapRange;
use stardust_xr_fusion::{
	core::values::Transform,
	drawable::Lines,
	input::{InputData, InputDataType, InputHandler},
};
use stardust_xr_molecules::lines::circle;

pub struct GrabCircle {
	lines: Lines,
	radius: f32,
}
impl GrabCircle {
	pub fn new(input_handler: &InputHandler, radius: f32) -> Self {
		let points = circle(64, 0.5, 0.005, rgba!(1.0, 1.0, 1.0, 1.0));
		GrabCircle {
			lines: Lines::create(input_handler, Transform::default(), &points, true).unwrap(),
			radius,
		}
	}

	pub fn update(
		&self,
		input_data: &InputData,
		_in_range_data: Option<&InputData>,
		grabbing: bool,
	) {
		let interact_point = match &input_data.input {
			InputDataType::Pointer(p) => p.deepest_point,
			InputDataType::Hand(h) => h.palm.position,
			InputDataType::Tip(t) => t.origin,
		};
		let interact_direction = vec2(interact_point.x, interact_point.z).normalize_or_zero();
		let xz_position = interact_direction * self.radius;
		let position = [xz_position.x, interact_point.y, xz_position.y];
		let rotation = Quat::from_rotation_y(-vec2(0.0, 1.0).angle_between(interact_direction));
		let scale = if !grabbing {
			input_data
				.distance
				.abs()
				.map_range(0.1..0.05, 0.0..0.1)
				.clamp(0.0, 0.1)
		} else {
			0.1
		};

		self.lines
			.set_transform(
				None,
				Transform::from_position_rotation_scale(position, rotation, [scale; 3]),
			)
			.unwrap();
	}
}
