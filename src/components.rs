use na::{Point3, Vector3};
use nalgebra as na;

#[derive(Copy, Clone)]
pub struct Position
{
	pub pos: Point3<f32>,
	pub dir: f32,
}

#[derive(Copy, Clone)]
pub struct Velocity
{
	pub vel: Vector3<f32>,
	pub dir_vel: f32,
}

#[derive(Copy, Clone)]
pub struct Solid
{
	pub size: f32,
	pub mass: f32,
}

#[derive(Copy, Clone)]
pub struct Drawable;
