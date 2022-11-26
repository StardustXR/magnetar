use crate::ring::{self, Ring};
use glam::Quat;
use mint::Vector3;
use rustc_hash::FxHashSet;
use stardust_xr_molecules::fusion::{
	client::LogicStepInfo,
	fields::CylinderField,
	spatial::{Spatial, Zone, ZoneHandler},
	HandlerWrapper, WeakNodeRef,
};
use std::{f32::consts::PI, sync::Arc};

pub struct Cell {
	_root: Arc<Spatial>,
	_field: Arc<CylinderField>,
	zone: WeakNodeRef<Zone>,
	pub active: bool,
	queued_zoneables: FxHashSet<String>,
	top_ring: Ring,
	bottom_ring: Ring,
}
impl Cell {
	pub fn new(parent: &Spatial, height: f32) -> HandlerWrapper<Zone, Self> {
		let root = Arc::new(
			Spatial::builder()
				.spatial_parent(parent)
				.position(Vector3::from([0.0, height, 0.0]))
				.zoneable(false)
				.build()
				.unwrap(),
		);

		let field = Arc::new(
			CylinderField::builder()
				.spatial_parent(&root)
				.rotation(Quat::from_rotation_x(PI * 0.5))
				.length(1.0)
				.radius(1.0)
				.build()
				.unwrap(),
		);

		let top_ring = Ring::new_from_point(&root, 0.5, 1.0);
		let bottom_ring = Ring::new_from_point(&root, -0.5, 1.0);

		let root_2 = root.clone();
		let field_2 = field.clone();
		Zone::create(&root_2, None, None, &*field_2, move |zone, _| Cell {
			_root: root,
			_field: field,
			zone,
			active: false,
			queued_zoneables: FxHashSet::default(),
			top_ring,
			bottom_ring,
		})
		.unwrap()
	}

	pub fn logic_step(&mut self, info: LogicStepInfo) {
		self.top_ring.logic_step(info);
		self.active = match self.bottom_ring.logic_step(info) {
			ring::State::Idle => true,
			_ => false,
		};
		if self.active && !self.queued_zoneables.is_empty() {
			self.zone.with_node(|zone| {
				self.queued_zoneables
					.drain()
					.for_each(|zoneable| zone.capture(&zoneable).unwrap());
			});
		}
	}
}
impl ZoneHandler for Cell {
	fn enter(&mut self, zone: &Zone, uid: &str, _spatial: &Spatial) {
		dbg!(self.active);
		dbg!(uid);
		if self.active {
			zone.capture(uid).unwrap();
		} else {
			self.queued_zoneables.insert(uid.to_string());
		}
	}
	fn capture(&mut self, _zone: &Zone, _uid: &str, _spatial: &Spatial) {}
	fn release(&mut self, _zone: &Zone, _uid: &str) {}
	fn leave(&mut self, _zone: &Zone, uid: &str) {
		self.queued_zoneables.remove(uid);
	}
}
