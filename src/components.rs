use na::{Point2, Point3, Vector3};
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CollisionClass
{
	Regular,
	Tiny,
}

impl CollisionClass
{
	pub fn collides_with(&self, other: CollisionClass) -> bool
	{
		match self
		{
			CollisionClass::Regular => match other
			{
				CollisionClass::Regular => true,
				CollisionClass::Tiny => true,
			},
			CollisionClass::Tiny => match other
			{
				CollisionClass::Regular => true,
				CollisionClass::Tiny => false,
			},
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct Solid
{
	pub size: f32,
	pub mass: f32,
	pub collision_class: CollisionClass,
}

#[derive(Debug, Clone)]
pub struct Drawable
{
	pub size: f32,
	pub sprite_sheet: String,
}

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
	pub last_fire_time: f64,
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

#[derive(Debug, Clone)]
pub struct Health
{
	pub health: f32,
	pub corpse_sprite_sheet: String,
}

#[derive(Debug, Copy, Clone)]
pub enum Status
{
	Idle,
	Attacking(hecs::Entity),
	Searching(hecs::Entity, Point3<f32>, f64),
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
