use crate::error::{Error, Result};
use allegro::*;
use allegro_audio::*;
use allegro_color::*;
use nalgebra;
use rand::prelude::*;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use slr_config::{from_element, to_element, ConfigElement, Source};
use std::{fs, path};

pub const DT: f32 = 1. / 120.;
pub const PI: f32 = std::f32::consts::PI;
pub type Vec2D = nalgebra::Vector2<f32>;
pub type Vec3D = nalgebra::Vector3<f32>;

use na::{
	Isometry3, Matrix4, Perspective3, Point3, Quaternion, RealField, Rotation3, Unit, Vector3,
	Vector4,
};
use nalgebra as na;

pub fn projection_transform(dw: f32, dh: f32) -> Perspective3<f32>
{
	Perspective3::new(dw / dh, f32::pi() / 2., 0.1, 2000.)
}

pub fn mat4_to_transform(mat: Matrix4<f32>) -> Transform
{
	let mut trans = Transform::identity();
	for i in 0..4
	{
		for j in 0..4
		{
			trans.get_matrix_mut()[j][i] = mat[(i, j)];
		}
	}
	trans
}

pub fn camera_project(x: f32, y: f32, z: f32, player_z: f32) -> Isometry3<f32>
{
	let eye = Point3::new(x, y, z);
	let target = Point3::new(x, y, player_z);
	let view = Isometry3::look_at_rh(&eye, &target, &Vector3::y());
	view
}

pub fn random_color(seed: u64, saturation: f32, value: f32) -> Color
{
	let mut rng = StdRng::seed_from_u64(seed);
	Color::from_hsv(rng.gen_range(0. ..360.), saturation, value)
}

pub trait ColorExt
{
	fn interpolate(&self, other: Color, f: f32) -> Color;
}

impl ColorExt for Color
{
	fn interpolate(&self, other: Color, f: f32) -> Color
	{
		let fi = 1. - f;
		let (r, g, b, a) = self.to_rgba_f();
		let (or, og, ob, oa) = other.to_rgba_f();
		Color::from_rgba_f(
			r * fi + or * f,
			g * fi + og * f,
			b * fi + ob * f,
			a * fi + oa * f,
		)
	}
}

pub fn max<T: PartialOrd>(x: T, y: T) -> T
{
	if x > y
	{
		x
	}
	else
	{
		y
	}
}

pub fn min<T: PartialOrd>(x: T, y: T) -> T
{
	if x < y
	{
		x
	}
	else
	{
		y
	}
}

pub fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T
{
	if x < min
	{
		min
	}
	else if x > max
	{
		max
	}
	else
	{
		x
	}
}

pub fn sigmoid(x: f32) -> f32
{
	1. / (1. + (-x).exp())
}

pub fn read_to_string(path: &str) -> Result<String>
{
	fs::read_to_string(path)
		.map_err(|e| Error::new(format!("Couldn't read '{}'", path), Some(Box::new(e))))
}

pub fn load_config<T: DeserializeOwned + Clone>(file: &str) -> Result<T>
{
	let contents = read_to_string(file)?;
	let mut source = Source::new(path::Path::new(file), &contents);
	let element = ConfigElement::from_source(&mut source)
		.map_err(|e| Error::new(format!("Config parsing error"), Some(Box::new(e))))?;
	from_element::<T>(&element, Some(&source))
		.map_err(|e| Error::new(format!("Config parsing error"), Some(Box::new(e))))
}

pub fn load_bitmap(core: &Core, file: &str) -> Result<Bitmap>
{
	Ok(Bitmap::load(&core, file).map_err(|_| format!("Couldn't load {}", file))?)
}

pub fn load_sample(audio: &AudioAddon, path: &str) -> Result<Sample>
{
	Ok(Sample::load(audio, path).map_err(|_| format!("Couldn't load '{}'", path))?)
}
