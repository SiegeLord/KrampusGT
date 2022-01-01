use crate::error::Result;
use crate::{components, game_state, map, utils};

use allegro::*;
use allegro_font::*;
use allegro_sys::*;
use nalgebra::{Matrix4, Point2, Vector2, Vector3};

struct Button
{
	start: Point2<f32>,
	end: Point2<f32>,
	text: String,
	action: Action,
}

#[derive(Clone, PartialEq, Eq)]
enum Action
{
	SelectMe,
	MainMenu,
	LevelMenu,
	SelectLevel(String),
	SelectCharacter(game_state::PlayerClass),
	Quit,
}

impl Button
{
	fn new(x: f32, y: f32, w: f32, h: f32, text: &str, action: Action) -> Self
	{
		Self {
			start: Point2::new(x - w / 2., y - h / 2.),
			end: Point2::new(x + w / 2., y + h / 2.),
			text: text.into(),
			action: action,
		}
	}

	fn draw(&self, selected: bool, state: &game_state::GameState)
	{
		let c_ui = if selected
		{
			Color::from_rgb_f(1., 1., 1.)
		}
		else
		{
			Color::from_rgb_f(0.8, 0.8, 0.5)
		};

		let center = self.start + (self.end - self.start) / 2.;

		state.core.draw_text(
			&state.ui_font,
			c_ui,
			center.x,
			center.y - 16.,
			FontAlign::Centre,
			&self.text,
		);
	}

	fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		match event
		{
			Event::MouseAxes { x, y, .. } =>
			{
				let (x, y) = state.transform_mouse(*x as f32, *y as f32);
				if x > self.start.x && x < self.end.x && y > self.start.y && y < self.end.y
				{
					return Some(Action::SelectMe);
				}
			}
			Event::MouseButtonUp { x, y, .. } =>
			{
				let (x, y) = state.transform_mouse(*x as f32, *y as f32);
				if x > self.start.x && x < self.end.x && y > self.start.y && y < self.end.y
				{
					state.sfx.play_sound("data/ui2.ogg").unwrap();
					return Some(self.action.clone());
				}
			}
			_ => (),
		}
		None
	}
}

struct MainMenu
{
	buttons: Vec<Button>,
	cur_selection: usize,
}

impl MainMenu
{
	pub fn new(_display_width: f32, display_height: f32) -> Self
	{
		let w = 128.;
		let h = 32.;
		//~ let cx = display_width / 2.;
		let cy = display_height / 2.;
		Self {
			buttons: vec![
				Button::new(128., cy - h / 2., w, h, "NEW GAME", Action::LevelMenu),
				Button::new(128., cy + h / 2., w, h, "QUIT", Action::Quit),
			],
			cur_selection: 0,
		}
	}

	pub fn draw(&self, state: &game_state::GameState)
	{
		state.core.draw_bitmap(
			state.get_bitmap("data/main_menu.png").unwrap(),
			0.,
			0.,
			Flag::zero(),
		);
		for (i, button) in self.buttons.iter().enumerate()
		{
			button.draw(i == self.cur_selection, state);
		}
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		for (i, button) in self.buttons.iter_mut().enumerate()
		{
			let action = button.input(state, event);
			if action.is_some()
			{
				if action == Some(Action::SelectMe)
				{
					if self.cur_selection != i
					{
						state.sfx.play_sound("data/ui1.ogg").unwrap();
					}
					self.cur_selection = i;
				}
				return action;
			}
		}
		match event
		{
			Event::KeyUp { keycode, .. } => match *keycode
			{
				KeyCode::Up =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					self.cur_selection =
						(self.cur_selection + self.buttons.len() - 1) % self.buttons.len();
				}
				KeyCode::Down =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					self.cur_selection =
						(self.cur_selection + self.buttons.len() + 1) % self.buttons.len();
				}
				KeyCode::Enter | KeyCode::Space =>
				{
					state.sfx.play_sound("data/ui2.ogg").unwrap();
					return Some(self.buttons[self.cur_selection].action.clone());
				}
				KeyCode::Escape =>
				{
					return Some(Action::Quit);
				}
				_ => (),
			},
			_ => (),
		}
		None
	}
}

struct LevelMenu
{
	buttons: Vec<Button>,
	cur_selection: usize,
}

