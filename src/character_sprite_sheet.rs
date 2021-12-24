use crate::error::Result;
use crate::{atlas, utils};

use allegro::*;
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
}
