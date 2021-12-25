use crate::error::Result;
use crate::{atlas, components, utils};

use allegro::*;
use nalgebra::{RealField, Vector2, Vector3};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct OrientationDesc
{
	#[serde(default = "Vec::new")]
	idle: Vec<String>,
	#[serde(default = "Vec::new")]
	walk: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct CharacterSpriteSheetDesc
{
	orientations: Vec<OrientationDesc>,
}

pub struct Orientation
{
	pub idle: Vec<atlas::AtlasBitmap>,
	pub walk: Vec<atlas::AtlasBitmap>,
}

pub struct CharacterSpriteSheet
{
	desc: CharacterSpriteSheetDesc,
	pub orientations: Vec<Orientation>,
}

impl CharacterSpriteSheet
{
	pub fn new(core: &Core, filename: &str, atlas: &mut atlas::Atlas) -> Result<Self>
	{
		let desc: CharacterSpriteSheetDesc = utils::load_config(filename)?;

		let mut orientations = Vec::with_capacity(desc.orientations.len());
		for orientation_desc in &desc.orientations
		{
			let mut idle = Vec::with_capacity(orientation_desc.idle.len());
			for bitmap_name in &orientation_desc.idle
			{
				idle.push(atlas.insert(core, bitmap_name)?);
			}

			let mut walk = Vec::with_capacity(orientation_desc.walk.len());
			for bitmap_name in &orientation_desc.walk
			{
				walk.push(atlas.insert(core, bitmap_name)?);
			}

			orientations.push(Orientation {
				idle: idle,
				walk: walk,
			});
		}

		Ok(Self {
			desc: desc,
			orientations: orientations,
		})
	}

	pub fn get_bitmap(
		&self, time: f64, dir: f32, camera_dir: f32, vel: Option<components::Velocity>,
	) -> Option<&atlas::AtlasBitmap>
	{
		let num_orientations = self.orientations.len();
		if num_orientations == 0
		{
			return None;
		}

		let dir = utils::dir_vec3(dir).xz();
		let camera_front = utils::dir_vec3(camera_dir).xz();
		let camera_left = Vector2::new(-camera_front.y, camera_front.x);

		let x = dir.dot(&camera_left);
		let y = dir.dot(&camera_front);

		let rel_dir = x.atan2(y);
		let window_size = 2. * f32::pi() / num_orientations as f32;
		let orientation =
			((rel_dir + f32::pi() + window_size / 2.) / window_size) as usize % num_orientations;

		if let Some(vel) = vel
		{
			let walk = &self.orientations[orientation].walk;
			if !walk.is_empty()
			{
				let f = 0.2;
				let mut eff_vel = 0.;
				if vel.vel.norm() > 0.
				{
					eff_vel = vel.vel.xz().dot(&dir).signum() * vel.vel.norm();
				}
				else if vel.dir_vel.abs() > 0.
				{
					eff_vel = 5. * vel.dir_vel;
				}

				let frame = ((time * f * eff_vel.abs() as f64) as usize) % walk.len();
				if eff_vel > 0.
				{
					return Some(&self.orientations[orientation].walk[frame]);
				}
				else if eff_vel < 0.
				{
					return Some(&self.orientations[orientation].walk[walk.len() - 1 - frame]);
				}
			}
		}
		Some(&self.orientations[orientation].idle[0])
	}
}
