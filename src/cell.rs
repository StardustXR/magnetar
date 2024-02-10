use crate::ring::{self, Ring};
use glam::Quat;
use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	client::FrameInfo,
	fields::CylinderField,
	node::NodeType,
	spatial::{Spatial, SpatialAspect, Transform, Zone, ZoneAspect, ZoneHandler},
	HandlerWrapper,
};
use std::f32::consts::PI;

pub struct Cell {
	root: Spatial,
	_field: CylinderField,
	zone: Zone,
	pub active: bool,
	zoneables: FxHashMap<String, Spatial>,
	queued_zoneables: Vec<Spatial>,
	top_ring: Ring,
	bottom_ring: Ring,
}
impl Cell {
	pub fn new(parent: &Spatial, height: f32) -> HandlerWrapper<Zone, Self> {
		let root = Spatial::create(
			parent,
			Transform::from_translation([0.0, height, 0.0]),
			false,
		)
		.unwrap();

		let field = CylinderField::create(
			&root,
			Transform::from_rotation(Quat::from_rotation_x(PI * 0.5)),
			1.0,
			1.0,
		)
		.unwrap();

		let top_ring = Ring::new_from_point(&root, 0.5, 1.0);
		let bottom_ring = Ring::new_from_point(&root, -0.5, 1.0);

		let zone = Zone::create(&root, Transform::identity(), &field).unwrap();
		let cell = Cell {
			root,
			_field: field,
			zone: zone.alias(),
			active: false,
			zoneables: FxHashMap::default(),
			queued_zoneables: Vec::new(),
			top_ring,
			bottom_ring,
		};
		zone.wrap(cell).unwrap()
	}

	pub fn logic_step(&mut self, info: FrameInfo) {
		self.zone.update().unwrap();
		self.top_ring.logic_step(info);
		self.active = match self.bottom_ring.logic_step(info) {
			ring::State::Idle => true,
			_ => false,
		};
		if self.active && !self.queued_zoneables.is_empty() {
			self.queued_zoneables
				.drain(..)
				.for_each(|zoneable| self.zone.capture(&zoneable).unwrap());
		}
	}
}
impl ZoneHandler for Cell {
	fn enter(&mut self, uid: String, spatial: Spatial) {
		println!("Entered {}", uid);
		if self.active {
			self.zone.capture(&spatial).unwrap();
		} else {
			self.queued_zoneables.push(spatial.alias());
		}
		self.zoneables.insert(uid, spatial);
	}
	fn capture(&mut self, uid: String) {
		println!("Captured {}", uid);
		self.zoneables
			.get(&uid)
			.unwrap()
			.set_spatial_parent_in_place(&self.root)
			.unwrap();
	}
	fn release(&mut self, uid: String) {
		println!("Released {}", uid);
	}
	fn leave(&mut self, uid: String) {
		println!("Left {}", uid);
		self.zoneables.remove(&uid);
	}
}
