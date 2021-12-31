use crate::{game_state, utils};
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

#[derive(Debug, Copy, Clone)]
pub enum DamageType
{
	Regular,
	Flame,
	Cold(f32),
}

impl DamageType
{
	pub fn loose_eq(&self, other: &DamageType) -> bool
	{
		match self
		{
			DamageType::Regular => match other
			{
				DamageType::Regular => true,
				_ => false,
			},
			DamageType::Flame => match other
			{
				DamageType::Flame => true,
				_ => false,
			},
			DamageType::Cold(_) => match other
			{
				DamageType::Cold(_) => true,
				_ => false,
			},
		}
	}
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

#[derive(Debug, Copy, Clone)]
pub struct AmmoRegen
{
	pub weapon_type: WeaponType,
	pub ammount: i32,
	pub time_to_regen: f64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WeaponType
{
	SantaGun,
	BuggyGun,
	RocketGun,
	FlameGun,
	BigFlameGun,
	FreezeGun,
	OrbGun,
	SnowmanGun,
	BigSnowmanGun,
	KrampusGun,
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
			WeaponType::BigFlameGun => 16.,
			WeaponType::FreezeGun => 8.,
			WeaponType::OrbGun => 4.,
			WeaponType::SnowmanGun => 12.,
			WeaponType::BigSnowmanGun => 14.,
			WeaponType::KrampusGun => 14.,
		}
	}

	pub fn ammo_usage(&self) -> i32
	{
		match self
		{
			WeaponType::SantaGun => 1,
			WeaponType::BuggyGun => 2,
			WeaponType::RocketGun => 1,
			WeaponType::FlameGun => 1,
			WeaponType::BigFlameGun => 1,
			WeaponType::FreezeGun => 1,
			WeaponType::OrbGun => 1,
			WeaponType::SnowmanGun => 2,
			WeaponType::BigSnowmanGun => 1,
			WeaponType::KrampusGun => 1,
		}
	}
}

#[derive(Debug, Clone)]
pub struct Weapon
{
	pub delay: f64,
	pub time_to_fire: f64,
	pub weapon_type: WeaponType,
	pub ammo: i32,
	pub max_ammo: i32,
	pub selectable: bool,
}

impl Weapon
{
	pub fn santa_gun() -> Self
	{
		Weapon {
			delay: 0.2,
			time_to_fire: 0.,
			weapon_type: WeaponType::SantaGun,
			ammo: 50,
			max_ammo: 200,
			selectable: true,
		}
	}

	pub fn grinch_gun() -> Self
	{
		Weapon {
			delay: 0.2,
			time_to_fire: 0.,
			weapon_type: WeaponType::SantaGun,
			ammo: 5,
			max_ammo: 5,
			selectable: true,
		}
	}

	pub fn buggy_gun() -> Self
	{
		Weapon {
			delay: 0.2,
			time_to_fire: 0.,
			weapon_type: WeaponType::BuggyGun,
			ammo: 300,
			max_ammo: 300,
			selectable: true,
		}
	}

	pub fn rocket_gun() -> Self
	{
		Weapon {
			delay: 0.75,
			time_to_fire: 0.,
			weapon_type: WeaponType::RocketGun,
			ammo: 25,
			max_ammo: 100,
			selectable: true,
		}
	}

	pub fn flame_gun() -> Self
	{
		Weapon {
			delay: 0.125,
			time_to_fire: 0.,
			weapon_type: WeaponType::FlameGun,
			ammo: 20,
			max_ammo: 20,
			selectable: false,
		}
	}

	pub fn big_flame_gun() -> Self
	{
		Weapon {
			delay: 0.125,
			time_to_fire: 0.,
			weapon_type: WeaponType::BigFlameGun,
			ammo: 20,
			max_ammo: 20,
			selectable: false,
		}
	}

	pub fn freeze_gun() -> Self
	{
		Weapon {
			delay: 0.125,
			time_to_fire: 0.,
			weapon_type: WeaponType::FreezeGun,
			ammo: 0,
			max_ammo: 300,
			selectable: false,
		}
	}

