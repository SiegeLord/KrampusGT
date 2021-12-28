use crate::game_state;
use na::{Point2, Point3, Vector3};
use nalgebra as na;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Copy, Clone)]
pub struct AffectedByFriction;

#[derive(Debug, Copy, Clone)]
pub struct CreationTime
{
	pub time: f64,
}

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
pub struct GasCloud
{
	pub base_size: f32,
	pub growth_rate: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Freezable
{
	pub amount: f32,
}

impl Freezable
{
	pub fn is_frozen(&self) -> bool
	{
		self.amount > 1.
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DamageType
{
	Regular,
	Flame,
	Cold,
}

#[derive(Debug, Copy, Clone)]
pub struct Damage
{
	pub amount: f32,
	pub damage_type: DamageType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CollisionClass
{
	Regular,
	Tiny,
	Gas,
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
				CollisionClass::Gas => true,
			},
			CollisionClass::Tiny => match other
			{
				CollisionClass::Regular => true,
				CollisionClass::Tiny => false,
				CollisionClass::Gas => false,
			},
			CollisionClass::Gas => match other
			{
				CollisionClass::Regular => false,
				CollisionClass::Tiny => false,
				CollisionClass::Gas => false,
			},
		}
	}

	pub fn interacts(&self) -> bool
	{
		match self
		{
			CollisionClass::Regular => true,
			CollisionClass::Tiny => true,
			CollisionClass::Gas => false,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WeaponType
{
	SantaGun,
	BuggyGun,
	RocketGun,
	FlameGun,
	FreezeGun,
	OrbGun,
}

impl WeaponType
{
	pub fn proj_size(&self) -> f32
	{
		match self
		{
			WeaponType::SantaGun => 4.,
			WeaponType::BuggyGun => 4.,
			WeaponType::RocketGun => 8.,
			WeaponType::FlameGun => 8.,
			WeaponType::FreezeGun => 8.,
			WeaponType::OrbGun => 4.,
		}
	}
}

#[derive(Debug, Clone)]
pub struct Weapon
{
	pub delay: f64,
	pub time_to_fire: f64,
	pub weapon_type: WeaponType,
}

impl Weapon
{
	pub fn santa_gun() -> Self
	{
		Weapon {
			delay: 0.2,
			time_to_fire: 0.,
			weapon_type: WeaponType::SantaGun,
		}
	}

	pub fn buggy_gun() -> Self
	{
		Weapon {
			delay: 0.2,
			time_to_fire: 0.,
			weapon_type: WeaponType::BuggyGun,
		}
	}

	pub fn rocket_gun() -> Self
	{
		Weapon {
			delay: 0.75,
			time_to_fire: 0.,
			weapon_type: WeaponType::RocketGun,
		}
	}

	pub fn flame_gun() -> Self
	{
		Weapon {
			delay: 0.125,
			time_to_fire: 0.,
			weapon_type: WeaponType::FlameGun,
		}
	}

	pub fn freeze_gun() -> Self
	{
		Weapon {
			delay: 0.125,
			time_to_fire: 0.,
			weapon_type: WeaponType::FreezeGun,
		}
	}

	pub fn orb_gun() -> Self
	{
		Weapon {
			delay: 0.5,
			time_to_fire: 0.,
			weapon_type: WeaponType::OrbGun,
		}
	}
}

#[derive(Debug, Clone)]
pub struct WeaponSet
{
	pub weapons: HashMap<WeaponType, Weapon>,
	pub want_to_fire: bool,
	pub cur_weapon: WeaponType,
	pub last_fire_time: f64,
}

#[derive(Debug, Clone)]
pub struct TimeToDie
{
	pub time_to_die: f64,
}

#[derive(Debug, Copy, Clone)]
pub enum ContactEffect
{
	Die,
	Hurt
	{
		damage: Damage,
	},
	DamageOverTime
	{
		damage_rate: Damage,
	},
}

#[derive(Debug, Clone)]
pub struct OnContactEffect
{
	pub effects: Vec<ContactEffect>,
}

pub enum DeathEffect
{
	Spawn(
		Box<
			dyn Fn(
					Point3<f32>,
					f32,
					Vector3<f32>,
					&mut game_state::GameState,
					&mut hecs::World,
				) -> hecs::Entity
				+ Sync
				+ Send,
		>,
	),
	DamageInRadius
	{
		damage: Damage,
		radius: f32,
		push_strength: f32,
	},
	Orb,
	IncrementCounter
	{
		target: String,
	},
}

pub struct OnDeathEffect
{
	pub effects: Vec<DeathEffect>,
}

pub struct Active
{
	pub active: bool,
}

pub struct Spawner
{
	pub delay: f64,
	pub count: i32,
	pub max_count: i32,
	pub time_to_spawn: f64,
	pub spawn_fn: Arc<
		dyn Fn(
				Point3<f32>,
				f32,
				Vector3<f32>,
				&mut game_state::GameState,
				&mut hecs::World,
			) -> hecs::Entity
			+ Sync
			+ Send,
	>,
}

pub struct AreaTrigger
{
	pub start: Point2<f32>,
	pub end: Point2<f32>,
	pub targets: Vec<String>,
}

pub struct Counter
{
	pub count: i32,
	pub max_count: i32,
	pub targets: Vec<String>,
}

pub struct Deleter
{
	pub targets: Vec<String>,
}

pub struct PlayerStart;

#[derive(Debug, Clone)]
pub struct Health
{
	pub health: f32,
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
	Neutral,
}

impl Team
{
	pub fn friendly(&self, other: &Self) -> bool
	{
		match self
		{
			Team::Player => match other
			{
				Team::Player => true,
				Team::Monster => false,
				Team::Neutral => true,
			},
			Team::Monster => match other
			{
				Team::Player => false,
				Team::Monster => true,
				Team::Neutral => true,
			},
			Team::Neutral => match other
			{
				Team::Player => true,
				Team::Monster => true,
				Team::Neutral => true,
			},
		}
	}
}

pub struct Vehicle
{
	pub contents:
		Option<Box<dyn Fn(Point3<f32>, f32, &mut hecs::World) -> hecs::Entity + Sync + Send>>,
}
