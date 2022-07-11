use crate::error::Result;
use crate::{components, controls, game_state, map, utils};

use allegro::*;
use allegro_font::*;
use allegro_sys::*;
use nalgebra::{Matrix4, Point2, Vector2, Vector3};

#[derive(Clone, PartialEq, Eq)]
enum Action
{
	SelectMe,
	MainMenu,
	ControlsMenu,
	LevelMenu,
	SelectLevel(String),
	SelectCharacter(game_state::PlayerClass),
	Quit,
	ChangeInput(controls::Action),
}

#[derive(Clone)]
struct Button
{
	loc: Point2<f32>,
	size: Vector2<f32>,
	text: String,
	action: Action,
}

impl Button
{
	fn new(x: f32, y: f32, w: f32, h: f32, text: &str, action: Action) -> Self
	{
		Self {
			loc: Point2::new(x, y),
			size: Vector2::new(w, h),
			text: text.into(),
			action: action,
		}
	}

	fn width(&self) -> f32
	{
		self.size.x
	}

	fn height(&self) -> f32
	{
		self.size.y
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

		state.core.draw_text(
			&state.ui_font,
			c_ui,
			self.loc.x,
			self.loc.y - state.ui_font.get_line_height() as f32 / 2.,
			FontAlign::Centre,
			&self.text,
		);
	}

	fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		let start = self.loc - self.size / 2.;
		let end = self.loc + self.size / 2.;
		match event
		{
			Event::MouseAxes { x, y, .. } =>
			{
				let (x, y) = state.transform_mouse(*x as f32, *y as f32);
				if x > start.x && x < end.x && y > start.y && y < end.y
				{
					return Some(Action::SelectMe);
				}
			}
			Event::MouseButtonUp { x, y, .. } =>
			{
				let (x, y) = state.transform_mouse(*x as f32, *y as f32);
				if x > start.x && x < end.x && y > start.y && y < end.y
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

#[derive(Clone)]
struct Label
{
	loc: Point2<f32>,
	size: Vector2<f32>,
	text: String,
}

impl Label
{
	fn new(x: f32, y: f32, w: f32, h: f32, text: &str) -> Self
	{
		Self {
			loc: Point2::new(x, y),
			size: Vector2::new(w, h),
			text: text.into(),
		}
	}

	fn width(&self) -> f32
	{
		self.size.x
	}

	fn height(&self) -> f32
	{
		self.size.y
	}

	fn draw(&self, _selected: bool, state: &game_state::GameState)
	{
		state.core.draw_text(
			&state.ui_font,
			Color::from_rgb_f(0.6, 0.6, 0.4),
			self.loc.x,
			self.loc.y - state.ui_font.get_line_height() as f32 / 2.,
			FontAlign::Centre,
			&self.text,
		);
	}

	fn input(&mut self, _state: &mut game_state::GameState, _event: &Event) -> Option<Action>
	{
		None
	}
}

#[derive(Clone)]
enum Widget
{
	Button(Button),
	Label(Label),
}

impl Widget
{
	fn height(&self) -> f32
	{
		match self
		{
			Widget::Button(w) => w.height(),
			Widget::Label(w) => w.height(),
		}
	}

	fn width(&self) -> f32
	{
		match self
		{
			Widget::Button(w) => w.width(),
			Widget::Label(w) => w.width(),
		}
	}

	fn loc(&self) -> Point2<f32>
	{
		match self
		{
			Widget::Button(w) => w.loc,
			Widget::Label(w) => w.loc,
		}
	}

	fn action(&self) -> Option<&Action>
	{
		match self
		{
			Widget::Button(w) => Some(&w.action),
			Widget::Label(_) => None,
		}
	}

	fn selectable(&self) -> bool
	{
		match self
		{
			Widget::Button(_) => true,
			Widget::Label(_) => false,
		}
	}

	fn set_loc(&mut self, loc: Point2<f32>)
	{
		match self
		{
			Widget::Button(ref mut w) => w.loc = loc,
			Widget::Label(ref mut w) => w.loc = loc,
		}
	}

	fn draw(&self, selected: bool, state: &game_state::GameState)
	{
		match self
		{
			Widget::Button(w) => w.draw(selected, state),
			Widget::Label(w) => w.draw(selected, state),
		}
	}

	fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		match self
		{
			Widget::Button(w) => w.input(state, event),
			Widget::Label(w) => w.input(state, event),
		}
	}
}

struct WidgetList
{
	widgets: Vec<Vec<Widget>>,
	cur_selection: (usize, usize),
}

impl WidgetList
{
	fn new(cx: f32, cy: f32, w_space: f32, h_space: f32, widgets: &[&[Widget]]) -> Self
	{
		let mut y = 0.;
		let mut new_widgets = Vec::with_capacity(widgets.len());
		let mut cur_selection = None;
		for (i, row) in widgets.iter().enumerate()
		{
			let mut new_row = Vec::with_capacity(row.len());
			let mut max_height = -f32::INFINITY;
			let mut x = 0.;

			// Place the relative x's, collect max height.
			for (j, w) in row.iter().enumerate()
			{
				if w.selectable() && cur_selection.is_none()
				{
					cur_selection = Some((i, j));
				}
				if j > 0
				{
					x += (w_space + w.width()) / 2.;
				}
				let mut new_w = w.clone();
				let mut loc = w.loc();
				loc.x = x;
				new_w.set_loc(loc);
				new_row.push(new_w);
				max_height = utils::max(max_height, w.height());
				if j + 1 < row.len()
				{
					x += (w_space + w.width()) / 2.;
				}
			}

			if i > 0
			{
				y += (h_space + max_height) / 2.;
			}

			// Place the relative y's, shift the x's.
			for w in new_row.iter_mut()
			{
				let mut loc = w.loc();
				loc.y = y;
				loc.x += cx - x / 2.;
				w.set_loc(loc);
			}

			if i + 1 < widgets.len()
			{
				y += (h_space + max_height) / 2.;
			}
			new_widgets.push(new_row);
		}

		// Shift the y's
		for row in new_widgets.iter_mut()
		{
			for w in row.iter_mut()
			{
				let mut loc = w.loc();
				loc.y += cy - y / 2.;
				w.set_loc(loc);
			}
		}

		Self {
			widgets: new_widgets,
			cur_selection: cur_selection.expect("No selectable widgets?"),
		}
	}

	pub fn draw(&self, state: &game_state::GameState)
	{
		for (i, row) in self.widgets.iter().enumerate()
		{
			for (j, w) in row.iter().enumerate()
			{
				w.draw((i, j) == self.cur_selection, state);
			}
		}
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		for (i, row) in self.widgets.iter_mut().enumerate()
		{
			for (j, w) in row.iter_mut().enumerate()
			{
				let action = w.input(state, event);
				if action.is_some()
				{
					if action == Some(Action::SelectMe)
					{
						if self.cur_selection != (i, j)
						{
							state.sfx.play_sound("data/ui1.ogg").unwrap();
						}
						self.cur_selection = (i, j);
					}
					return action;
				}
			}
		}
		match event
		{
			Event::KeyUp { keycode, .. } => match *keycode
			{
				KeyCode::Up =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					'found1: loop
					{
						self.cur_selection.0 =
							(self.cur_selection.0 + self.widgets.len() - 1) % self.widgets.len();
						let row_len = self.widgets[self.cur_selection.0].len();
						if self.cur_selection.1 >= row_len
						{
							self.cur_selection.1 = row_len - 1;
						}
						for _ in 0..row_len
						{
							if self.widgets[self.cur_selection.0][self.cur_selection.1].selectable()
							{
								break 'found1;
							}
							self.cur_selection.1 = (self.cur_selection.1 + row_len - 1) % row_len;
						}
					}
				}
				KeyCode::Down =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					'found2: loop
					{
						self.cur_selection.0 =
							(self.cur_selection.0 + self.widgets.len() + 1) % self.widgets.len();
						let row_len = self.widgets[self.cur_selection.0].len();
						if self.cur_selection.1 >= row_len
						{
							self.cur_selection.1 = row_len - 1;
						}
						for _ in 0..row_len
						{
							if self.widgets[self.cur_selection.0][self.cur_selection.1].selectable()
							{
								break 'found2;
							}
							self.cur_selection.1 = (self.cur_selection.1 + row_len - 1) % row_len;
						}
					}
				}
				KeyCode::Left =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					let row_len = self.widgets[self.cur_selection.0].len();
					loop
					{
						self.cur_selection.1 = (self.cur_selection.1 + row_len - 1) % row_len;
						if self.widgets[self.cur_selection.0][self.cur_selection.1].selectable()
						{
							break;
						}
					}
				}
				KeyCode::Right =>
				{
					state.sfx.play_sound("data/ui1.ogg").unwrap();
					let row_len = self.widgets[self.cur_selection.0].len();
					loop
					{
						self.cur_selection.1 = (self.cur_selection.1 + row_len + 1) % row_len;
						if self.widgets[self.cur_selection.0][self.cur_selection.1].selectable()
						{
							break;
						}
					}
				}
				KeyCode::Enter | KeyCode::Space =>
				{
					state.sfx.play_sound("data/ui2.ogg").unwrap();
					return self.widgets[self.cur_selection.0][self.cur_selection.1]
						.action()
						.map(|a| a.to_owned());
				}
				_ => (),
			},
			_ => (),
		}
		None
	}
}

