use crate::error::Result;
use crate::utils;
use nalgebra::{Point2, Vector2};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use allegro::*;
use allegro_acodec::*;
use allegro_audio::*;

use rand::prelude::*;

pub struct Sfx
{
	audio: AudioAddon,
	acodec: AcodecAddon,
	sink: Sink,
	stream: Option<AudioStream>,
	music_file: String,
	sample_instances: Vec<SampleInstance>,
	exclusive_sounds: Vec<String>,
	exclusive_instance: Option<SampleInstance>,
	sfx_volume: f32,
	music_volume: f32,

	samples: HashMap<String, Sample>,
}

impl Sfx
{
	pub fn new(sfx_volume: f32, music_volume: f32, core: &Core) -> Result<Sfx>
	{
		let audio = AudioAddon::init(&core)?;
		let acodec = AcodecAddon::init(&audio)?;
		let sink = Sink::new(&audio).map_err(|_| "Couldn't create audio sink".to_string())?;

		Ok(Sfx {
			sfx_volume: sfx_volume,
			music_volume: music_volume,
			audio: audio,
			acodec: acodec,
			sink: sink,
			sample_instances: vec![],
			stream: None,
			exclusive_instance: None,
			exclusive_sounds: vec![],
			samples: HashMap::new(),
			music_file: "".into(),
		})
	}

	pub fn set_music_file(&mut self, music: &str)
	{
		self.music_file = music.to_string();
	}

	pub fn cache_sample<'l>(&'l mut self, name: &str) -> Result<&'l Sample>
	{
		Ok(match self.samples.entry(name.to_string())
		{
			Entry::Occupied(o) => o.into_mut(),
			Entry::Vacant(v) => v.insert(utils::load_sample(&self.audio, name)?),
		})
	}

	pub fn get_sample<'l>(&'l self, name: &str) -> Option<&'l Sample>
	{
		self.samples.get(name)
	}

	pub fn update_sounds(&mut self) -> Result<()>
	{
		self.sample_instances.retain(|s| s.get_playing().unwrap());
		if let Some(ref stream) = self.stream
		{
			if !stream.get_playing()
			{
				self.play_music()?
			}
		}

		if !self.exclusive_sounds.is_empty()
		{
			let mut play_next_sound = true;
			if let Some(exclusive_instance) = &self.exclusive_instance
			{
				play_next_sound = !exclusive_instance.get_playing().unwrap();
			}
			if play_next_sound
			{
				let name = self.exclusive_sounds.pop().unwrap();
				self.cache_sample(&name)?;
				let sample = self.samples.get(&name).unwrap();
				let instance = self
					.sink
					.play_sample(
						sample,
						1.,
						None,
						thread_rng().gen_range(0.9..1.1),
						Playmode::Once,
					)
					.map_err(|_| "Couldn't play sound".to_string())?;
				self.exclusive_instance = Some(instance);
			}
		}

		Ok(())
	}

	pub fn play_sound(&mut self, name: &str) -> Result<()>
	{
		self.cache_sample(name)?;
		let sample = self.samples.get(name).unwrap();
		let instance = self
			.sink
			.play_sample(
				sample,
				0.25 * self.sfx_volume,
				None,
				thread_rng().gen_range(0.9..1.1),
				Playmode::Once,
			)
			.map_err(|_| "Couldn't play sound".to_string())?;
		self.sample_instances.push(instance);
		Ok(())
	}

	pub fn play_positional_sound(
		&mut self, name: &str, sound_pos: Point2<f32>, camera_pos: Point2<f32>, dir: f32,
		volume: f32,
	) -> Result<()>
	{
		self.cache_sample(name)?;

		if self.sample_instances.len() < 50
		{
			let sample = self.samples.get(name).unwrap();

			let dist = (sound_pos - camera_pos).norm();
			let volume = utils::clamp(self.sfx_volume * volume * 40000. / (dist * dist), 0., 1.);
			let diff = sound_pos - camera_pos;
			let diff = diff / (diff.norm() + 1e-3);

			let dir_vec = utils::dir_vec3(dir).xz();
			let left = Vector2::new(-dir_vec.y, dir_vec.x);
			let pan = utils::clamp(left.dot(&diff), -1., 1.);

			let instance = self
				.sink
				.play_sample(
					sample,
					volume,
					Some(pan),
					thread_rng().gen_range(0.9..1.1),
					Playmode::Once,
				)
				.map_err(|_| "Couldn't play sound".to_string())?;
			self.sample_instances.push(instance);
		}
		Ok(())
	}

	pub fn play_exclusive_sound(&mut self, name: &str) -> Result<()>
	{
		self.exclusive_sounds.insert(0, name.to_string());
		Ok(())
	}

	pub fn play_music(&mut self) -> Result<()>
	{
		let mut new_stream = AudioStream::load(&self.audio, &self.music_file)
			.map_err(|_| format!("Couldn't load {}", self.music_file))?;
		new_stream.attach(&mut self.sink).unwrap();
		//~ new_stream.set_playmode(Playmode::Loop).unwrap();
		new_stream.set_gain(self.music_volume).unwrap();
		self.stream = Some(new_stream);
		Ok(())
	}

	pub fn set_music_volume(&mut self, new_volume: f32)
	{
		self.music_volume = new_volume;
		if let Some(stream) = self.stream.as_mut()
		{
			stream.set_gain(new_volume).unwrap();
		}
	}

	pub fn set_sfx_volume(&mut self, new_volume: f32)
	{
		self.sfx_volume = new_volume;
	}
}
