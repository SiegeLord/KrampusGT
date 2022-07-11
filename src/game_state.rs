use crate::error::Result;
use crate::sfx::Sfx;
use crate::utils::{load_bitmap, Vec2D, DT};
use crate::{atlas, character_sprite_sheet, components, controls, utils};
use allegro::*;
use allegro_font::*;
use allegro_image::*;
use allegro_primitives::*;
use allegro_ttf::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Options
{
	pub fullscreen: bool,
	pub width: i32,
	pub height: i32,
	pub play_music: bool,
	pub vsync_method: i32,
	pub sfx_volume: f32,
	pub music_volume: f32,
	pub turn_sensitivity: f32,
	pub controls: controls::Controls,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct LevelEntry
{
	pub filename: String,
	pub name: String,
	pub unlocked: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Levels
{
	pub levels: Vec<LevelEntry>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlayerClass
{
	Santa,
	Reindeer,
}

pub enum NextScreen
{
	Game(
		String,
		PlayerClass,
		Option<components::Health>,
		Option<components::WeaponSet>,
		i32,
	),
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
	pub ui_font: Font,
	pub number_font: Font,
	pub options: Options,
	pub draw_scale: f32,
	pub display_width: f32,
	pub display_height: f32,
	pub levels: Levels,

	bitmaps: HashMap<String, Bitmap>,
	character_sheets: HashMap<String, character_sprite_sheet::CharacterSpriteSheet>,
}

impl GameState
{
	pub fn new() -> Result<GameState>
	{
		let options: Options = utils::load_config("options.cfg")?;
		let core = Core::init()?;
		let prim = PrimitivesAddon::init(&core)?;
		let image = ImageAddon::init(&core)?;
		let font = FontAddon::init(&core)?;
		let ttf = TtfAddon::init(&font)?;
		core.install_keyboard()
			.map_err(|_| "Couldn't install keyboard".to_string())?;
		core.install_mouse()
			.map_err(|_| "Couldn't install mouse".to_string())?;

		let sfx = Sfx::new(options.sfx_volume, options.music_volume, &core)?;

		let ui_font = ttf
			.load_ttf_font("data/GAMEPLAY-1987.ttf", 16, TtfFlags::zero())
			.map_err(|_| "Couldn't load 'data/GAMEPLAY-1987.ttf'".to_string())?;
		let number_font = ttf
			.load_ttf_font("data/Open 24 Display St.ttf", -48, TtfFlags::zero())
			.map_err(|_| "Couldn't load 'data/Open 24 Display St.ttf'".to_string())?;

		let levels: Levels = utils::load_config("data/levels.cfg")?;

		Ok(GameState {
			options: options,
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
			hide_mouse: false,
			atlas: atlas::Atlas::new(4096),
			ui_font: ui_font,
			number_font: number_font,
			draw_scale: 1.,
			display_width: 0.,
			display_height: 0.,
			levels: levels,
		})
	}

	pub fn transform_mouse(&self, x: f32, y: f32) -> (f32, f32)
	{
		let bw = 800.;
		let bh = 600.;

		let x = (x - self.display_width / 2.) / self.draw_scale + bw / 2.;
		let y = (y - self.display_height / 2.) / self.draw_scale + bh / 2.;
		(x, y)
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
