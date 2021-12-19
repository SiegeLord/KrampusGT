#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(backtrace)]

mod components;
mod error;
mod game_state;
mod map;
//~ mod menu;
mod sfx;
mod spatial_grid;
//~ mod speech;
mod sprite;
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Options
{
	fullscreen: bool,
	width: i32,
	height: i32,
	play_music: bool,
	vsync_method: i32,
}

fn real_main() -> Result<()>
{
	let options: Options = load_config("options.cfg")?;

	let mut state = GameState::new()?;
	if options.play_music
	{
		state.sfx.play_music()?;
	}

	if options.fullscreen
	{
		state.core.set_new_display_flags(FULLSCREEN_WINDOW);
	}

	state.core.set_new_display_option(
		DisplayOption::DepthSize,
		16,
		DisplayOptionImportance::Suggest,
	);
	if options.vsync_method == 1
	{
		state.core.set_new_display_option(
			DisplayOption::Vsync,
			1,
			DisplayOptionImportance::Suggest,
		);
	}
	let display = Display::new(&state.core, options.width, options.height)
		.map_err(|_| "Couldn't create display".to_string())?;

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

	//~ let mut menu = menu::Menu::new(
	//~ &mut state,
	//~ display.get_width() as f32,
	//~ display.get_height() as f32,
	//~ )?;
	let mut map: Option<map::Map> = Some(map::Map::new(
		&mut state,
		display.get_width() as f32,
		display.get_height() as f32,
	)?);
	let mut logics_without_draw = 0;
	let mut old_mouse_hide = false;
	//~ let mut prev_frame_start = state.core.get_time();

	timer.start();
	while !quit
	{
		if draw && queue.is_empty()
		{
			//~ let frame_start = state.core.get_time();
			state.core.set_target_bitmap(Some(display.get_backbuffer()));
			state.core.clear_to_color(Color::from_rgb_f(0., 0.2, 0.));
			state.core.clear_depth_buffer(1.);
			unsafe {
				gl::Enable(gl::CULL_FACE);
				gl::CullFace(gl::BACK);
			}

			if options.vsync_method == 2
			{
				state.core.wait_for_vsync().ok();
			}

			if let Some(map) = &mut map
			{
				map.draw(&state)?;
			}
			//~ else
			//~ {
			//~ menu.draw(&state)?;
			//~ }

			state.core.flip_display();

			//~ if state.tick % 20 == 0
			//~ {
			//~ println!("FPS: {}", 1. / (frame_start - prev_frame_start));
			//~ }
			//~ prev_frame_start = frame_start;
			logics_without_draw = 0;
		}

		let event = queue.wait_for_event();
		//~ let next_screen;
		if let Some(map) = &mut map
		{
			map.input(&event, &mut state)?;
		}

		//~ else
		//~ {
		//~ next_screen = menu.input(&event, &mut state)?;
		//~ }
		//~ if let Some(next_screen) = next_screen
		//~ {
		//~ match next_screen
		//~ {
		//~ NextScreen::Game =>
		//~ {
		//~ map = Some(map::Map::new(
		//~ &mut state,
		//~ rng.gen_range(0..16000),
		//~ display.get_width() as f32,
		//~ display.get_height() as f32,
		//~ )?);
		//~ }
		//~ NextScreen::Menu =>
		//~ {
		//~ map = None;
		//~ }
		//~ NextScreen::Quit =>
		//~ {
		//~ quit = true;
		//~ }
		//~ }
		//~ }
		match event
		{
			Event::DisplayClose { .. } => quit = true,
			Event::TimerTick { .. } =>
			{
				if logics_without_draw > 10
				{
					continue;
				}

				if let Some(map) = &mut map
				{
					map.logic(&mut state)?;
					//~ println!();
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
	}

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
