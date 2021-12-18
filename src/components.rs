use na::{Point3, Vector3};
use nalgebra as na;

#[derive(Debug, Copy, Clone)]
pub struct Position
{
	pub pos: Point3<f32>,
	pub dir: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity
{
	pub vel: Vector3<f32>,
	pub dir_vel: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Solid
{
	pub size: f32,
	pub mass: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Drawable;

#[derive(Debug, Clone)]
pub struct Weapon
{
	pub delay: f64,
	pub time_to_fire: f64,
}

#[derive(Debug, Clone)]
pub struct WeaponSet
{
	pub weapons: Vec<Weapon>,
	pub want_to_fire: bool,
	pub cur_weapon: usize,
}

#[derive(Debug, Clone)]
pub struct TimeToDie
{
	pub time_to_die: f64,
}

#[derive(Debug, Copy, Clone)]
pub enum Effect
{
	Die,
	Hurt
	{
		damage: f32,
	},
}

#[derive(Debug, Clone)]
pub struct OnContactEffect
{
	pub effects: Vec<Effect>,
}

#[derive(Debug, Copy, Clone)]
pub struct Health
{
	pub health: f32,
}

#[derive(Debug, Copy, Clone)]
pub enum Status
{
	Idle,
	Moving(hecs::Entity),
	Attacking(hecs::Entity),
}

#[derive(Debug, Copy, Clone)]
pub struct AI
{
	pub sense_range: f32,
	pub disengage_range: f32,
	pub attack_range: f32,
	pub status: Status,
	pub time_to_check_status: f64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Team
{
	Player,
	Monster,
}
