use crate::error::Result;
use crate::{components, controls, game_state, map, utils};

use allegro::*;
use allegro_font::*;
use allegro_sys::*;
use nalgebra::{Matrix4, Point2, Vector2, Vector3};

#[derive(Clone, Debug, PartialEq)]
pub enum Action
{
	SelectMe,
	MainMenu,
	SelectLevel(String),
	SelectCharacter(game_state::PlayerClass),
	Quit,
	Back,
	Forward(fn(&mut game_state::GameState, f32, f32) -> SubScreen),
	ToggleFullscreen,
	ChangeInput(controls::Action, usize),
	MouseSensitivity(f32),
	MusicVolume(f32),
	SfxVolume(f32),
}

impl Action
{
	pub fn is_select_me(&self) -> bool
	{
		match self
		{
			Action::SelectMe => true,
			_ => false,
		}
	}

	pub fn is_back(&self) -> bool
	{
		match self
		{
			Action::Back => true,
			_ => false,
		}
	}
}

#[derive(Clone)]
struct Button
{
	loc: Point2<f32>,
	size: Vector2<f32>,
	text: String,
	action: Action,
	selected: bool,
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
			selected: false,
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

	fn draw(&self, state: &game_state::GameState)
	{
		let c_ui = if self.selected
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
					if !self.selected
					{
						state.sfx.play_sound("data/ui1.ogg").unwrap();
					}
					return Some(Action::SelectMe);
				}
			}
			Event::KeyDown { keycode, .. } => match keycode
			{
				KeyCode::Enter | KeyCode::Space =>
				{
					if self.selected
					{
						state.sfx.play_sound("data/ui1.ogg").unwrap();
						return Some(self.action.clone());
					}
				}
				KeyCode::Escape =>
				{
					if self.action.is_back()
					{
						state.sfx.play_sound("data/ui2.ogg").unwrap();
						return Some(self.action.clone());
					}
				}
				_ => (),
			},
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
struct Toggle
{
	loc: Point2<f32>,
	size: Vector2<f32>,
	texts: Vec<String>,
	cur_value: usize,
	action_fn: fn(usize) -> Action,
	selected: bool,
}

impl Toggle
{
	fn new(
		x: f32, y: f32, w: f32, h: f32, cur_value: usize, texts: Vec<String>,
		action_fn: fn(usize) -> Action,
	) -> Self
	{
		Self {
			loc: Point2::new(x, y),
			size: Vector2::new(w, h),
			texts: texts,
			cur_value: cur_value,
			action_fn: action_fn,
			selected: false,
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

	fn draw(&self, state: &game_state::GameState)
	{
		let c_ui = if self.selected
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
			&self.texts[self.cur_value],
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
					if !self.selected
					{
						state.sfx.play_sound("data/ui1.ogg").unwrap();
					}
					return Some(Action::SelectMe);
				}
			}
			Event::KeyDown { keycode, .. } => match keycode
			{
				KeyCode::Enter | KeyCode::Space =>
				{
					if self.selected
					{
						return Some(self.trigger(state));
					}
				}
				_ => (),
			},
			Event::MouseButtonUp { x, y, .. } =>
			{
				let (x, y) = state.transform_mouse(*x as f32, *y as f32);
				if x > start.x && x < end.x && y > start.y && y < end.y
				{
					return Some(self.trigger(state));
				}
			}
			_ => (),
		}
		None
	}

	fn trigger(&mut self, state: &mut game_state::GameState) -> Action
	{
		state.sfx.play_sound("data/ui2.ogg").unwrap();
		self.cur_value = (self.cur_value + 1) % self.texts.len();
		(self.action_fn)(self.cur_value)
	}
}

#[derive(Clone)]
struct Slider
{
	loc: Point2<f32>,
	size: Vector2<f32>,
	cur_pos: f32,
	max_pos: f32,
	grabbed: bool,
	selected: bool,
	action_fn: fn(f32) -> Action,
}

