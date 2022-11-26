use crate::cell::Cell;
use glam::Quat;
use mint::Vector3;
use stardust_xr_molecules::{
	fusion::{
		client::{Client, LifeCycleHandler, LogicStepInfo},
		fields::CylinderField,
		input::{
			action::{BaseInputAction, InputAction, InputActionHandler},
			InputData, InputDataType, InputHandler,
		},
		spatial::{Spatial, Zone},
		HandlerWrapper,
	},
	SingleActorAction,
};
use std::f32::consts::PI;

pub struct Magnetar {
	root: Spatial,
	field: CylinderField,
	y_pos_tmp: f32,
	y_pos: f32,
	y_offset: f32,
	input_handler: HandlerWrapper<InputHandler, InputActionHandler<()>>,
	hover_input_action: BaseInputAction<()>,
	grab_input_action: SingleActorAction<()>,
	cells: Vec<HandlerWrapper<Zone, Cell>>,
}
impl Magnetar {
	pub fn new(client: &Client) -> Self {
		let root = Spatial::builder()
			.spatial_parent(client.get_root())
			.zoneable(false)
			.build()
			.unwrap();

		let field = CylinderField::builder()
			.spatial_parent(&root)
			.rotation(Quat::from_rotation_x(PI * 0.5))
			.length(0.0)
			.radius(1.0)
			.build()
			.unwrap();

		let input_handler = InputHandler::create(&client.get_root(), None, None, &field, |_, _| {
			InputActionHandler::new(())
		})
		.unwrap();
		let hover_input_action =
			BaseInputAction::new(false, |input_data, _| input_data.distance.abs() < 0.05);
		let grab_input_action = SingleActorAction::new(true, Magnetar::grab_action, true);
		Magnetar {
			root,
			field,
			y_pos_tmp: 0.0,
			y_pos: 0.0,
			y_offset: 0.0,
			input_handler,
			hover_input_action,
			grab_input_action,
			cells: vec![],
		}
	}
	pub fn add_cell(&mut self) {
		self.cells
			.push(Cell::new(&self.root, -(self.cells.len() as f32)));
		let cells_height = self.cells.len() as f32;
		self.field
			.set_position(None, Vector3::from([0.0, cells_height * 0.5 - 0.5, 0.0]))
			.unwrap();
		self.field.set_size(cells_height, 1.0).unwrap();
	}

	fn grab_action(input_data: &InputData, _: &()) -> bool {
		input_data
			.datamap
			.with_data(|data| match &input_data.input {
				InputDataType::Hand(_) => data.idx("grabStrength").as_f32() > 0.8,
				_ => data.idx("grab").as_f32() > 0.9,
			})
	}
}
impl LifeCycleHandler for Magnetar {
	fn logic_step(&mut self, info: LogicStepInfo) {
		for cell in &self.cells {
			cell.lock_inner().logic_step(info);
		}

		self.input_handler.lock_inner().update_actions([
			self.hover_input_action.type_erase(),
			self.grab_input_action.type_erase(),
		]);
		self.grab_input_action.update(&mut self.hover_input_action);

		if self.grab_input_action.actor_started() {
			for cell in &self.cells {
				cell.lock_inner().active = false;
			}
		}
		if self.grab_input_action.actor_acting() {
			let y = y_pos(self.grab_input_action.actor().unwrap());

			if self.grab_input_action.actor_started() {
				self.y_offset = y;
			}

			self.y_pos_tmp = y - self.y_offset + self.y_pos;

			self.root
				.set_position(None, Vector3::from([0.0, self.y_pos_tmp, 0.0]))
				.unwrap();
		}
		if self.grab_input_action.actor_stopped() {
			for cell in &self.cells {
				cell.lock_inner().active = true;
			}
			self.y_pos = self.y_pos_tmp;
		}

		if let Some(scroll) = self
			.hover_input_action
			.actively_acting
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
				.set_position(None, Vector3::from([0.0, self.y_pos_tmp, 0.0]))
				.unwrap();
		}
	}
}

fn y_pos(data: &InputData) -> f32 {
	match &data.input {
		InputDataType::Pointer(p) => p.deepest_point.y,
		InputDataType::Hand(h) => h.palm.position.y,
		InputDataType::Tip(t) => t.origin.y,
	}
}
