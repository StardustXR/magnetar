use crate::{cell::Cell, grab_circle::GrabCircle};
use color_eyre::eyre::Result;
use glam::Quat;
use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	client::{Client, ClientState, FrameInfo, RootHandler},
	fields::{CylinderField, CylinderFieldAspect},
	input::{InputData, InputDataType, InputHandler},
	spatial::{Spatial, SpatialAspect, Transform, Zone},
	HandlerWrapper,
};
use stardust_xr_molecules::input_action::{BaseInputAction, InputActionHandler, SingleActorAction};
use std::f32::consts::PI;

pub struct Magnetar {
	root: Spatial,
	field: CylinderField,
	y_pos_tmp: f32,
	y_pos: f32,
	y_offset: f32,
	input_handler: HandlerWrapper<InputHandler, InputActionHandler<()>>,
	always_input_action: BaseInputAction<()>,
	hover_input_action: BaseInputAction<()>,
	grab_input_action: SingleActorAction<()>,
	cells: Vec<HandlerWrapper<Zone, Cell>>,
	grab_circles: FxHashMap<String, GrabCircle>,
}
impl Magnetar {
	pub fn new(client: &Client) -> Result<Self> {
		let root = Spatial::create(client.get_root(), Transform::identity(), false)?;

		let field = CylinderField::create(
			&root,
			Transform::from_rotation(Quat::from_rotation_x(PI * 0.5)),
			0.0,
			1.0,
		)?;

		let input_handler = InputActionHandler::wrap(
			InputHandler::create(client.get_root(), Transform::identity(), &field)?,
			(),
		)?;
		let always_input_action = BaseInputAction::new(false, |_, _| true);
		let hover_input_action =
			BaseInputAction::new(false, |input_data, _| input_data.distance.abs() < 0.05);
		let grab_input_action = SingleActorAction::new(true, Magnetar::grab_action, true);
		Ok(Magnetar {
			root,
			field,
			y_pos_tmp: 0.0,
			y_pos: 0.0,
			y_offset: 0.0,
			input_handler,
			always_input_action,
			hover_input_action,
			grab_input_action,
			cells: vec![],
			grab_circles: FxHashMap::default(),
		})
	}
	pub fn add_cell(&mut self) {
		self.cells
			.push(Cell::new(&self.root, -(self.cells.len() as f32)));
		let cells_height = self.cells.len() as f32;
		self.field
			.set_local_transform(Transform::from_translation([
				0.0,
				cells_height * 0.5 - 0.5,
				0.0,
			]))
			.unwrap();
		self.field.set_size(cells_height, 1.0).unwrap();
	}

	fn grab_action(input_data: &InputData, _: &()) -> bool {
		input_data
			.datamap
			.with_data(|data| match &input_data.input {
				InputDataType::Hand(_) => data.idx("grab_strength").as_f32() > 0.8,
				_ => data.idx("grab").as_f32() > 0.9,
			})
	}

	fn update_grab_circles(&mut self) {
		for input in &self.always_input_action.started_acting {
			self.grab_circles.insert(
				input.uid.clone(),
				GrabCircle::new(self.input_handler.node(), 1.0),
			);
		}
		for input in &self.always_input_action.currently_acting {
			let grabbing = self.grab_input_action.actor_acting()
				&& self.grab_input_action.actor().as_ref().unwrap().uid == input.uid;
			self.grab_circles
				.get(&input.uid)
				.unwrap()
				.update(input, None, grabbing);
		}
		for input in &self.always_input_action.stopped_acting {
			self.grab_circles.remove(&input.uid);
		}
	}
}
impl RootHandler for Magnetar {
	fn frame(&mut self, info: FrameInfo) {
		for cell in &self.cells {
			cell.lock_wrapped().logic_step(info);
		}

		self.input_handler.lock_wrapped().update_actions([
			&mut self.always_input_action,
			&mut self.hover_input_action,
			self.grab_input_action.base_mut(),
		]);
		self.grab_input_action
			.update(Some(&mut self.hover_input_action));

		self.update_grab_circles();

		if self.grab_input_action.actor_started() {
			for cell in &self.cells {
				cell.lock_wrapped().active = false;
			}
		}
		if self.grab_input_action.actor_acting() {
			let y = y_pos(self.grab_input_action.actor().unwrap());

			if self.grab_input_action.actor_started() {
				self.y_offset = y;
			}

			self.y_pos_tmp = y - self.y_offset + self.y_pos;

			self.root
				.set_local_transform(Transform::from_translation([0.0, self.y_pos_tmp, 0.0]))
				.unwrap();
		}
		if self.grab_input_action.actor_stopped() {
			for cell in &self.cells {
				cell.lock_wrapped().active = true;
			}
			self.y_pos = self.y_pos_tmp;
		}

		if let Some(scroll) = self
			.hover_input_action
			.currently_acting
			.iter()
			.map(|input| {
				input
					.datamap
					.with_data(|data| data.idx("scroll").as_vector().idx(1).as_f32())
					* 0.1
			})
			.filter(|distance| distance.abs() > f32::EPSILON)
			.reduce(|a, b| a + b)
		{
			self.y_offset -= scroll;
			self.y_pos += scroll;
			self.y_pos_tmp += scroll;

			self.root
				.set_local_transform(Transform::from_translation([0.0, self.y_pos_tmp, 0.0]))
				.unwrap();
		}
	}

	fn save_state(&mut self) -> ClientState {
		ClientState::default()
	}
}

fn y_pos(data: &InputData) -> f32 {
	match &data.input {
		InputDataType::Pointer(p) => p.deepest_point.y,
		InputDataType::Hand(h) => h.palm.position.y,
		InputDataType::Tip(t) => t.origin.y,
	}
}
