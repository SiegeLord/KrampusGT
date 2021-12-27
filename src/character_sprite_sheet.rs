use crate::error::Result;
use crate::{atlas, components, utils};

use allegro::*;
use nalgebra::{RealField, Vector2, Vector3};
use serde_derive::{Deserialize, Serialize};

fn default_speed() -> f32
{
	100.
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct OrientationDesc
{
	#[serde(default = "Vec::new")]
	idle: Vec<String>,
	#[serde(default = "Vec::new")]
	walk: Vec<String>,
	#[serde(default = "Vec::new")]
	fire: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct CharacterSpriteSheetDesc
{
	#[serde(default = "default_speed")]
	speed: f32,
	orientations: Vec<OrientationDesc>,
}

pub struct Orientation
{
	pub idle: Vec<atlas::AtlasBitmap>,
	pub walk: Vec<atlas::AtlasBitmap>,
	pub fire: Vec<atlas::AtlasBitmap>,
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

			let mut fire = Vec::with_capacity(orientation_desc.fire.len());
			for bitmap_name in &orientation_desc.fire
			{
				fire.push(atlas.insert(core, bitmap_name)?);
			}

			orientations.push(Orientation {
				idle: idle,
				walk: walk,
				fire: fire,
			});
		}

		Ok(Self {
			desc: desc,
			orientations: orientations,
		})
	}

	pub fn get_bitmap(
		&self, time: f64, dir: f32, camera_dir: f32, last_fire_time: Option<f64>,
		vel: Option<components::Velocity>,
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
		let orientation = &self.orientations
			[((rel_dir + f32::pi() + window_size / 2.) / window_size) as usize % num_orientations];

		let mut speed = self.desc.speed * 100.;
		let mut reverse = false;
		let mut animation = &orientation.idle;

		if let Some(vel) = vel
		{
			let walk = &orientation.walk;
			if !walk.is_empty()
			{
				let f = 0.2;
				let eff_vel;
				if vel.vel.norm() > 0.
				{
					eff_vel = vel.vel.xz().dot(&dir).signum() * vel.vel.norm();
					speed = self.desc.speed * f * eff_vel.abs();
					reverse = eff_vel < 0.;
					animation = walk;
				}
				else if vel.dir_vel.abs() > 0.
				{
					eff_vel = 50. * vel.dir_vel;
					speed = self.desc.speed * f * eff_vel.abs();
					reverse = eff_vel < 0.;
					animation = walk;
				}
			}
		}
		if let Some(last_fire_time) = last_fire_time
		{
			if (time - last_fire_time) < 0.25 && !orientation.fire.is_empty()
			{
				animation = &orientation.fire;
			}
		}

		let mut frame = ((time * speed as f64) as usize) % animation.len();
		if reverse
		{
			frame = animation.len() - 1 - frame;
		}
		Some(&animation[frame])
	}
}