impl Slider
{
	fn new(
		x: f32, y: f32, w: f32, h: f32, cur_pos: f32, max_pos: f32, action_fn: fn(f32) -> Action,
	) -> Self
	{
		Self {
			loc: Point2::new(x, y),
			size: Vector2::new(w, h),
			cur_pos: cur_pos,
			max_pos: max_pos,
			grabbed: false,
			selected: false,
			action_fn: action_fn,
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

	fn draw(&self, state: &game_state::GameState)
	{
		let c_ui = if self.selected
		{
			Color::from_rgb_f(1., 1., 1.)
		}
		else
		{
			Color::from_rgb_f(0.8, 0.8, 0.5)
		};

		let w = self.width();
		let cursor_x = self.loc.x - w / 2. + w * self.cur_pos / self.max_pos;
		let start_x = self.loc.x - w / 2.;
		let end_x = self.loc.x + w / 2.;
		if cursor_x - start_x > 16.
		{
			state
				.prim
				.draw_line(start_x, self.loc.y, cursor_x - 16., self.loc.y, c_ui, 4.);
		}
		if end_x - cursor_x > 16.
		{
			state
				.prim
				.draw_line(cursor_x + 16., self.loc.y, end_x, self.loc.y, c_ui, 4.);
		}
		//state.prim.draw_filled_circle(self.loc.x - w / 2. + w * self.cur_pos / self.max_pos, self.loc.y, 8., c_ui);
		state.core.draw_text(
			&state.ui_font,
			c_ui,
			cursor_x,
			self.loc.y - state.ui_font.get_line_height() as f32 / 2.,
			FontAlign::Centre,
			&format!("{:.1}", self.cur_pos),
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
					if self.grabbed
					{
						self.cur_pos = (x - start.x) / self.width() * self.max_pos;
						return Some((self.action_fn)(self.cur_pos));
					}
					else
					{
						if !self.selected
						{
							state.sfx.play_sound("data/ui1.ogg").unwrap();
						}
						return Some(Action::SelectMe);
					}
				}
			}
			Event::MouseButtonUp { .. } =>
			{
				self.grabbed = false;
			}
			Event::MouseButtonDown { x, y, .. } =>
			{
				let (x, y) = state.transform_mouse(*x as f32, *y as f32);
				if x > start.x && x < end.x && y > start.y && y < end.y
				{
					state.sfx.play_sound("data/ui2.ogg").unwrap();
					self.grabbed = true;
					self.cur_pos = (x - start.x) / self.width() * self.max_pos;
					return Some((self.action_fn)(self.cur_pos));
				}
			}
			Event::KeyDown { keycode, .. } =>
			{
				if self.selected
				{
					match keycode
					{
						KeyCode::Left =>
						{
							if self.cur_pos > 0.
							{
								state.sfx.play_sound("data/ui2.ogg").unwrap();
								self.cur_pos = utils::max(0., self.cur_pos - self.max_pos / 25.);
								return Some((self.action_fn)(self.cur_pos));
							}
						}
						KeyCode::Right =>
						{
							if self.cur_pos < self.max_pos
							{
								state.sfx.play_sound("data/ui2.ogg").unwrap();
								self.cur_pos =
									utils::min(self.max_pos, self.cur_pos + self.max_pos / 25.);
								return Some((self.action_fn)(self.cur_pos));
							}
						}
						_ => (),
					}
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

	fn draw(&self, state: &game_state::GameState)
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
	Slider(Slider),
	Toggle(Toggle),
}

impl Widget
{
	fn height(&self) -> f32
	{
		match self
		{
			Widget::Button(w) => w.height(),
			Widget::Label(w) => w.height(),
			Widget::Slider(w) => w.height(),
			Widget::Toggle(w) => w.height(),
		}
	}

	fn width(&self) -> f32
	{
		match self
		{
			Widget::Button(w) => w.width(),
			Widget::Label(w) => w.width(),
			Widget::Slider(w) => w.width(),
			Widget::Toggle(w) => w.width(),
		}
	}

	fn loc(&self) -> Point2<f32>
	{
		match self
		{
			Widget::Button(w) => w.loc,
			Widget::Label(w) => w.loc,
			Widget::Slider(w) => w.loc,
			Widget::Toggle(w) => w.loc,
		}
	}

	fn selectable(&self) -> bool
	{
		match self
		{
			Widget::Button(_) => true,
			Widget::Label(_) => false,
			Widget::Slider(_) => true,
			Widget::Toggle(_) => true,
		}
	}

	fn set_loc(&mut self, loc: Point2<f32>)
	{
		match self
		{
			Widget::Button(ref mut w) => w.loc = loc,
			Widget::Label(ref mut w) => w.loc = loc,
			Widget::Slider(ref mut w) => w.loc = loc,
			Widget::Toggle(ref mut w) => w.loc = loc,
		}
	}

	fn selected(&self) -> bool
	{
		match self
		{
			Widget::Button(w) => w.selected,
			Widget::Label(_) => false,
			Widget::Slider(w) => w.selected,
			Widget::Toggle(w) => w.selected,
		}
	}

	fn set_selected(&mut self, selected: bool)
	{
		match self
		{
			Widget::Button(ref mut w) => w.selected = selected,
			Widget::Label(_) => (),
			Widget::Slider(ref mut w) => w.selected = selected,
			Widget::Toggle(ref mut w) => w.selected = selected,
		}
	}

	fn draw(&self, state: &game_state::GameState)
	{
		match self
		{
			Widget::Button(w) => w.draw(state),
			Widget::Label(w) => w.draw(state),
			Widget::Slider(w) => w.draw(state),
			Widget::Toggle(w) => w.draw(state),
		}
	}

	fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		match self
		{
			Widget::Button(w) => w.input(state, event),
			Widget::Label(w) => w.input(state, event),
			Widget::Slider(w) => w.input(state, event),
			Widget::Toggle(w) => w.input(state, event),
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

		if let Some((i, j)) = cur_selection
		{
			new_widgets[i][j].set_selected(true);
		}

		Self {
			widgets: new_widgets,
			cur_selection: cur_selection.expect("No selectable widgets?"),
		}
	}

	pub fn draw(&self, state: &game_state::GameState)
	{
		for row in &self.widgets
		{
			for w in row
			{
				w.draw(state);
			}
		}
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		let mut action = None;
		let old_selection = self.cur_selection;
		'got_action: for (i, row) in self.widgets.iter_mut().enumerate()
		{
			for (j, w) in row.iter_mut().enumerate()
			{
				let cur_action = w.input(state, event);
				if cur_action.is_some()
				{
					action = cur_action;
					self.cur_selection = (i, j);
					break 'got_action;
				}
			}
		}
		if action.is_none() || action.as_ref().map(|a| a.is_select_me()) == Some(true)
		{
			match event
			{
				Event::KeyDown { keycode, .. } => match *keycode
				{
					KeyCode::Up =>
					{
						state.sfx.play_sound("data/ui1.ogg").unwrap();
						'found1: loop
						{
							self.cur_selection.0 = (self.cur_selection.0 + self.widgets.len() - 1)
								% self.widgets.len();
							let row_len = self.widgets[self.cur_selection.0].len();
							if self.cur_selection.1 >= row_len
							{
								self.cur_selection.1 = row_len - 1;
							}
							for _ in 0..row_len
							{
								if self.widgets[self.cur_selection.0][self.cur_selection.1]
									.selectable()
								{
									break 'found1;
								}
								self.cur_selection.1 =
									(self.cur_selection.1 + row_len - 1) % row_len;
							}
						}
					}
					KeyCode::Down =>
					{
						state.sfx.play_sound("data/ui1.ogg").unwrap();
						'found2: loop
						{
							self.cur_selection.0 = (self.cur_selection.0 + self.widgets.len() + 1)
								% self.widgets.len();
							let row_len = self.widgets[self.cur_selection.0].len();
							if self.cur_selection.1 >= row_len
							{
								self.cur_selection.1 = row_len - 1;
							}
							for _ in 0..row_len
							{
								if self.widgets[self.cur_selection.0][self.cur_selection.1]
									.selectable()
								{
									break 'found2;
								}
								self.cur_selection.1 =
									(self.cur_selection.1 + row_len - 1) % row_len;
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
					_ => (),
				},
				_ => (),
			}
		}
		self.widgets[old_selection.0][old_selection.1].set_selected(false);
		self.widgets[self.cur_selection.0][self.cur_selection.1].set_selected(true);
		action
	}
}

pub struct MainMenu
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
						Action::Forward(|s, dx, dy| {
							SubScreen::LevelMenu(LevelMenu::new(s, dx, dy))
						}),
					))],
					&[Widget::Button(Button::new(
						0.,
						0.,
						w,
						h,
						"CONTROLS",
						Action::Forward(|s, dx, dy| {
							SubScreen::ControlsMenu(ControlsMenu::new(s, dx, dy))
						}),
					))],
					&[Widget::Button(Button::new(
						0.,
						0.,
						w,
						h,
						"OPTIONS",
						Action::Forward(|s, dx, dy| {
							SubScreen::OptionsMenu(OptionsMenu::new(s, dx, dy))
						}),
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

pub struct LevelMenu
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
			if state.options.unlocked.contains(&level.name)
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
			Action::Back,
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

pub struct ControlsMenu
{
	widgets: WidgetList,
	accepting_input: bool,
}

impl ControlsMenu
{
	pub fn new(state: &game_state::GameState, display_width: f32, display_height: f32) -> Self
	{
		let w = 256.;
		let h = 16.;
		let cx = display_width / 2.;
		let cy = display_height / 2.;

		let mut widgets = vec![];
		widgets.push(vec![
			Widget::Label(Label::new(0., 0., w, h, "MOUSE SENSITIVITY")),
			Widget::Slider(Slider::new(
				0.,
				0.,
				w,
				h,
				state.controls.get_mouse_sensitivity(),
				2.,
				|i| Action::MouseSensitivity(i),
			)),
		]);

		for (&action, &inputs) in state.controls.get_actions_to_inputs()
		{
			let mut row = vec![Widget::Label(Label::new(
				0.,
				0.,
				w,
				h,
				&action.to_str().to_uppercase(),
			))];
			for i in 0..2
			{
				let input = inputs[i];
				let input_str = input
					.map(|i| i.to_str().to_uppercase())
					.unwrap_or("NONE".into());
				row.push(Widget::Button(Button::new(
					0.,
					0.,
					w,
					h,
					&input_str,
					Action::ChangeInput(action, i),
				)));
			}
			widgets.push(row);
		}
		widgets.push(vec![Widget::Button(Button::new(
			0.,
			0.,
			w,
			h,
			"BACK",
			Action::Back,
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
		let mut options_changed = false;
		if self.accepting_input
		{
			match &mut self.widgets.widgets[self.widgets.cur_selection.0]
				[self.widgets.cur_selection.1]
			{
				Widget::Button(b) =>
				{
					if let Action::ChangeInput(action, index) = b.action
					{
						if let Some(changed) = state.controls.change_action(action, index, event)
						{
							options_changed = changed;
							state.sfx.play_sound("data/ui2.ogg").unwrap();
							self.accepting_input = false;
						}
					}
				}
				_ => (),
			}
		}
		else
		{
			if let allegro::Event::KeyDown {
				keycode: allegro::KeyCode::Delete,
				..
			} = event
			{
				match &mut self.widgets.widgets[self.widgets.cur_selection.0]
					[self.widgets.cur_selection.1]
				{
					Widget::Button(b) =>
					{
						if let Action::ChangeInput(action, index) = b.action
						{
							state.controls.clear_action(action, index);
							options_changed = true;
							state.sfx.play_sound("data/ui2.ogg").unwrap();
						}
					}
					_ => (),
				}
			}
			action = self.widgets.input(state, event);
			match action
			{
				Some(Action::ChangeInput(_, _)) =>
				{
					self.accepting_input = true;
					match &mut self.widgets.widgets[self.widgets.cur_selection.0]
						[self.widgets.cur_selection.1]
					{
						Widget::Button(b) => b.text = "PRESS INPUT".into(),
						_ => (),
					}
				}
				Some(Action::MouseSensitivity(ms)) =>
				{
					state.controls.set_mouse_sensitivity(ms);
					options_changed = true;
				}
				_ => (),
			}
		}
		if options_changed
		{
			for widget_row in &mut self.widgets.widgets
			{
				for widget in widget_row
				{
					match widget
					{
						Widget::Button(b) =>
						{
							if let Action::ChangeInput(action, index) = b.action
							{
								b.text = state.controls.get_inputs(action).unwrap()[index]
									.map(|a| a.to_str().to_uppercase())
									.unwrap_or("NONE".into());
							}
						}
						_ => (),
					}
				}
			}
			state.options.controls = state.controls.get_controls().clone();
			game_state::save_options(&state.core, &state.options).unwrap();
		}
		action
	}
}

pub struct OptionsMenu
{
	widgets: WidgetList,
}

impl OptionsMenu
{
	pub fn new(state: &game_state::GameState, display_width: f32, display_height: f32) -> Self
	{
		let w = 256.;
		let h = 16.;
		let cx = display_width / 2.;
		let cy = display_height / 2.;

		let widgets = [
			vec![
				Widget::Label(Label::new(0., 0., w, h, "FULLSCREEN")),
				Widget::Toggle(Toggle::new(
					0.,
					0.,
					w,
					h,
					state.options.fullscreen as usize,
					vec!["NO".into(), "YES".into()],
					|_| Action::ToggleFullscreen,
				)),
			],
			vec![
				Widget::Label(Label::new(0., 0., w, h, "MUSIC VOLUME")),
				Widget::Slider(Slider::new(
					0.,
					0.,
					w,
					h,
					state.options.music_volume,
					4.,
					|i| Action::MusicVolume(i),
				)),
			],
			vec![
				Widget::Label(Label::new(0., 0., w, h, "SFX VOLUME")),
				Widget::Slider(Slider::new(
					0.,
					0.,
					w,
					h,
					state.options.sfx_volume,
					4.,
					|i| Action::SfxVolume(i),
				)),
			],
			vec![Widget::Button(Button::new(
				0.,
				0.,
				w,
				h,
				"BACK",
				Action::Back,
			))],
		];

		Self {
			widgets: WidgetList::new(
				cx,
				cy,
				h,
				h,
				&widgets.iter().map(|r| &r[..]).collect::<Vec<_>>(),
			),
		}
	}

	pub fn draw(&self, state: &game_state::GameState)
	{
		self.widgets.draw(state);
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		let mut options_changed = false;
		let action = self.widgets.input(state, event);
		if let Some(action) = action
		{
			match action
			{
				Action::ToggleFullscreen =>
				{
					state.options.fullscreen = !state.options.fullscreen;
					options_changed = true;
				}
				Action::MusicVolume(v) =>
				{
					state.options.music_volume = v;
					state.sfx.set_music_volume(v);
					options_changed = true;
				}
				Action::SfxVolume(v) =>
				{
					state.options.sfx_volume = v;
					state.sfx.set_sfx_volume(v);
					options_changed = true;
				}
				_ => return Some(action),
			}
		}
		if options_changed
		{
			game_state::save_options(&state.core, &state.options).unwrap();
		}
		None
	}
}

pub struct CharacterMenu
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
						Action::Back,
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

pub struct InGameMenu
{
	widgets: WidgetList,
}

impl InGameMenu
{
	pub fn new(display_width: f32, display_height: f32) -> Self
	{
		let w = 128.;
		let h = 32.;
		let cx = display_width / 2.;
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
						"RESUME",
						Action::Back,
					))],
					&[Widget::Button(Button::new(
						0.,
						0.,
						w,
						h,
						"CONTROLS",
						Action::Forward(|s, dx, dy| {
							SubScreen::ControlsMenu(ControlsMenu::new(s, dx, dy))
						}),
					))],
					&[Widget::Button(Button::new(
						0.,
						0.,
						w,
						h,
						"OPTIONS",
						Action::Forward(|s, dx, dy| {
							SubScreen::OptionsMenu(OptionsMenu::new(s, dx, dy))
						}),
					))],
					&[Widget::Button(Button::new(
						0.,
						0.,
						w,
						h,
						"QUIT",
						Action::MainMenu,
					))],
				],
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

pub enum SubScreen
{
	MainMenu(MainMenu),
	LevelMenu(LevelMenu),
	ControlsMenu(ControlsMenu),
	CharacterMenu(CharacterMenu),
	OptionsMenu(OptionsMenu),
	InGameMenu(InGameMenu),
}

impl SubScreen
{
	pub fn draw(&self, state: &game_state::GameState)
	{
		match self
		{
			SubScreen::MainMenu(s) => s.draw(state),
			SubScreen::LevelMenu(s) => s.draw(state),
			SubScreen::ControlsMenu(s) => s.draw(state),
			SubScreen::CharacterMenu(s) => s.draw(state),
			SubScreen::OptionsMenu(s) => s.draw(state),
			SubScreen::InGameMenu(s) => s.draw(state),
		}
	}

	pub fn input(&mut self, state: &mut game_state::GameState, event: &Event) -> Option<Action>
	{
		match self
		{
			SubScreen::MainMenu(s) => s.input(state, event),
			SubScreen::LevelMenu(s) => s.input(state, event),
			SubScreen::ControlsMenu(s) => s.input(state, event),
			SubScreen::CharacterMenu(s) => s.input(state, event),
			SubScreen::OptionsMenu(s) => s.input(state, event),
			SubScreen::InGameMenu(s) => s.input(state, event),
		}
	}
}
