use crate::error::Result;
use crate::ui::{Action, CharacterMenu, ControlsMenu, LevelMenu, MainMenu, OptionsMenu, SubScreen};
use crate::{components, controls, game_state, map, utils};

use allegro::*;
use allegro_sys::*;
use nalgebra::Matrix4;

pub struct Menu
{
	display_width: f32,
	display_height: f32,
	next_level: String,

	subscreens: Vec<SubScreen>,
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
		state.hide_mouse = false;
		state.paused = false;

		state.cache_bitmap("data/main_menu.png")?;
		state.cache_sprite_sheet("data/santa.cfg")?;
		state.cache_sprite_sheet("data/reindeer.cfg")?;
		state.sfx.cache_sample("data/ui1.ogg")?;
		state.sfx.cache_sample("data/ui2.ogg")?;

		Ok(Self {
			display_width: display_width,
			display_height: display_height,
			subscreens: vec![SubScreen::MainMenu(MainMenu::new(
				display_width,
				display_height,
			))],
			next_level: "".into(),
		})
	}

	pub fn input(
		&mut self, event: &Event, state: &mut game_state::GameState,
	) -> Result<Option<game_state::NextScreen>>
	{
		if let Event::KeyDown {
			keycode: KeyCode::Escape,
			..
		} = event
		{
			if self.subscreens.len() > 1
			{
				state.sfx.play_sound("data/ui2.ogg").unwrap();
				self.subscreens.pop().unwrap();
				return Ok(None);
			}
		}
		if let Some(action) = self.subscreens.last_mut().unwrap().input(state, event)
		{
			match action
			{
				Action::Forward(subscreen_fn) =>
				{
					self.subscreens.push(subscreen_fn(state, self.display_width, self.display_height));
				}
				Action::Quit => return Ok(Some(game_state::NextScreen::Quit)),
				Action::SelectLevel(name) =>
				{
					self.next_level = name.clone();
					self.subscreens
						.push(SubScreen::CharacterMenu(CharacterMenu::new(
							state,
							self.display_width,
							self.display_height,
						)));
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
				Action::Back =>
				{
					self.subscreens.pop().unwrap();
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
		self.subscreens.last().unwrap().draw(state);
		Ok(())
	}
}