struct MainMenu
{
	widgets: WidgetList,
}

impl MainMenu
{
	pub fn new(_display_width: f32, display_height: f32) -> Self
	{
		let w = 128.;
		let h = 32.;
		let cx = 128.;
		let cy = display_height / 2.;

		Self {
			widgets: WidgetList::new(
				cx,
				cy,
				h,
				h,
				&[
					&[Widget::Button(Button::new(
						0.,
						0.,
						w,
						h,
						"NEW GAME",
						Action::LevelMenu,
					))],
					&[Widget::Button(Button::new(
						0.,
						0.,
						w,
						h,
						"CONTROLS",
						Action::ControlsMenu,
					))],
					&[Widget::Button(Button::new(
						0.,
						0.,
						w,
						h,
						"QUIT",
						Action::Quit,
					))],
				],
			),
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
		self.widgets.draw(state);
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		self.widgets.input(state, event)
	}
}

struct LevelMenu
{
	widgets: WidgetList,
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
		for level in &state.levels.levels
		{
			if level.unlocked
			{
				buttons.push([Widget::Button(Button::new(
					0.,
					0.,
					w,
					h,
					&level.name,
					Action::SelectLevel(level.filename.clone()),
				))]);
			}
		}

		buttons.push([Widget::Button(Button::new(
			0.,
			0.,
			w,
			h,
			"BACK",
			Action::MainMenu,
		))]);

		Self {
			widgets: WidgetList::new(
				cx,
				cy,
				h,
				h,
				&buttons.iter().map(|r| &r[..]).collect::<Vec<_>>(),
			),
		}
	}

	pub fn draw(&self, state: &game_state::GameState)
	{
		self.widgets.draw(state);
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		self.widgets.input(state, event)
	}
}

struct ControlsMenu
{
	widgets: WidgetList,
	accepting_input: bool,
}

impl ControlsMenu
{
	pub fn new(display_width: f32, display_height: f32, state: &game_state::GameState) -> Self
	{
		let w = 128.;
		let h = 16.;
		let cx = display_width / 2.;
		let cy = display_height / 2.;

		let mut widgets = vec![];

		let actions = [
			controls::Action::TurnLeft,
			controls::Action::TurnRight,
			controls::Action::MoveForward,
			controls::Action::StrafeLeft,
			controls::Action::MoveBackward,
			controls::Action::StrafeRight,
			controls::Action::FireWeapon,
			controls::Action::SelectWeapon1,
			controls::Action::SelectWeapon2,
			controls::Action::SelectWeapon3,
			controls::Action::EnterVehicle,
		];

		for action in &actions
		{
			let keycode = state.options.controls.controls.get_by_left(action).unwrap();
			widgets.push(vec![
				Widget::Label(Label::new(0., 0., w, h, &action.to_str().to_uppercase())),
				Widget::Button(Button::new(
					0.,
					0.,
					w,
					h,
					&keycode.to_str().to_uppercase(),
					Action::ChangeInput(*action),
				)),
			])
		}
		widgets.push(vec![Widget::Button(Button::new(
			0.,
			0.,
			w,
			h,
			"BACK",
			Action::MainMenu,
		))]);

		Self {
			widgets: WidgetList::new(
				cx,
				cy,
				h,
				h,
				&widgets.iter().map(|r| &r[..]).collect::<Vec<_>>(),
			),
			accepting_input: false,
		}
	}