impl LevelMenu
{
	pub fn new(state: &game_state::GameState, display_width: f32, display_height: f32) -> Self
	{
		let w = 128.;
		let h = 32.;
		let cx = display_width / 2.;
		let cy = display_height / 2.;

		let mut buttons = vec![];

		let mut num_unlocked = 0;
		for level in &state.levels.levels
		{
			if level.unlocked
			{
				num_unlocked += 1;
			}
		}

		let offt = num_unlocked as f32 * h / 2.;
		let mut i = 0;
		for level in &state.levels.levels
		{
			if level.unlocked
			{
				buttons.push(Button::new(
					cx,
					cy - offt + (i as f32 * h),
					w,
					h,
					&level.name,
					Action::SelectLevel(level.filename.clone()),
				));
				i += 1;
			}
		}

		buttons.push(Button::new(
			cx,
			cy - offt + ((1 + state.levels.levels.len()) as f32 * h),
			w,
			h,
			"BACK",
			Action::MainMenu,
		));

		Self {
			buttons: buttons,
			cur_selection: 0,
		}
	}

	pub fn draw(&self, state: &game_state::GameState)
	{
		for (i, button) in self.buttons.iter().enumerate()
		{
			button.draw(i == self.cur_selection, state);
		}
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		for (i, button) in self.buttons.iter_mut().enumerate()
		{
			let action = button.input(state, event);
			if action.is_some()
			{
				if action == Some(Action::SelectMe)
				{
					if self.cur_selection != i
					{
						state.sfx.play_sound("data/ui1.ogg").unwrap();
					}
					self.cur_selection = i;
				}
				return action;
			}
		}
		match event
		{
			Event::KeyUp { keycode, .. } => match *keycode
			{
				KeyCode::Up =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					self.cur_selection =
						(self.cur_selection + self.buttons.len() - 1) % self.buttons.len();
				}
				KeyCode::Down =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					self.cur_selection =
						(self.cur_selection + self.buttons.len() + 1) % self.buttons.len();
				}
				KeyCode::Enter | KeyCode::Space =>
				{
					state.sfx.play_sound("data/ui2.ogg").unwrap();
					return Some(self.buttons[self.cur_selection].action.clone());
				}
				KeyCode::Escape =>
				{
					return Some(Action::MainMenu);
				}
				_ => (),
			},
			_ => (),
		}
		None
	}
}

struct CharacterMenu
{
	buttons: Vec<Button>,
	cur_selection: usize,
	display_width: f32,
	display_height: f32,
}

impl CharacterMenu
{
	pub fn new(_state: &game_state::GameState, display_width: f32, display_height: f32) -> Self
	{
		let w = 128.;
		let h = 32.;
		let cx = display_width / 2.;
		let cy = display_height / 2.;

		let mut buttons = vec![];

		buttons.push(Button::new(
			cx - 256.,
			cy + 128.,
			w,
			h,
			"SANTA",
			Action::SelectCharacter(game_state::PlayerClass::Santa),
		));

		buttons.push(Button::new(
			cx + 256.,
			cy + 128.,
			w,
			h,
			"CYBER RUDE-OLF",
			Action::SelectCharacter(game_state::PlayerClass::Reindeer),
		));

		buttons.push(Button::new(cx, cy + 256., w, h, "BACK", Action::LevelMenu));

		Self {
			buttons: buttons,
			cur_selection: 0,
			display_width: display_width,
			display_height: display_height,
		}
	}

	pub fn draw(&self, state: &game_state::GameState)
	{
		for (i, button) in self.buttons.iter().enumerate()
		{
			button.draw(i == self.cur_selection, state);
		}

		let cx = self.display_width / 2.;
		let cy = self.display_height / 2.;

		let sheet = state.get_sprite_sheet("data/santa.cfg").unwrap();
		if let Some(atlas_bmp) = sheet.get_bitmap(
			state.time(),
			2. * state.time() as f32,
			0.,
			None,
			Some(components::Velocity {
				vel: Vector3::new(64., 0., 0.),
				dir_vel: 0.,
			}),
		)
		{
			let w = atlas_bmp.width();
			let h = atlas_bmp.height();
			state.core.draw_bitmap_region(
				&state.atlas.pages[atlas_bmp.page].bitmap,
				atlas_bmp.start.x,
				atlas_bmp.start.y,
				w,
				h,
				cx - 256. - w / 2.,
				cy - 64.,
				Flag::zero(),
			);
		}

		let sheet = state.get_sprite_sheet("data/reindeer.cfg").unwrap();
		if let Some(atlas_bmp) = sheet.get_bitmap(
			state.time(),
			2. * state.time() as f32,
			0.,
			None,
			Some(components::Velocity {
				vel: Vector3::new(64., 0., 0.),
				dir_vel: 0.,
			}),
		)
		{
			let w = atlas_bmp.width();
			let h = atlas_bmp.height();
			state.core.draw_bitmap_region(
				&state.atlas.pages[atlas_bmp.page].bitmap,
				atlas_bmp.start.x,
				atlas_bmp.start.y,
				w,
				h,
				cx + 256. - w / 2.,
				cy - 64.,
				Flag::zero(),
			);
		}
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		for (i, button) in self.buttons.iter_mut().enumerate()
		{
			let action = button.input(state, event);
			if action.is_some()
			{
				if action == Some(Action::SelectMe)
				{
					if self.cur_selection != i
					{
						state.sfx.play_sound("data/ui1.ogg").unwrap();
					}
					self.cur_selection = i;
				}
				return action;
			}
		}
		match event
		{
			Event::KeyUp { keycode, .. } => match *keycode
			{
				KeyCode::Left =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					self.cur_selection =
						(self.cur_selection + self.buttons.len() - 1) % self.buttons.len();
				}
				KeyCode::Right =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					self.cur_selection =
						(self.cur_selection + self.buttons.len() + 1) % self.buttons.len();
				}
				KeyCode::Enter | KeyCode::Space =>
				{
					state.sfx.play_sound("data/ui2.ogg").unwrap();
					return Some(self.buttons[self.cur_selection].action.clone());
				}
				KeyCode::Escape =>
				{
					return Some(Action::LevelMenu);
				}
				_ => (),
			},
			_ => (),
		}
		None
	}
}