	pub fn orb_gun() -> Self
	{
		Weapon {
			delay: 0.5,
			time_to_fire: 0.,
			weapon_type: WeaponType::OrbGun,
			ammo: 0,
			max_ammo: 50,
			selectable: false,
		}
	}

	pub fn snowman_gun() -> Self
	{
		Weapon {
			delay: 1.,
			time_to_fire: 0.,
			weapon_type: WeaponType::SnowmanGun,
			ammo: 0,
			max_ammo: 2,
			selectable: false,
		}
	}

	pub fn big_snowman_gun() -> Self
	{
		Weapon {
			delay: 1.,
			time_to_fire: 0.,
			weapon_type: WeaponType::BigSnowmanGun,
			ammo: 0,
			max_ammo: 2,
			selectable: false,
		}
	}
	pub fn krampus_gun() -> Self
	{
		Weapon {
			delay: 1.,
			time_to_fire: 0.,
			weapon_type: WeaponType::KrampusGun,
			ammo: 0,
			max_ammo: 2,
			selectable: false,
		}
	}

	pub fn add_ammo(&mut self, ammount: i32) -> bool
	{
		if self.ammo >= self.max_ammo
		{
			false
		}
		else
		{
			self.ammo = utils::min(self.max_ammo, self.ammo + ammount);
			true
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
	Item
	{
		item_type: ItemType,
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

pub struct Message
{
	pub message: String,
}

pub struct Trigger
{
	pub delay: f64,
	pub time_to_trigger: f64,
	pub targets: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct Moveable
{
	pub speed: f32,
	pub rot_speed: f32,
	pub can_strafe: bool,
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
	pub armour: f32,

	pub max_health: f32,
	pub max_armour: f32,

	pub immunities: Vec<DamageType>,
}

impl Health
{
	pub fn damage(&mut self, damage: Damage, factor: f32) -> bool
	{
		for immunity in &self.immunities
		{
			if immunity.loose_eq(&damage.damage_type)
			{
				return false;
			}
		}

		let mut amount = damage.amount * factor;
		let prevented_by_armor = utils::min(self.armour, amount / 3.);
		self.armour -= prevented_by_armor;
		amount -= prevented_by_armor;
		self.health = utils::max(0., self.health - amount);
		true
	}

	pub fn add_armour(&mut self, amount: f32) -> bool
	{
		if self.armour >= self.max_armour
		{
			false
		}
		else
		{
			self.armour = utils::min(self.max_armour, self.armour + amount);
			true
		}
	}

	pub fn add_health(&mut self, amount: f32) -> bool
	{
		if self.health >= self.max_health
		{
			false
		}
		else
		{
			self.health = utils::min(self.max_health, self.health + amount);
			true
		}
	}
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

#[derive(Debug, Copy, Clone)]
pub enum ItemType
{
	Shard,
	Suit,
	Flask,
	Heart,
	BulletAmmo,
	FreezeAmmo,
	OrbAmmo,
	ExtraLife,
	FreezeGun,
	OrbGun,
}

impl ItemType
{
	pub fn sprite_sheet(&self) -> &str
	{
		match self
		{
			ItemType::Shard => "data/armor_shard.cfg",
			ItemType::Suit => "data/armor_suit.cfg",
			ItemType::Flask => "data/flask.cfg",
			ItemType::Heart => "data/heart.cfg",
			ItemType::BulletAmmo => "data/bullet_ammo.cfg",
			ItemType::FreezeAmmo => "data/freeze_ammo.cfg",
			ItemType::OrbAmmo => "data/star_ammo.cfg",
			ItemType::ExtraLife => "data/extra_life.cfg",
			ItemType::FreezeGun => "data/freeze_gun.cfg",
			ItemType::OrbGun => "data/orb_gun.cfg",
		}
	}

	pub fn size(&self) -> f32
	{
		match self
		{
			ItemType::Shard => 6.,
			ItemType::Suit => 8.,
			ItemType::Flask => 6.,
			ItemType::Heart => 8.,
			ItemType::BulletAmmo => 4.,
			ItemType::FreezeAmmo => 4.,
			ItemType::OrbAmmo => 4.,
			ItemType::ExtraLife => 10.,
			ItemType::FreezeGun => 10.,
			ItemType::OrbGun => 10.,
		}
	}
}
