use crate::ring::{self, Ring};
use glam::Quat;
use stardust_xr_fusion::{
	fields::{CylinderShape, Field, Shape},
	node::NodeType,
	root::FrameInfo,
	spatial::{Spatial, SpatialAspect, SpatialRef, Transform, Zone, ZoneAspect, ZoneHandler},
	HandlerWrapper,
};
use std::f32::consts::PI;

pub struct Cell {
	root: Spatial,
	_field: Field,
	zone: Zone,
	pub active: bool,
	// zoneables: FxHashMap<u64, SpatialRef>,
	queued_zoneables: Vec<SpatialRef>,
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

		let field = Field::create(
			&root,
			Transform::from_rotation(Quat::from_rotation_x(PI * 0.5)),
			Shape::Cylinder(CylinderShape {
				length: 1.0,
				radius: 1.0,
			}),
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
			// zoneables: FxHashMap::default(),
			queued_zoneables: Vec::new(),
			top_ring,
			bottom_ring,
		};
		zone.wrap(cell).unwrap()
	}

	pub fn logic_step(&mut self, info: &FrameInfo) {
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
	fn enter(&mut self, spatial: SpatialRef) {
		println!("Entered {}", spatial.node().get_id().unwrap());
		if self.active {
			self.zone.capture(&spatial).unwrap();
		} else {
			self.queued_zoneables.push(spatial);
		}
		// self.zoneables.insert(uid, spatial);
	}
	fn capture(&mut self, spatial: Spatial) {
		println!("Captured {}", spatial.node().get_id().unwrap());
		spatial.set_spatial_parent_in_place(&self.root).unwrap();
	}
	fn release(&mut self, id: u64) {
		println!("Released {}", id);
	}
	fn leave(&mut self, id: u64) {
		println!("Left {}", id);
		// self.zoneables.remove(&id);
	}
}
