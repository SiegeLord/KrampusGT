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
	Isometry3, Matrix4, Perspective3, Point2, Point3, Quaternion, RealField, Rotation3, Unit,
	Vector2, Vector3, Vector4,
};
use nalgebra as na;

pub fn projection_transform(dw: f32, dh: f32) -> Perspective3<f32>
{
	Perspective3::new(dw / dh, f32::pi() / 2., 1., 2000.)
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

pub fn camera_project(x: f32, y: f32, z: f32, player_x: f32, player_z: f32) -> Isometry3<f32>
{
	let eye = Point3::new(x, y, z);
	let target = Point3::new(player_x, y, player_z);
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

pub fn nearest_line_point(v1: Point2<f32>, v2: Point2<f32>, test_point: Point2<f32>)
	-> Point2<f32>
{
	let v1_t = test_point - v1;

	let v2_v1 = v2 - v1;
	let v2_v1_norm_sq = max(v2_v1.norm_squared(), 1e-20);

	let dot = v1_t.dot(&v2_v1) / v2_v1_norm_sq;

	if dot < 0.
	{
		v1
	}
	else if dot > 1.
	{
		v2
	}
	else
	{
		v1 + dot * (v2 - v1)
	}
}

pub fn nearest_poly_point(vs: &[Point2<f32>], test_point: Point2<f32>) -> Point2<f32>
{
	assert!(vs.len() >= 3);
	let mut best_dist_sq = f32::INFINITY;
	let mut best_point = Point2::new(0., 0.);

	for idx in 0..vs.len()
	{
		let v1 = vs[idx];
		let v2 = vs[(idx + 1) % vs.len()];

		let cand_point = nearest_line_point(v1, v2, test_point);
		let cand_dist_sq = (cand_point - test_point).norm_squared();
		if cand_dist_sq < best_dist_sq
		{
			best_point = cand_point;
			best_dist_sq = cand_dist_sq;
		}
	}
	best_point
}

pub fn is_inside_poly(vs: &[Point2<f32>], test_point: Point2<f32>) -> bool
{
	assert!(vs.len() >= 3);
	let mut inside = true;

	for idx in 0..vs.len()
	{
		let v1 = vs[idx];
		let v2 = vs[(idx + 1) % vs.len()];
		let v1_v2 = v2 - v1;
		let normal = Vector2::new(-v1_v2.y, v1_v2.x);

		let v1_t = test_point - v1;
		inside &= v1_t.dot(&normal) < 0.;
		if !inside
		{
			return false;
		}
	}
	true
}

#[test]
fn test_nearest_line_point()
{
	let v1 = Point2::new(1., 2.);
	let v2 = Point2::new(3., 4.);

	let t = Point2::new(-1., -1.);
	let n = nearest_line_point(v1, v2, t);
	assert!(n == v1);

	let t = Point2::new(5., 5.);
	let n = nearest_line_point(v1, v2, t);
	assert!((n - v2).norm() < 1e-3);

	let t = Point2::new(2., 3.);
	let n = nearest_line_point(v1, v2, t);
	dbg!(n, t);
	assert!((n - t).norm() < 1e-3);

	let t = Point2::new(1., 4.);
	let n = nearest_line_point(v1, v2, t);
	assert!((n - Point2::new(2., 3.)).norm() < 1e-3);
}

#[test]
fn is_inside_poly_test()
{
	let vs = [
		Point2::new(0., 0.),
		Point2::new(0., 3.),
		Point2::new(3., 3.),
		Point2::new(3., 0.),
	];

	assert!(is_inside_poly(&vs, Point2::new(1., 1.)));
	assert!(!is_inside_poly(&vs, Point2::new(-1., -1.)));

	let vs = [
		Point2::new(-1., 1.),
		Point2::new(1., 3.),
		Point2::new(4., -3.),
		Point2::new(-1., -1.),
	];
	assert!(is_inside_poly(&vs, Point2::new(0., 0.)));
}
