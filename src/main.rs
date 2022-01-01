#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(backtrace)]

mod atlas;
mod character_sprite_sheet;
mod components;
mod error;
mod game_state;
mod map;
mod sfx;
mod spatial_grid;
mod ui;
mod utils;

use crate::error::Result;
use crate::game_state::{GameState, NextScreen};
use crate::utils::{load_config, DT};
use allegro::*;
use allegro_dialog::*;
use allegro_sys::*;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::rc::Rc;

enum CurScreen
{
	Game(map::Map),
	Menu(ui::Menu),
}

fn real_main() -> Result<()>
{
	let mut state = GameState::new()?;

	if state.options.fullscreen
	{
		state.core.set_new_display_flags(OPENGL | FULLSCREEN_WINDOW);
	}

	state.core.set_new_display_option(
		DisplayOption::DepthSize,
		16,
		DisplayOptionImportance::Suggest,
	);
	if state.options.vsync_method == 1
	{
		state.core.set_new_display_option(
			DisplayOption::Vsync,
			1,
			DisplayOptionImportance::Suggest,
		);
	}
	let display = Display::new(&state.core, state.options.width, state.options.height)
		.map_err(|_| "Couldn't create display".to_string())?;

	let buffer_width = 800;
	let buffer_height = 600;
	state.core.set_new_bitmap_depth(16);
	let buffer = Bitmap::new(&state.core, buffer_width, buffer_height).unwrap();
	state.core.set_new_bitmap_depth(0);

	state.display_width = display.get_width() as f32;
	state.display_height = display.get_height() as f32;
	state.draw_scale = utils::min(
		(display.get_width() as f32) / (buffer.get_width() as f32),
		(display.get_height() as f32) / (buffer.get_height() as f32),
	);

	gl_loader::init_gl();
	gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

	let timer =
		Timer::new(&state.core, DT as f64).map_err(|_| "Couldn't create timer".to_string())?;

	let queue =
		EventQueue::new(&state.core).map_err(|_| "Couldn't create event queue".to_string())?;
	queue.register_event_source(display.get_event_source());
	queue.register_event_source(
		state
			.core
			.get_keyboard_event_source()
			.expect("Couldn't get keyboard"),
	);
	queue.register_event_source(
		state
			.core
			.get_mouse_event_source()
			.expect("Couldn't get mouse"),
	);
	queue.register_event_source(timer.get_event_source());

	let mut quit = false;
	let mut draw = true;
	//~ let mut rng = thread_rng();

	let mut cur_screen = CurScreen::Menu(ui::Menu::new(
		&mut state,
		buffer_width as f32,
		buffer_height as f32,
	)?);

	let mut logics_without_draw = 0;
	let mut old_mouse_hide = state.hide_mouse;
	//~ let mut prev_frame_start = state.core.get_time();

	timer.start();
	while !quit
	{
		if draw && queue.is_empty()
		{
			//~ let frame_start = state.core.get_time();
			state.core.set_target_bitmap(Some(&buffer));

			match &mut cur_screen
			{
				CurScreen::Game(map) => map.draw(&state)?,
				CurScreen::Menu(menu) => menu.draw(&state)?,
			}

			if state.options.vsync_method == 2
			{
				state.core.wait_for_vsync().ok();
			}

			state.core.set_target_bitmap(Some(display.get_backbuffer()));

			state.core.clear_to_color(Color::from_rgb_f(0., 0., 0.));

			let bw = buffer.get_width() as f32;
			let bh = buffer.get_height() as f32;
			let dw = display.get_width() as f32;
			let dh = display.get_height() as f32;

			state.core.draw_scaled_bitmap(
				&buffer,
				0.,
				0.,
				bw,
				bh,
				dw / 2. - bw / 2. * state.draw_scale,
				dh / 2. - bh / 2. * state.draw_scale,
				bw * state.draw_scale,
				bh * state.draw_scale,
				Flag::zero(),
			);

			state.core.flip_display();

			//~ if state.tick % 20 == 0
			//~ {
			//~ println!("FPS: {}", 1. / (frame_start - prev_frame_start));
			//~ }
			//~ prev_frame_start = frame_start;
			logics_without_draw = 0;
		}

		let event = queue.wait_for_event();
		let mut next_screen = match &mut cur_screen
		{
			CurScreen::Game(map) => map.input(&event, &mut state)?,
			CurScreen::Menu(menu) => menu.input(&event, &mut state)?,
		};

		match event
		{
			Event::DisplayClose { .. } => quit = true,
			Event::DisplaySwitchOut { .. } =>
			{
				state.hide_mouse = false;
			}
			Event::DisplaySwitchIn { .. } =>
			{
				match cur_screen
				{
					CurScreen::Game(_) =>
					{
						state.hide_mouse = true;
					}
					_ => (),
				}
			}
			Event::TimerTick { .. } =>
			{
				if logics_without_draw > 10
				{
					continue;
				}

				if next_screen.is_none()
				{
					next_screen = match &mut cur_screen
					{
						CurScreen::Game(map) => map.logic(&mut state)?,
						_ => None,
					}
				}

				if state.hide_mouse
				{
					state
						.core
						.set_mouse_xy(&display, display.get_width() / 2, display.get_height() / 2)
						.map_err(|_| "Couldn't set mouse position".to_string())?;
				}

				if old_mouse_hide != state.hide_mouse
				{
					old_mouse_hide = state.hide_mouse;
					unsafe {
						if state.hide_mouse
						{
							al_hide_mouse_cursor(display.get_allegro_display());
						}
						else
						{
							al_show_mouse_cursor(display.get_allegro_display());
						}
					}
				}

				logics_without_draw += 1;
				state.sfx.update_sounds()?;

				if !state.paused
				{
					state.tick += 1;
				}
				draw = true;
			}
			_ => (),
		}

		if let Some(next_screen) = next_screen
		{
			match next_screen
			{
				NextScreen::Game(level, class, health, weapons, lives) =>
				{
					for other_level in &mut state.levels.levels
					{
						if level == other_level.filename
						{
							other_level.unlocked = true;
						}
					}
					cur_screen = CurScreen::Game(map::Map::new(
						&mut state,
						&level,
						class,
						health,
						weapons,
						lives,
						buffer_width as f32,
						buffer_height as f32,
					)?);
				}
				NextScreen::Menu =>
				{
					cur_screen = CurScreen::Menu(ui::Menu::new(
						&mut state,
						buffer_width as f32,
						buffer_height as f32,
					)?);
					state.hide_mouse = false;
				}
				NextScreen::Quit =>
				{
					quit = true;
				}
			}
		}
	}

	utils::save_config("data/levels.cfg", state.levels)?;

	Ok(())
}

fn main()
{
	use std::panic::catch_unwind;

	match catch_unwind(|| real_main().unwrap())
	{
		Err(e) =>
		{
			let err: String = e
				.downcast_ref::<&'static str>()
				.map(|&e| e.to_owned())
				.or_else(|| e.downcast_ref::<String>().map(|e| e.clone()))
				.unwrap_or("Unknown error!".to_owned());

			show_native_message_box(
				None,
				"Error!",
				"An error has occurred!",
				&err,
				Some("You make me sad."),
				MESSAGEBOX_ERROR,
			);
		}
		Ok(_) => (),
	}
}