	pub fn draw(&self, state: &game_state::GameState)
	{
		self.widgets.draw(state);
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		let mut action = None;
		if self.accepting_input
		{
			match event
			{
				Event::KeyUp { keycode, .. } =>
				{
					self.accepting_input = false;
					state.sfx.play_sound("data/ui2.ogg").unwrap();
					if *keycode != KeyCode::Escape
					{
						match &mut self.widgets.widgets[self.widgets.cur_selection.0]
							[self.widgets.cur_selection.1]
						{
							Widget::Button(b) =>
							{
								let new_keycode = controls::KeyCode(*keycode);
								if let Action::ChangeInput(action) = b.action
								{
									if state.options.controls.controls.contains_right(&new_keycode)
									{
										let old_keycode = *state
											.options
											.controls
											.controls
											.get_by_left(&action)
											.unwrap();
										let other_action = *state
											.options
											.controls
											.controls
											.get_by_right(&new_keycode)
											.unwrap();
										state
											.options
											.controls
											.controls
											.insert(other_action, old_keycode);
									}
									state.options.controls.controls.insert(action, new_keycode);
								}
							}
							_ => (),
						}
					}
					utils::save_config("options.cfg", &state.options).unwrap();
					for row in &mut self.widgets.widgets
					{
						for w in row
						{
							match w
							{
								Widget::Button(b) => match b.action
								{
									Action::ChangeInput(action) =>
									{
										b.text = state
											.options
											.controls
											.controls
											.get_by_left(&action)
											.unwrap()
											.to_str()
											.to_uppercase();
									}
									_ => (),
								},
								_ => (),
							}
						}
					}
				}
				_ => (),
			}
		}
		else
		{
			action = self.widgets.input(state, event);
			if let Some(Action::ChangeInput(_)) = action
			{
				self.accepting_input = true;
				match &mut self.widgets.widgets[self.widgets.cur_selection.0]
					[self.widgets.cur_selection.1]
				{
					Widget::Button(b) => b.text = "PRESS KEY".into(),
					_ => (),
				}
			}
		}
		action
	}
}

struct CharacterMenu
{
	widgets: WidgetList,
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

		buttons.push(Widget::Button(Button::new(
			0.,
			0.,
			w,
			h,
			"SANTA",
			Action::SelectCharacter(game_state::PlayerClass::Santa),
		)));

		buttons.push(Widget::Button(Button::new(
			0.,
			0.,
			w,
			h,
			"CYBER RUDE-OLF",
			Action::SelectCharacter(game_state::PlayerClass::Reindeer),
		)));

		Self {
			widgets: WidgetList::new(
				cx,
				cy + 128.,
				512. - w,
				h,
				&[
					&buttons,
					&[Widget::Button(Button::new(
						cx,
						cy + 64.,
						w,
						h,
						"BACK",
						Action::LevelMenu,
					))],
				],
			),
			display_width: display_width,
			display_height: display_height,
		}
	}

	pub fn draw(&self, state: &game_state::GameState)
	{
		self.widgets.draw(state);

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
		self.widgets.input(state, event)
	}
}

enum CurSubScreen
{
	MainMenu(MainMenu),
	LevelMenu(LevelMenu),
	ControlsMenu(ControlsMenu),
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
			CurSubScreen::ControlsMenu(menu) => menu.input(state, event),
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
				Action::ControlsMenu =>
				{
					self.subscreen = CurSubScreen::ControlsMenu(ControlsMenu::new(
						self.display_width,
						self.display_height,
						state,
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
			CurSubScreen::ControlsMenu(menu) => menu.draw(state),
			CurSubScreen::CharacterMenu(menu) => menu.draw(state),
		};
		Ok(())
	}
}
