use crate::error::Result;
use crate::sfx::Sfx;
use crate::utils::{load_bitmap, Vec2D, DT};
use crate::{atlas, character_sprite_sheet};
use allegro::*;
use allegro_font::*;
use allegro_image::*;
use allegro_primitives::*;
use allegro_ttf::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

pub enum NextScreen
{
	Game,
	Menu,
	Quit,
}

pub struct GameState
{
	pub core: Core,
	pub prim: PrimitivesAddon,
	pub image: ImageAddon,
	pub font: FontAddon,
	pub ttf: TtfAddon,
	pub tick: i64,
	pub paused: bool,
	pub sfx: Sfx,
	pub hide_mouse: bool,
	pub atlas: atlas::Atlas,

	bitmaps: HashMap<String, Bitmap>,
	character_sheets: HashMap<String, character_sprite_sheet::CharacterSpriteSheet>,
}

impl GameState
{
	pub fn new() -> Result<GameState>
	{
		let core = Core::init()?;
		let prim = PrimitivesAddon::init(&core)?;
		let image = ImageAddon::init(&core)?;
		let font = FontAddon::init(&core)?;
		let ttf = TtfAddon::init(&font)?;
		core.install_keyboard()
			.map_err(|_| "Couldn't install keyboard".to_string())?;
		core.install_mouse()
			.map_err(|_| "Couldn't install mouse".to_string())?;

		let sfx = Sfx::new(&core)?;

		Ok(GameState {
			core: core,
			prim: prim,
			image: image,
			tick: 0,
			bitmaps: HashMap::new(),
			character_sheets: HashMap::new(),
			font: font,
			ttf: ttf,
			sfx: sfx,
			paused: false,
			hide_mouse: true,
			atlas: atlas::Atlas::new(4096),
		})
	}

	pub fn cache_bitmap<'l>(&'l mut self, name: &str) -> Result<&'l Bitmap>
	{
		Ok(match self.bitmaps.entry(name.to_string())
		{
			Entry::Occupied(o) => o.into_mut(),
			Entry::Vacant(v) => v.insert(load_bitmap(&self.core, name)?),
		})
	}

	pub fn cache_sprite_sheet<'l>(
		&'l mut self, name: &str,
	) -> Result<&'l character_sprite_sheet::CharacterSpriteSheet>
	{
		Ok(match self.character_sheets.entry(name.to_string())
		{
			Entry::Occupied(o) => o.into_mut(),
			Entry::Vacant(v) => v.insert(character_sprite_sheet::CharacterSpriteSheet::new(
				&self.core,
				name,
				&mut self.atlas,
			)?),
		})
	}

	pub fn get_bitmap<'l>(&'l self, name: &str) -> Option<&'l Bitmap>
	{
		self.bitmaps.get(name)
	}

	pub fn get_sprite_sheet<'l>(
		&'l self, name: &str,
	) -> Option<&'l character_sprite_sheet::CharacterSpriteSheet>
	{
		self.character_sheets.get(name)
	}

	pub fn time(&self) -> f64
	{
		self.tick as f64 * DT as f64
	}
}
