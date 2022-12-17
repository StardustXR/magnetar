use crate::ring::{self, Ring};
use glam::Quat;
use rustc_hash::FxHashSet;
use stardust_xr_molecules::fusion::{
	client::LogicStepInfo,
	core::values::Transform,
	fields::CylinderField,
	node::NodeType,
	spatial::{Spatial, Zone, ZoneHandler},
	HandlerWrapper,
};
use std::f32::consts::PI;

pub struct Cell {
	_root: Spatial,
	_field: CylinderField,
	zone: Zone,
	pub active: bool,
	queued_zoneables: FxHashSet<String>,
	top_ring: Ring,
	bottom_ring: Ring,
}
impl Cell {
	pub fn new(parent: &Spatial, height: f32) -> HandlerWrapper<Zone, Self> {
		let root =
			Spatial::create(parent, Transform::from_position([0.0, height, 0.0]), false).unwrap();

		let field = CylinderField::create(
			&root,
			Transform::from_rotation(Quat::from_rotation_x(PI * 0.5)),
			1.0,
			1.0,
		)
		.unwrap();

		let top_ring = Ring::new_from_point(&root, 0.5, 1.0);
		let bottom_ring = Ring::new_from_point(&root, -0.5, 1.0);

		let zone = Zone::create(&root, Transform::default(), &field).unwrap();
		let cell = Cell {
			_root: root,
			_field: field,
			zone: zone.alias(),
			active: false,
			queued_zoneables: FxHashSet::default(),
			top_ring,
			bottom_ring,
		};
		zone.wrap(cell).unwrap()
	}

	pub fn logic_step(&mut self, info: LogicStepInfo) {
		self.top_ring.logic_step(info);
		self.active = match self.bottom_ring.logic_step(info) {
			ring::State::Idle => true,
			_ => false,
		};
		if self.active && !self.queued_zoneables.is_empty() {
			self.queued_zoneables
				.drain()
				.for_each(|zoneable| self.zone.capture(&zoneable).unwrap());
		}
	}
}
impl ZoneHandler for Cell {
	fn enter(&mut self, uid: &str, _spatial: Spatial) {
		dbg!(self.active);
		dbg!(uid);
		if self.active {
			self.zone.capture(uid).unwrap();
		} else {
			self.queued_zoneables.insert(uid.to_string());
		}
	}
	fn capture(&mut self, _uid: &str, _spatial: Spatial) {}
	fn release(&mut self, _uid: &str) {}
	fn leave(&mut self, uid: &str) {
		self.queued_zoneables.remove(uid);
	}
}