enum CurSubScreen
{
	MainMenu(MainMenu),
	LevelMenu(LevelMenu),
	CharacterMenu(CharacterMenu),
}

pub struct Menu
{
	display_width: f32,
	display_height: f32,
	next_level: String,

	subscreen: CurSubScreen,
}

impl Menu
{
	pub fn new(
		state: &mut game_state::GameState, display_width: f32, display_height: f32,
	) -> Result<Self>
	{
		if state.options.play_music
		{
			state.sfx.set_music_file("data/evil_minded.mod");
			state.sfx.play_music()?;
		}

		state.cache_bitmap("data/main_menu.png")?;
		state.cache_sprite_sheet("data/santa.cfg")?;
		state.cache_sprite_sheet("data/reindeer.cfg")?;
		state.sfx.cache_sample("data/ui1.ogg")?;
		state.sfx.cache_sample("data/ui2.ogg")?;

		Ok(Self {
			display_width: display_width,
			display_height: display_height,
			subscreen: CurSubScreen::MainMenu(MainMenu::new(display_width, display_height)),
			next_level: "".into(),
		})
	}

	pub fn input(
		&mut self, event: &Event, state: &mut game_state::GameState,
	) -> Result<Option<game_state::NextScreen>>
	{
		let action = match &mut self.subscreen
		{
			CurSubScreen::MainMenu(menu) => menu.input(state, event),
			CurSubScreen::LevelMenu(menu) => menu.input(state, event),
			CurSubScreen::CharacterMenu(menu) => menu.input(state, event),
		};
		if let Some(action) = action
		{
			match action
			{
				Action::MainMenu =>
				{
					self.subscreen = CurSubScreen::MainMenu(MainMenu::new(
						self.display_width,
						self.display_height,
					));
				}
				Action::LevelMenu =>
				{
					self.subscreen = CurSubScreen::LevelMenu(LevelMenu::new(
						state,
						self.display_width,
						self.display_height,
					));
				}
				Action::Quit => return Ok(Some(game_state::NextScreen::Quit)),
				Action::SelectLevel(name) =>
				{
					self.next_level = name.clone();
					self.subscreen = CurSubScreen::CharacterMenu(CharacterMenu::new(
						state,
						self.display_width,
						self.display_height,
					));
				}
				Action::SelectCharacter(character) =>
				{
					return Ok(Some(game_state::NextScreen::Game(
						self.next_level.clone(),
						character,
						None,
						None,
						3,
					)));
				}
				_ => (),
			}
		}
		Ok(None)
	}

	pub fn draw(&mut self, state: &game_state::GameState) -> Result<()>
	{
		let ortho_mat = Matrix4::new_orthographic(
			0.,
			self.display_width as f32,
			self.display_height as f32,
			0.,
			-1.,
			1.,
		);

		unsafe {
			gl::Disable(gl::CULL_FACE);
		}
		state
			.core
			.use_projection_transform(&utils::mat4_to_transform(ortho_mat));
		state.core.use_transform(&Transform::identity());
		state.core.set_depth_test(None);
		unsafe {
			al_set_render_state(ALLEGRO_ALPHA_TEST_RS, 0);
		}
		state.core.clear_to_color(Color::from_rgb_f(0., 0., 0.));
		match &self.subscreen
		{
			CurSubScreen::MainMenu(menu) => menu.draw(state),
			CurSubScreen::LevelMenu(menu) => menu.draw(state),
			CurSubScreen::CharacterMenu(menu) => menu.draw(state),
		};
		Ok(())
	}
}
