use crate::error::Result;
use crate::{atlas, components, game_state, spatial_grid, utils};

use allegro::*;
use allegro_font::*;
use allegro_primitives::*;
use allegro_sys::*;
use na::{
	Isometry3, Matrix4, Perspective3, Point2, Point3, Quaternion, RealField, Rotation2, Rotation3,
	Unit, Vector2, Vector3, Vector4,
};
use nalgebra as na;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub const TILE: f32 = 64.;
pub const FREEZE_FACTOR: f32 = 0.5;

struct Bucket
{
	vertices: Vec<Vertex>,
	indices: Vec<i32>,
}

struct Scene
{
	buckets: Vec<Bucket>,
}

impl Scene
{
	fn new() -> Self
	{
		Scene { buckets: vec![] }
	}

	fn ensure_bucket(&mut self, page: usize)
	{
		while page >= self.buckets.len()
		{
			self.buckets.push(Bucket {
				vertices: vec![],
				indices: vec![],
			});
		}
	}

	fn add_vertices(&mut self, vertices: &[Vertex], page: usize)
	{
		self.ensure_bucket(page);
		self.buckets[page].vertices.extend(vertices);
	}

	fn add_indices(&mut self, indices: &[i32], page: usize)
	{
		self.ensure_bucket(page);
		self.buckets[page].indices.extend(indices);
	}

	fn add_vertex(&mut self, vertex: Vertex, page: usize)
	{
		self.ensure_bucket(page);
		self.buckets[page].vertices.push(vertex);
	}

	fn add_index(&mut self, index: i32, page: usize)
	{
		self.ensure_bucket(page);
		self.buckets[page].indices.push(index);
	}

	fn num_vertices(&mut self, page: usize) -> i32
	{
		self.ensure_bucket(page);
		self.buckets[page].vertices.len() as i32
	}
}

fn draw_billboard(
	pos: Point3<f32>, camera_angle: f32, size: f32, bitmap: &atlas::AtlasBitmap, color: Color,
	scene: &mut Scene,
)
{
	let rot = Rotation2::new(camera_angle);
	let diff = rot * Vector2::new(0., 1.);

	let width = size;
	let height = bitmap.height() * size / bitmap.width();

	let horiz_offt = width / 2. * Vector3::new(-diff.y, 0., diff.x);
	let vert_offt = height * Vector3::new(0., 1., 0.);

	let pos1 = pos - horiz_offt + vert_offt;
	let pos2 = pos + horiz_offt + vert_offt;
	let pos3 = pos + horiz_offt;
	let pos4 = pos - horiz_offt;

	let idx = scene.num_vertices(bitmap.page);
	scene.add_vertices(
		&[
			Vertex {
				x: pos1.x,
				y: pos1.y,
				z: pos1.z,
				u: bitmap.start.x,
				v: bitmap.start.y,
				color: color,
			},
			Vertex {
				x: pos4.x,
				y: pos4.y,
				z: pos4.z,
				u: bitmap.start.x,
				v: bitmap.end.y,
				color: color,
			},
			Vertex {
				x: pos3.x,
				y: pos3.y,
				z: pos3.z,
				u: bitmap.end.x,
				v: bitmap.end.y,
				color: color,
			},
			Vertex {
				x: pos2.x,
				y: pos2.y,
				z: pos2.z,
				u: bitmap.end.x,
				v: bitmap.start.y,
				color: color,
			},
		],
		bitmap.page,
	);

	scene.add_indices(
		&[idx + 0, idx + 1, idx + 2, idx + 0, idx + 2, idx + 3],
		bitmap.page,
	);
}

fn get_float_property(property: &str, obj: &tiled::objects::Object) -> Option<Result<f32>>
{
	obj.properties.get(property).map(|p| match p
	{
		tiled::properties::PropertyValue::FloatValue(v) => Ok(*v),
		other => Err(format!(
			"Invalid value for '{}' in object {:?}: {:?}",
			property, obj, other
		)
		.into()),
	})
}

fn get_int_property(property: &str, obj: &tiled::objects::Object) -> Option<Result<i32>>
{
	obj.properties.get(property).map(|p| match p
	{
		tiled::properties::PropertyValue::IntValue(v) => Ok(*v),
		other => Err(format!(
			"Invalid value for '{}' in object {:?}: {:?}",
			property, obj, other
		)
		.into()),
	})
}

fn get_bool_property(property: &str, obj: &tiled::objects::Object) -> Option<Result<bool>>
{
	obj.properties.get(property).map(|p| match p
	{
		tiled::properties::PropertyValue::BoolValue(v) => Ok(*v),
		other => Err(format!(
			"Invalid value for '{}' in object {:?}: {:?}",
			property, obj, other
		)
		.into()),
	})
}

fn get_target_property(
	property: &str, obj: &tiled::objects::Object, id_to_name: &HashMap<u32, String>,
) -> Option<Result<String>>
{
	obj.properties.get(property).and_then(|p| match p
	{
		tiled::properties::PropertyValue::ObjectValue(v) =>
		{
			if *v == 0
			{
				return None;
			}
			else
			{
				Some(Ok(id_to_name[&v].clone()))
			}
		}
		other => Some(Err(format!(
			"Invalid value for '{}' in object {:?}: {:?}",
			property, obj, other
		)
		.into())),
	})
}

fn get_string_property(property: &str, obj: &tiled::objects::Object) -> Option<Result<String>>
{
	obj.properties.get(property).map(|p| match p
	{
		tiled::properties::PropertyValue::StringValue(v) => Ok(v.clone()),
		other => Err(format!(
			"Invalid value for '{}' in object {:?}: {:?}",
			property, obj, other
		)
		.into()),
	})
}

fn get_targets_property(
	obj: &tiled::objects::Object, id_to_name: &HashMap<u32, String>,
) -> Result<Vec<String>>
{
	let mut ret = vec![];
	for i in 0..10
	{
		let property = format!("target{}", i);
		let value = get_target_property(&property, obj, id_to_name);
		if value.is_none()
		{
			break;
		}
		ret.push(value.unwrap()?);
	}
	Ok(ret)
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct LevelDesc
{
	level: String,
	meshes: String,
}

pub struct Level
{
	width: i32,
	height: i32,
	tile_meshes: Vec<Mesh>,
	tiles: Vec<i32>,
}

impl Level
{
	pub fn new(
		filename: &str, named_entities: &mut HashMap<String, hecs::Entity>,
		state: &mut game_state::GameState, world: &mut hecs::World,
	) -> Result<Self>
	{
		let desc: LevelDesc = utils::load_config(filename)?;

		let tile_meshes = load_meshes(&desc.meshes);

		let map = tiled::parse_file(&Path::new(&desc.level))?;
		let tile_width = map.tile_width as f32;

		let layer_tiles = &map.layers[0].tiles;
		let layer_tiles = match layer_tiles
		{
			tiled::layers::LayerData::Finite(layer_tiles) => layer_tiles,
			_ => return Err("Bad map!".to_string().into()),
		};

		let height = layer_tiles.len();
		let width = layer_tiles[0].len();

		let tileset = &map.tilesets[0];
		let mut tiles = Vec::with_capacity(width * height);

		for row in layer_tiles
		{
			for tile in row
			{
				tiles.push((utils::max(tile.gid, tileset.first_gid) - tileset.first_gid) as i32);
			}
		}

		let mut tile_meshes_vec = Vec::with_capacity(tile_meshes.len());
		for i in 0..tile_meshes.len()
		{
			tile_meshes_vec.push(tile_meshes[&i.to_string()].clone())
		}

		let mut id_to_name = HashMap::new();
		let objects = &map.object_groups[0].objects;
		for obj in objects
		{
			id_to_name.insert(obj.id, format!("{}|{}", obj.name, obj.id));
		}
		for obj in objects
		{
			let start = Point3::new(obj.x, 0., obj.y) / tile_width * TILE;
			let end = Point3::new(obj.x + obj.width, 0., obj.y + obj.height) / tile_width * TILE;
			let center = start + (end - start) / 2.;
			dbg!(obj, start, tile_width, TILE, center);

			let entity = match &obj.obj_type[..]
			{
				"start" => spawn_player_start(
					center,
					get_float_property("dir", obj).unwrap_or(Ok(0.))?,
					get_bool_property("active", obj).unwrap_or(Ok(false))?,
					world,
				),
				"area trigger" => spawn_area_trigger(
					start.xz(),
					end.xz(),
					get_targets_property(obj, &id_to_name)?,
					get_bool_property("active", obj).unwrap_or(Ok(true))?,
					world,
				),
				"spawner" => spawn_spawner(
					center,
					get_float_property("dir", obj).unwrap_or(Ok(0.))?,
					&get_target_property("counter", obj, &id_to_name)
						.unwrap_or(Ok("".to_string()))?,
					get_bool_property("active", obj).unwrap_or(Ok(false))?,
					str_to_spawn_fn(&get_string_property("spawn", obj).unwrap_or(Err(
						format!("Spawner {:?} needs 'spawn' specified.", obj).into(),
					))?)?,
					get_int_property("max_count", obj).unwrap_or(Ok(1))?,
					get_float_property("delay", obj).unwrap_or(Ok(0.1))?,
					world,
				),
				"counter" => spawn_counter(
					get_int_property("max_count", obj).unwrap_or(Err(format!(
						"Counter {:?} needs 'max_count' specified.",
						obj
					)
					.into()))?,
					get_targets_property(obj, &id_to_name)?,
					get_bool_property("active", obj).unwrap_or(Ok(true))?,
					world,
				),
				"trigger" => spawn_trigger(
					get_float_property("delay", obj).unwrap_or(Ok(0.))? as f64,
					get_targets_property(obj, &id_to_name)?,
					get_bool_property("active", obj).unwrap_or(Ok(false))?,
					world,
				),
				"deleter" => spawn_deleter(
					get_targets_property(obj, &id_to_name)?,
					get_bool_property("active", obj).unwrap_or(Ok(false))?,
					world,
				),
				"object" =>
				{
					let spawn_fn = str_to_spawn_fn(&get_string_property("spawn", obj).unwrap_or(
						Err(format!("Object {:?} needs 'spawn' specified.", obj).into()),
					)?)?;
					spawn_fn(
						center,
						get_float_property("dir", obj).unwrap_or(Ok(0.))?,
						&get_target_property("counter", obj, &id_to_name)
							.unwrap_or(Ok("".to_string()))?,
						state,
						world,
					)
				}
				other => return Err(format!("Unknown object type '{}'", other).into()),
			};
			named_entities.insert(format!("{}|{}", obj.name, obj.id), entity);
		}

		Ok(Level {
			width: width as i32,
			height: height as i32,
			tile_meshes: tile_meshes_vec,
			tiles: tiles,
		})
	}

	fn draw(&self, state: &game_state::GameState, scene: &mut Scene)
	{
		let bmp = &state
			.get_sprite_sheet("data/terrain.cfg")
			.unwrap()
			.orientations[0]
			.idle[0];

		for z in 0..self.height
		{
			for x in 0..self.width
			{
				let shift_x = x as f32 * TILE + TILE / 2.;
				let shift_z = z as f32 * TILE + TILE / 2.;

				let idx = scene.num_vertices(bmp.page);

				let mesh = &self.tile_meshes[self.tiles[(x + z * self.width) as usize] as usize];

				for vtx in &mesh.vtxs
				{
					scene.add_vertex(
						Vertex {
							x: vtx.x + shift_x,
							y: vtx.y,
							z: vtx.z + shift_z,
							u: bmp.start.x + bmp.width() * vtx.u,
							v: bmp.start.y + bmp.height() * vtx.v,
							color: Color::from_rgb_f(1., 1., 1.), //Color::from_rgb_f(vtx.x / 64., vtx.z / 64., vtx.y / 64.),
						},
						bmp.page,
					);
				}
				for vec_idx in &mesh.idxs
				{
					scene.add_index(vec_idx + idx, bmp.page);
				}
			}
		}
	}

	pub fn tile_is_empty(&self, tile: i32) -> bool
	{
		tile <= 2 || tile >= 24
	}

	pub fn check_collision(&self, loc: Point3<f32>, size: f32) -> Option<Vector3<f32>>
	{
		let center_x = (loc.x / TILE).floor() as i32;
		let center_z = (loc.z / TILE).floor() as i32;

		let mut res = Vector2::zeros();
		// TODO: This -1/1 isn't really right
		for map_z in center_z - 1..=center_z + 1
		{
			for map_x in center_x - 1..=center_x + 1
			{
				if map_x < 0 || map_x >= self.width || map_z < 0 || map_z >= self.height
				{
					continue;
				}
				let tile = self.tiles[(map_z * self.width + map_x) as usize];
				if self.tile_is_empty(tile)
				{
					continue;
				}

				let cx = map_x as f32 * TILE;
				let cz = map_z as f32 * TILE;

				let vs = [
					Point2::new(cx, cz),
					Point2::new(cx, cz + TILE),
					Point2::new(cx + TILE, cz + TILE),
					Point2::new(cx + TILE, cz),
				];

				let nearest_point = utils::nearest_poly_point(&vs, loc.xz());

				let nearest_dist = utils::max(1e-20, (loc.xz() - nearest_point).norm());
				let inside = utils::is_inside_poly(&vs, loc.xz());
				if nearest_dist < size || inside
				{
					let new_dir = if inside
					{
						(nearest_point - loc.xz()) * (nearest_dist + size) / nearest_dist
					}
					else
					{
						(loc.xz() - nearest_point) * (size - nearest_dist) / nearest_dist
					};

					if new_dir.norm() > res.norm()
					{
						res = new_dir;
					}
				}
			}
		}
		if res.norm() > 0.
		{
			Some(Vector3::new(res.x, 0., res.y))
		}
		else
		{
			None
		}
	}

	pub fn check_segment(&self, start: Point2<f32>, end: Point2<f32>, size: f32) -> bool
	{
		let start_x = (start.x / TILE).floor() as i32;
		let start_z = (start.y / TILE).floor() as i32;
		let end_x = (end.x / TILE).floor() as i32;
		let end_z = (end.y / TILE).floor() as i32;

		let (start_x, end_x) = if start_x > end_x
		{
			(end_x, start_x)
		}
		else
		{
			(start_x, end_x)
		};
		let (start_z, end_z) = if start_z > end_z
		{
			(end_z, start_z)
		}
		else
		{
			(start_z, end_z)
		};

		// TODO: This -1/1 isn't really right
		for map_z in start_z - 1..end_z + 1
		{
			for map_x in start_x - 1..end_x + 1
			{
				if map_x < 0 || map_x >= self.width || map_z < 0 || map_z >= self.height
				{
					continue;
				}
				let tile = self.tiles[(map_z * self.width + map_x) as usize];
				if self.tile_is_empty(tile)
				{
					continue;
				}

				let cx = map_x as f32 * TILE;
				let cz = map_z as f32 * TILE;

				let rect = spatial_grid::Rect {
					start: Point2::new(cx - size, cz - size),
					end: Point2::new(cx + size + TILE, cz + size + TILE),
				};

				if rect.intersects_with_segment(start, end)
				{
					return true;
				}
			}
		}
		false
	}
}

pub fn spawn_projectile(
	pos: Point3<f32>, dir: f32, lifetime: f64, state: &mut game_state::GameState,
	world: &mut hecs::World,
) -> hecs::Entity
{
	let size = components::WeaponType::SantaGun.proj_size();
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: 256. * utils::dir_vec3(dir),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/bullet.cfg".into(),
		},
		components::Solid {
			size: size / 2.,
			mass: 0.,
			collision_class: components::CollisionClass::Tiny,
		},
		components::TimeToDie {
			time_to_die: state.time() + lifetime,
		},
		components::OnContactEffect {
			effects: vec![
				components::ContactEffect::Die,
				components::ContactEffect::Hurt {
					damage: components::Damage {
						amount: 6.,
						damage_type: components::DamageType::Regular,
					},
				},
			],
		},
		components::OnDeathEffect {
			effects: vec![components::DeathEffect::Spawn(Box::new(
				move |pos, _, _, state, world| {
					spawn_explosion(
						pos - Vector3::new(0., size, 0.),
						size,
						"data/purple_explosion.cfg".into(),
						0.25,
						state,
						world,
					)
				},
			))],
		},
	))
}

pub fn spawn_orb_shard(
	pos: Point3<f32>, dir: f32, lifetime: f64, state: &mut game_state::GameState,
	world: &mut hecs::World,
) -> hecs::Entity
{
	let size = components::WeaponType::SantaGun.proj_size();
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: 256. * utils::dir_vec3(dir),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/orb_shard.cfg".into(),
		},
		components::Solid {
			size: size / 2.,
			mass: 0.,
			collision_class: components::CollisionClass::Tiny,
		},
		components::TimeToDie {
			time_to_die: state.time() + lifetime,
		},
		components::OnContactEffect {
			effects: vec![
				components::ContactEffect::Die,
				components::ContactEffect::Hurt {
					damage: components::Damage {
						amount: 6.,
						damage_type: components::DamageType::Regular,
					},
				},
			],
		},
		components::OnDeathEffect {
			effects: vec![components::DeathEffect::Spawn(Box::new(
				move |pos, _, _, state, world| {
					spawn_explosion(
						pos - Vector3::new(0., size, 0.),
						size,
						"data/green_explosion.cfg".into(),
						0.25,
						state,
						world,
					)
				},
			))],
		},
	))
}

pub fn spawn_rocket(
	pos: Point3<f32>, dir: f32, state: &mut game_state::GameState, world: &mut hecs::World,
) -> hecs::Entity
{
	let size = components::WeaponType::RocketGun.proj_size();
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: 128. * utils::dir_vec3(dir),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/rocket.cfg".into(),
		},
		components::Solid {
			size: size / 2.,
			mass: 0.,
			collision_class: components::CollisionClass::Tiny,
		},
		components::TimeToDie {
			time_to_die: state.time() + 4.,
		},
		components::OnContactEffect {
			effects: vec![components::ContactEffect::Die],
		},
		components::OnDeathEffect {
			effects: vec![
				components::DeathEffect::Spawn(Box::new(move |pos, _, _, state, world| {
					spawn_explosion(
						pos - Vector3::new(0., size, 0.),
						2. * size,
						"data/smoke.cfg".into(),
						0.25,
						state,
						world,
					)
				})),
				components::DeathEffect::DamageInRadius {
					damage: components::Damage {
						amount: 12.,
						damage_type: components::DamageType::Regular,
					},
					radius: TILE,
					push_strength: 100.,
				},
			],
		},
		components::Spawner {
			count: 0,
			max_count: -1,
			delay: 0.1,
			time_to_spawn: 0.,
			spawn_fn: Arc::new(move |pos, dir, _, state, world| {
				spawn_explosion(
					pos - Vector3::new(0., -0.22 * size, 0.) - 8. * utils::dir_vec3(dir),
					2. * 0.25 * size,
					"data/smoke.cfg".into(),
					0.25,
					state,
					world,
				)
			}),
		},
	))
}

pub fn spawn_flame(
	pos: Point3<f32>, dir: f32, state: &mut game_state::GameState, world: &mut hecs::World,
) -> hecs::Entity
{
	let size = components::WeaponType::FlameGun.proj_size() / 3.;
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: 64. * utils::dir_vec3(dir),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/flame_cloud.cfg".into(),
		},
		components::Solid {
			mass: 0.,
			size: size / 2.,
			collision_class: components::CollisionClass::Gas,
		},
		components::GasCloud {
			base_size: size,
			growth_rate: 32.,
		},
		components::CreationTime { time: state.time() },
		components::TimeToDie {
			time_to_die: state.time() + 0.75,
		},
		components::OnContactEffect {
			effects: vec![components::ContactEffect::DamageOverTime {
				damage_rate: components::Damage {
					amount: 4.,
					damage_type: components::DamageType::Flame,
				},
			}],
		},
	))
}

pub fn spawn_freeze(
	pos: Point3<f32>, dir: f32, state: &mut game_state::GameState, world: &mut hecs::World,
) -> hecs::Entity
{
	let size = components::WeaponType::FlameGun.proj_size() / 3.;
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: 64. * utils::dir_vec3(dir),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/ice_cloud.cfg".into(),
		},
		components::Solid {
			mass: 0.,
			size: size / 2.,
			collision_class: components::CollisionClass::Gas,
		},
		components::GasCloud {
			base_size: size,
			growth_rate: 32.,
		},
		components::CreationTime { time: state.time() },
		components::TimeToDie {
			time_to_die: state.time() + 0.75,
		},
		components::OnContactEffect {
			effects: vec![components::ContactEffect::DamageOverTime {
				damage_rate: components::Damage {
					amount: 2.,
					damage_type: components::DamageType::Cold,
				},
			}],
		},
	))
}

pub fn spawn_orb(
	pos: Point3<f32>, dir: f32, state: &mut game_state::GameState, world: &mut hecs::World,
) -> hecs::Entity
{
	let size = components::WeaponType::OrbGun.proj_size();
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: 128. * utils::dir_vec3(dir),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/orb.cfg".into(),
		},
		components::Solid {
			mass: 0.,
			size: size / 2.,
			collision_class: components::CollisionClass::Gas,
		},
		components::CreationTime { time: state.time() },
		components::TimeToDie {
			time_to_die: state.time() + 0.75,
		},
		components::OnDeathEffect {
			effects: vec![components::DeathEffect::Orb],
		},
	))
}

pub fn spawn_corpse(
	pos: Point3<f32>, dir: f32, vel: Vector3<f32>, size: f32, sprite_sheet: String,
	team: components::Team, world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: vel,
			dir_vel: 0.,
		},
		components::AffectedByFriction,
		components::Solid {
			size: size / 2.,
			mass: 1.,
			collision_class: components::CollisionClass::Tiny,
		},
		components::Drawable {
			size: size,
			sprite_sheet: sprite_sheet,
		},
		team,
	))
}

pub fn spawn_explosion(
	pos: Point3<f32>, size: f32, sprite_sheet: String, lifetime: f64,
	state: &mut game_state::GameState, world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		components::Position { pos: pos, dir: 0. },
		components::Drawable {
			size: size,
			sprite_sheet: sprite_sheet,
		},
		components::TimeToDie {
			time_to_die: state.time() + lifetime,
		},
		components::CreationTime { time: state.time() },
	))
}

pub fn spawn_player(
	pos: Point3<f32>, dir: f32, health: components::Health, weapon_set: components::WeaponSet,
	world: &mut hecs::World,
) -> hecs::Entity
{
	let size = TILE / 8.;
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: Vector3::zeros(),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/santa.cfg".into(),
		},
		components::Solid {
			size: size / 2.,
			mass: 1.,
			collision_class: components::CollisionClass::Regular,
		},
		weapon_set,
		health,
		components::Freezable { amount: 0. },
		components::OnDeathEffect {
			effects: vec![components::DeathEffect::Spawn(Box::new(
				move |pos, dir, vel, _, world| {
					spawn_corpse(
						pos,
						dir,
						vel,
						size,
						"data/santa_corpse.cfg".into(),
						components::Team::Player,
						world,
					)
				},
			))],
		},
		components::Team::Player,
		components::AmmoRegen {
			weapon_type: components::WeaponType::SantaGun,
			ammount: 5,
			time_to_regen: 0.,
		},
		components::Moveable {
			speed: 100.,
			rot_speed: f32::pi(),
			can_strafe: true,
		},
	))
}

pub fn spawn_buggy(
	pos: Point3<f32>, dir: f32, counter_name: &str, world: &mut hecs::World,
) -> hecs::Entity
{
	let size = 4. * TILE / 8.;

	let mut on_death_effects = vec![components::DeathEffect::Spawn(Box::new(
		move |pos, _, _, state, world| {
			spawn_explosion(pos, size, "data/smoke.cfg".into(), 0.25, state, world)
		},
	))];
	if !counter_name.is_empty()
	{
		on_death_effects.push(components::DeathEffect::IncrementCounter {
			target: counter_name.into(),
		});
	}

	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: Vector3::zeros(),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/buggy.cfg".into(),
		},
		components::Solid {
			size: size / 2.,
			mass: 5.,
			collision_class: components::CollisionClass::Regular,
		},
		components::Health {
			health: 50.,
			armour: 100.,
			max_health: 50.,
			max_armour: 50.,
		},
		components::Freezable { amount: 0. },
		components::OnDeathEffect {
			effects: on_death_effects,
		},
		components::Team::Neutral,
		components::WeaponSet {
			weapons: HashMap::from([(
				components::WeaponType::BuggyGun,
				components::Weapon::buggy_gun(),
			)]),
			cur_weapon: components::WeaponType::BuggyGun,
			want_to_fire: false,
			last_fire_time: -f64::INFINITY,
		},
		components::Vehicle { contents: None },
		components::Moveable {
			speed: 200.,
			rot_speed: f32::pi(),
			can_strafe: false,
		},
		components::AffectedByFriction,
	))
}

pub fn spawn_doodad(
	pos: Point3<f32>, dir: f32, draw_size: f32, solid_size: f32, sprite_sheet: &str,
	world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Drawable {
			size: draw_size,
			sprite_sheet: sprite_sheet.into(),
		},
		components::Solid {
			size: solid_size,
			mass: f32::INFINITY,
			collision_class: components::CollisionClass::Regular,
		},
		components::OnDeathEffect {
			effects: vec![components::DeathEffect::Spawn(Box::new(
				move |pos, _, _, state, world| {
					spawn_explosion(pos, draw_size, "data/smoke.cfg".into(), 0.25, state, world)
				},
			))],
		},
	))
}

pub fn spawn_item(
	pos: Point3<f32>, item_type: components::ItemType, counter_name: &str, world: &mut hecs::World,
) -> hecs::Entity
{
	let size = item_type.size();

	let mut on_death_effects = vec![];
	if !counter_name.is_empty()
	{
		on_death_effects.push(components::DeathEffect::IncrementCounter {
			target: counter_name.into(),
		});
	}

	world.spawn((
		components::Position { pos: pos, dir: 0. },
		components::Drawable {
			size: size,
			sprite_sheet: item_type.sprite_sheet().into(),
		},
		components::Solid {
			size: size / 2.,
			mass: 5.,
			collision_class: components::CollisionClass::Gas,
		},
		components::OnDeathEffect {
			effects: on_death_effects,
		},
		components::OnContactEffect {
			effects: vec![components::ContactEffect::Item {
				item_type: item_type,
			}],
		},
		components::Team::Neutral,
	))
}

pub fn spawn_monster(
	pos: Point3<f32>, dir: f32, counter_name: &str, world: &mut hecs::World,
) -> hecs::Entity
{
	let size = 2. * TILE / 8.;
	let mut on_death_effects = vec![components::DeathEffect::Spawn(Box::new(
		move |pos, dir, vel, _, world| {
			spawn_corpse(
				pos,
				dir,
				vel,
				size,
				"data/cat_corpse.cfg".into(),
				components::Team::Neutral,
				world,
			)
		},
	))];
	if !counter_name.is_empty()
	{
		on_death_effects.push(components::DeathEffect::IncrementCounter {
			target: counter_name.into(),
		});
	}

	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: Vector3::zeros(),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/cat.cfg".into(),
		},
		components::Solid {
			size: size / 2.,
			mass: 1.,
			collision_class: components::CollisionClass::Regular,
		},
		components::Health {
			health: 10.,
			armour: 5.,
			max_health: 10.,
			max_armour: 5.,
		},
		components::Freezable { amount: 0. },
		components::OnDeathEffect {
			effects: on_death_effects,
		},
		components::Team::Monster,
		components::WeaponSet {
			weapons: HashMap::from([(
				components::WeaponType::FlameGun,
				components::Weapon::flame_gun(),
			)]),
			cur_weapon: components::WeaponType::FlameGun,
			want_to_fire: false,
			last_fire_time: -f64::INFINITY,
		},
		components::AI {
			sense_range: TILE * 15.,
			attack_range: TILE,
			disengage_range: TILE * 16.,
			status: components::Status::Idle,
			time_to_check_status: 0.,
		},
		components::AmmoRegen {
			weapon_type: components::WeaponType::FlameGun,
			ammount: 30,
			time_to_regen: 0.,
		},
		components::Moveable {
			speed: 50.,
			rot_speed: f32::pi(),
			can_strafe: true,
		},
	))
}

pub fn spawn_spawner(
	pos: Point3<f32>, dir: f32, counter_name: &str, active: bool,
	spawn_fn: Arc<
		dyn Fn(Point3<f32>, f32, &str, &mut game_state::GameState, &mut hecs::World) -> hecs::Entity
			+ Sync
			+ Send,
	>,
	max_count: i32, delay: f32, world: &mut hecs::World,
) -> hecs::Entity
{
	let counter_name = counter_name.to_string();
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Active { active: active },
		components::Spawner {
			count: 0,
			max_count: max_count,
			delay: delay as f64,
			time_to_spawn: 0.,
			spawn_fn: Arc::new(move |pos, dir, _, state, world| {
				spawn_explosion(
					pos,
					TILE / 4.,
					"data/purple_explosion.cfg".into(),
					0.25,
					state,
					world,
				);
				spawn_fn(pos, dir, &counter_name, state, world)
			}),
		},
	))
}

pub fn spawn_area_trigger(
	start: Point2<f32>, end: Point2<f32>, targets: Vec<String>, active: bool,
	world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		components::Active { active: active },
		components::AreaTrigger {
			start: start,
			end: end,
			targets: targets,
		},
	))
}

pub fn spawn_counter(
	max_count: i32, targets: Vec<String>, active: bool, world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		components::Active { active: active },
		components::Counter {
			count: 0,
			max_count: max_count,
			targets: targets,
		},
	))
}

pub fn spawn_trigger(
	delay: f64, targets: Vec<String>, active: bool, world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		components::Active { active: active },
		components::Trigger {
			delay: delay,
			time_to_trigger: 0.,
			targets: targets,
		},
	))
}

pub fn spawn_deleter(targets: Vec<String>, active: bool, world: &mut hecs::World) -> hecs::Entity
{
	world.spawn((
		components::Active { active: active },
		components::Deleter { targets: targets },
	))
}

pub fn spawn_player_start(
	pos: Point3<f32>, dir: f32, active: bool, world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Active { active: active },
		components::PlayerStart,
	))
}

fn str_to_spawn_fn(
	name: &str,
) -> Result<
	Arc<
		dyn Fn(Point3<f32>, f32, &str, &mut game_state::GameState, &mut hecs::World) -> hecs::Entity
			+ Sync
			+ Send,
	>,
>
{
	Ok(match name
	{
		"monster" =>
		{
			Arc::new(|pos, dir, counter, _, world| spawn_monster(pos, dir, counter, world))
		}
		"buggy" => Arc::new(|pos, dir, counter, _, world| spawn_buggy(pos, dir, counter, world)),
		"suit" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::Suit, counter, world)
		}),
		"shard" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::Shard, counter, world)
		}),
		"flask" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::Flask, counter, world)
		}),
		"heart" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::Heart, counter, world)
		}),
		"bullet_ammo" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::BulletAmmo, counter, world)
		}),
		"orb_ammo" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::OrbAmmo, counter, world)
		}),
		"freeze_ammo" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::FreezeAmmo, counter, world)
		}),
		"extra_life" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::ExtraLife, counter, world)
		}),
		"freeze_gun" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::FreezeGun, counter, world)
		}),
		"orb_gun" => Arc::new(|pos, _, counter, _, world| {
			spawn_item(pos, components::ItemType::OrbGun, counter, world)
		}),
		"rock" => Arc::new(|pos, dir, _, _, world| {
			spawn_doodad(pos, dir, 0.55 * TILE, 0.55 * TILE, "data/rock.cfg", world)
		}),
		"tree" => Arc::new(|pos, dir, _, _, world| {
			spawn_doodad(pos, dir, 0.55 * TILE, 0.1 * TILE, "data/tree.cfg", world)
		}),
		"blocker" => Arc::new(|pos, dir, _, _, world| {
			spawn_doodad(
				pos,
				dir,
				0.55 * TILE,
				0.55 * TILE,
				"data/blocker.cfg",
				world,
			)
		}),
		other => return Err(format!("Unknown spawn type '{}'", other).into()),
	})
}

#[derive(Clone)]
pub struct Mesh
{
	vtxs: Vec<Vertex>,
	idxs: Vec<i32>,
}

fn load_meshes(gltf_file: &str) -> HashMap<String, Mesh>
{
	let (document, buffers, _) = gltf::import(gltf_file).unwrap();
	let mut meshes = HashMap::new();
	for node in document.nodes()
	{
		//~ dbg!(node.name());
		//let ([dx, dy, dz], [rx, ry, rz, rw], [sx, sy, sz]) = node.transform().decomposed();
		let ([_, dy, _], _, _) = node.transform().decomposed();

		if let Some(mesh) = node.mesh()
		{
			let mut vtxs = vec![];
			let mut idxs = vec![];

			for prim in mesh.primitives()
			{
				let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));
				if let (Some(pos_iter), Some(gltf::mesh::util::ReadTexCoords::F32(uv_iter))) =
					(reader.read_positions(), reader.read_tex_coords(0))
				{
					for (pos, uv) in pos_iter.zip(uv_iter)
					{
						vtxs.push(Vertex {
							x: pos[0],
							y: pos[1] + dy,
							z: pos[2],
							u: uv[0],
							v: uv[1],
							color: Color::from_rgb_f(1., 1., 1.),
						})
					}
				}

				if let Some(iter) = reader.read_indices()
				{
					for idx in iter.into_u32()
					{
						idxs.push(idx as i32)
					}
				}
			}
			//~ dbg!(dx, dy, dz);
			//~ dbg!(rx, ry, rz, rw);
			//~ dbg!(sx, sy, sz);

			let out_mesh = Mesh {
				vtxs: vtxs,
				idxs: idxs,
			};

			meshes.insert(node.name().unwrap().to_string(), out_mesh);
		}
	}
	meshes
}

#[derive(Debug, Copy, Clone)]
pub struct GridInner
{
	id: hecs::Entity,
	pos: Point3<f32>,
}

fn segment_check(
	start: Point2<f32>, end: Point2<f32>, team: components::Team, origin_id: hecs::Entity,
	margin: f32, world: &hecs::World, grid: &spatial_grid::SpatialGrid<GridInner>,
) -> bool
{
	let blockers = grid.query_segment(start, end, |entry| {
		if entry.inner.id == origin_id
		{
			false
		}
		else
		{
			if let Ok(solid) = world.get::<components::Solid>(entry.inner.id)
			{
				if solid.collision_class == components::CollisionClass::Tiny
					|| solid.collision_class == components::CollisionClass::Gas
				{
					return false;
				}
			}
			if let Ok(other_team) = world.get::<components::Team>(entry.inner.id)
			{
				team.friendly(&other_team)
			}
			else
			{
				false
			}
		}
	});

	let mut actually_blocked = false;
	for blocker in blockers
	{
		let nearest = utils::nearest_line_point(start, end, blocker.inner.pos.xz());
		if let Ok(solid) = world.get::<components::Solid>(blocker.inner.id)
		{
			let size = solid.size + margin;
			if (nearest - blocker.inner.pos.xz()).norm_squared() < size * size
			{
				actually_blocked = true;
				break;
			}
		}
	}
	actually_blocked
}

fn turn_towards(
	origin: Point2<f32>, target: Point2<f32>, cur_dir: f32, rot_speed: f32,
) -> Option<f32>
{
	let diff = (target - origin).normalize();
	let cur_dir = utils::dir_vec3(cur_dir).xz();

	let angle_diff = diff.dot(&cur_dir).acos();

	let left_normal = Vector2::new(-diff.y, diff.x);

	let cross = left_normal.dot(&cur_dir);
	if cross.abs() < 0.01 && angle_diff.abs() < 0.01
	{
		None
	}
	else if cross < 0.
	{
		Some(utils::min(angle_diff / utils::DT, rot_speed))
	}
	else
	{
		Some(-utils::min(angle_diff / utils::DT, rot_speed))
	}
}

pub struct Map
{
	projection: Perspective3<f32>,
	display_width: f32,
	display_height: f32,

	level: Level,

	player: hecs::Entity,
	lifes: i32,
	camera_anchor: components::Position,

	rot_left_state: i32,
	rot_right_state: i32,
	up_state: bool,
	down_state: bool,
	left_state: bool,
	right_state: bool,
	fire_state: bool,
	clear_rot: bool,
	enter_state: bool,
	desired_weapon: i32,
	want_spawn: bool,

	saved_health: components::Health,
	saved_weapon_set: components::WeaponSet,

	test: hecs::Entity,
	named_entities: HashMap<String, hecs::Entity>,

	world: hecs::World,
}

impl Map
{
	pub fn new(
		state: &mut game_state::GameState, display_width: f32, display_height: f32,
	) -> Result<Self>
	{
		let mut world = hecs::World::default();
		let mut named_entities = HashMap::new();

		let level = Level::new("data/level.cfg", &mut named_entities, state, &mut world)?;

		let mut camera_anchor = components::Position {
			pos: Point3::new(0., 0., 0.),
			dir: 0.,
		};
		let mut player_start_entity = None;
		for (id, (active, pos, _)) in world.query_mut::<(
			&components::Active,
			&components::Position,
			&components::PlayerStart,
		)>()
		{
			if active.active
			{
				camera_anchor = *pos;
				player_start_entity = Some(id);
				break;
			}
		}
		if player_start_entity.is_none()
		{
			return Err("Map has no start".to_string().into());
		}
		let player_start = player_start_entity.unwrap();
		let test = player_start;

		state.cache_sprite_sheet("data/terrain.cfg")?;
		state.cache_sprite_sheet("data/rock.cfg")?;
		state.cache_sprite_sheet("data/tree.cfg")?;
		state.cache_sprite_sheet("data/blocker.cfg")?;
		state.cache_sprite_sheet("data/ice_cloud.cfg")?;
		state.cache_sprite_sheet("data/orb.cfg")?;
		state.cache_sprite_sheet("data/flame_cloud.cfg")?;
		state.cache_sprite_sheet("data/purple_explosion.cfg")?;
		state.cache_sprite_sheet("data/green_explosion.cfg")?;
		state.cache_sprite_sheet("data/buggy.cfg")?;
		state.cache_sprite_sheet("data/cat.cfg")?;
		state.cache_sprite_sheet("data/cat_corpse.cfg")?;
		state.cache_sprite_sheet("data/santa.cfg")?;
		state.cache_sprite_sheet("data/santa_corpse.cfg")?;
		state.cache_sprite_sheet("data/bullet.cfg")?;
		state.cache_sprite_sheet("data/armor_shard.cfg")?;
		state.cache_sprite_sheet("data/armor_suit.cfg")?;
		state.cache_sprite_sheet("data/flask.cfg")?;
		state.cache_sprite_sheet("data/heart.cfg")?;
		state.cache_sprite_sheet("data/bullet_ammo.cfg")?;
		state.cache_sprite_sheet("data/freeze_ammo.cfg")?;
		state.cache_sprite_sheet("data/star_ammo.cfg")?;
		state.cache_sprite_sheet("data/extra_life.cfg")?;
		state.cache_sprite_sheet("data/orb_shard.cfg")?;
		state.cache_sprite_sheet("data/rocket.cfg")?;
		state.cache_sprite_sheet("data/freeze_gun.cfg")?;
		state.cache_sprite_sheet("data/orb_gun.cfg")?;
		state.cache_sprite_sheet("data/test.cfg")?;
		state.cache_sprite_sheet("data/smoke.cfg")?;
		//~ state.atlas.dump_pages();

		Ok(Self {
			test: test,
			projection: utils::projection_transform(display_width, display_height),
			display_width: display_width,
			display_height: display_height,
			level: level,
			player: player_start,
			camera_anchor: camera_anchor,
			world: world,
			rot_left_state: 0,
			rot_right_state: 0,
			up_state: false,
			down_state: false,
			left_state: false,
			right_state: false,
			fire_state: false,
			clear_rot: false,
			enter_state: false,
			desired_weapon: 0,
			want_spawn: true,
			saved_health: components::Health {
				health: 100.,
				armour: 0.,
				max_health: 100.,
				max_armour: 100.,
			},
			saved_weapon_set: components::WeaponSet {
				weapons: HashMap::from([
					(
						components::WeaponType::OrbGun,
						components::Weapon::orb_gun(),
					),
					(
						components::WeaponType::SantaGun,
						components::Weapon::santa_gun(),
					),
					(
						components::WeaponType::RocketGun,
						components::Weapon::rocket_gun(),
					),
					(
						components::WeaponType::FlameGun,
						components::Weapon::flame_gun(),
					),
					(
						components::WeaponType::FreezeGun,
						components::Weapon::freeze_gun(),
					),
				]),
				cur_weapon: components::WeaponType::SantaGun,
				want_to_fire: false,
				last_fire_time: -f64::INFINITY,
			},
			named_entities: named_entities,
			lifes: 3,
		})
	}

	fn make_camera(&self) -> Isometry3<f32>
	{
		let rot = Rotation2::new(self.camera_anchor.dir);
		let offt = rot * Vector2::new(0., -TILE / 2.);
		let height = TILE / 2.2;

		utils::camera_project(
			self.camera_anchor.pos.x + offt.x,
			height,
			self.camera_anchor.pos.z + offt.y,
			self.camera_anchor.pos.x,
			self.camera_anchor.pos.z,
		)
	}

	pub fn logic(&mut self, state: &mut game_state::GameState) -> Result<()>
	{
		let mut to_die = vec![];

		// Collision detection.
		let mut grid = spatial_grid::SpatialGrid::new(
			self.level.width as usize,
			self.level.height as usize,
			TILE,
			TILE,
		);

		for (id, (pos, solid)) in self
			.world
			.query_mut::<(&components::Position, &components::Solid)>()
		{
			let margin = 8.;
			let r = solid.size + margin;
			let x = pos.pos.x;
			let z = pos.pos.z;
			grid.push(spatial_grid::entry(
				Point2::new(x - r, z - r),
				Point2::new(x + r, z + r),
				GridInner {
					pos: pos.pos,
					id: id,
				},
			));
		}

		let mut colliding_pairs = vec![];
		for (a, b) in grid.all_pairs(|a, b| {
			let a_solid = self.world.get::<components::Solid>(a.inner.id).unwrap();
			let b_solid = self.world.get::<components::Solid>(b.inner.id).unwrap();
			a_solid
				.collision_class
				.collides_with(b_solid.collision_class)
		})
		{
			colliding_pairs.push((a.inner, b.inner));
		}

		let mut on_contact_effects = vec![];
		for pass in 0..5
		{
			for &(inner1, inner2) in &colliding_pairs
			{
				let id1 = inner1.id;
				let id2 = inner2.id;
				let pos1 = self.world.get::<components::Position>(id1)?.pos;
				let pos2 = self.world.get::<components::Position>(id2)?.pos;

				let solid1 = *self.world.get::<components::Solid>(id1)?;
				let solid2 = *self.world.get::<components::Solid>(id2)?;

				let diff = pos2.xz() - pos1.xz();
				let diff_norm = utils::max(0.1, diff.norm());

				if diff_norm > solid1.size + solid2.size
				{
					continue;
				}

				if solid1.collision_class.interacts() && solid2.collision_class.interacts()
				{
					let diff = 0.9 * diff * (solid1.size + solid2.size - diff_norm) / diff_norm;
					let diff = Vector3::new(diff.x, 0., diff.y);

					let f1 = 1. - solid1.mass / (solid2.mass + solid1.mass);
					let f2 = 1. - solid2.mass / (solid2.mass + solid1.mass);
					if f32::is_finite(f1)
					{
						self.world.get_mut::<components::Position>(id1)?.pos -= diff * f1;
					}
					if f32::is_finite(f2)
					{
						self.world.get_mut::<components::Position>(id2)?.pos += diff * f2;
					}
				}

				if pass == 0
				{
					for (id, other_id) in [(id1, Some(id2)), (id2, Some(id1))]
					{
						if let Ok(on_contact_effect) =
							self.world.get::<components::OnContactEffect>(id)
						{
							on_contact_effects.push((
								id,
								other_id,
								on_contact_effect.effects.clone(),
							));
						}
					}
				}
			}

			for (id, (pos, solid)) in self
				.world
				.query::<(&mut components::Position, &components::Solid)>()
				.iter()
			{
				if let Some(resolve_diff) = self.level.check_collision(pos.pos, solid.size)
				{
					pos.pos += 0.9 * resolve_diff;

					if pass == 0
					{
						if let Ok(on_contact_effect) =
							self.world.get::<components::OnContactEffect>(id)
						{
							on_contact_effects.push((id, None, on_contact_effect.effects.clone()));
						}
					}
				}
			}
		}

		// On contact effects.
		for (id, other_id, effects) in on_contact_effects
		{
			for effect in effects
			{
				match (effect, other_id)
				{
					(components::ContactEffect::Die, _) => to_die.push((true, id)),
					(components::ContactEffect::Hurt { damage }, Some(other_id)) =>
					{
						if let Ok(mut health) = self.world.get_mut::<components::Health>(other_id)
						{
							health.damage(damage, 1.);
						}
						if damage.damage_type == components::DamageType::Cold
						{
							if let Ok(mut freezable) =
								self.world.get_mut::<components::Freezable>(other_id)
							{
								freezable.amount = utils::min(
									2.,
									freezable.amount + FREEZE_FACTOR * damage.amount,
								);
							}
						}
					}
					(components::ContactEffect::DamageOverTime { damage_rate }, Some(other_id)) =>
					{
						if let Ok(mut health) = self.world.get_mut::<components::Health>(other_id)
						{
							health.damage(damage_rate, utils::DT);
						}
						if damage_rate.damage_type == components::DamageType::Cold
						{
							if let Ok(mut freezable) =
								self.world.get_mut::<components::Freezable>(other_id)
							{
								freezable.amount = utils::min(
									2.,
									freezable.amount
										+ FREEZE_FACTOR * damage_rate.amount * utils::DT,
								);
							}
						}
					}
					(components::ContactEffect::Item { item_type }, Some(other_id)) =>
					{
						let team = self.world.get::<components::Team>(other_id);
						let vehicle = self.world.get::<components::Vehicle>(other_id);
						let health = self.world.get_mut::<components::Health>(other_id);
						let mut weapon_set = self.world.get_mut::<components::WeaponSet>(other_id);

						let mut new_weapon = None;
						if team.map(|t| *t) == Ok(components::Team::Player) && vehicle.is_err()
						{
							let picked_up = match item_type
							{
								components::ItemType::Shard =>
								{
									health.map(|mut h| h.add_armour(5.)).unwrap_or(false)
								}
								components::ItemType::Suit =>
								{
									health.map(|mut h| h.add_armour(50.)).unwrap_or(false)
								}
								components::ItemType::Flask =>
								{
									health.map(|mut h| h.add_health(5.)).unwrap_or(false)
								}
								components::ItemType::Heart =>
								{
									health.map(|mut h| h.add_health(50.)).unwrap_or(false)
								}
								components::ItemType::ExtraLife =>
								{
									self.lifes += 1;
									true
								}
								components::ItemType::BulletAmmo => weapon_set
									.as_mut()
									.map(|w| {
										w.weapons
											.get_mut(&components::WeaponType::SantaGun)
											.map(|w| w.add_ammo(10))
											.unwrap_or(false)
									})
									.unwrap_or(false),
								components::ItemType::OrbAmmo => weapon_set
									.as_mut()
									.map(|w| {
										w.weapons
											.get_mut(&components::WeaponType::OrbGun)
											.map(|w| w.add_ammo(10))
											.unwrap_or(false)
									})
									.unwrap_or(false),
								components::ItemType::FreezeAmmo => weapon_set
									.as_mut()
									.map(|w| {
										w.weapons
											.get_mut(&components::WeaponType::FreezeGun)
											.map(|w| w.add_ammo(10))
											.unwrap_or(false)
									})
									.unwrap_or(false),
								components::ItemType::FreezeGun => weapon_set
									.as_mut()
									.map(|w| {
										w.weapons
											.get_mut(&components::WeaponType::FreezeGun)
											.map(|w| {
												let old_selectable = w.selectable;
												if !old_selectable
												{
													new_weapon =
														Some(components::WeaponType::FreezeGun);
												}
												w.selectable = true;
												w.add_ammo(10) || !old_selectable
											})
											.unwrap_or(false)
									})
									.unwrap_or(false),
								components::ItemType::OrbGun => weapon_set
									.as_mut()
									.map(|w| {
										w.weapons
											.get_mut(&components::WeaponType::OrbGun)
											.map(|w| {
												let old_selectable = w.selectable;
												if !old_selectable
												{
													new_weapon =
														Some(components::WeaponType::OrbGun);
												}
												w.selectable = true;
												w.add_ammo(10) || !old_selectable
											})
											.unwrap_or(false)
									})
									.unwrap_or(false),
							};
							if let Some(new_weapon) = new_weapon
							{
								weapon_set.map(|mut w| w.cur_weapon = new_weapon).ok();
							}
							if picked_up
							{
								to_die.push((true, id));
							}
						}
					}
					_ => (),
				}
			}
		}

		// Player controller.
		if self.world.contains(self.player)
			&& self.world.get::<components::Team>(self.player).is_ok()
			&& self
				.world
				.get::<components::Health>(self.player)
				.map(|h| h.health > 0.)
				.unwrap_or(false)
			&& self
				.world
				.get::<components::Freezable>(self.player)
				.map(|f| !f.is_frozen())
				.unwrap_or(true)
		{
			let moveable = *self.world.get::<components::Moveable>(self.player)?;
			let rot_left_right = self.rot_right_state - (self.rot_left_state as i32);
			let left_right = if moveable.can_strafe
			{
				self.left_state as i32 - (self.right_state as i32)
			}
			else
			{
				0
			};
			let up_down = self.up_state as i32 - (self.down_state as i32);

			let pos = *self.world.get::<components::Position>(self.player)?;
			let dir = pos.dir;
			let rot = Rotation2::new(dir);
			let speed = moveable.speed;
			let vel = rot * Vector2::new(left_right as f32 * speed, up_down as f32 * speed);

			{
				let mut player_vel = self.world.get_mut::<components::Velocity>(self.player)?;
				player_vel.vel = Vector3::new(vel.x, 0., vel.y);
				player_vel.dir_vel = rot_left_right as f32 * f32::pi() / 2.;
			}

			if self.enter_state
			{
				let mut spawn_fn = None;
				if let Ok(mut vehicle) = self.world.get_mut::<components::Vehicle>(self.player)
				{
					spawn_fn = vehicle.contents.take();
					self.enter_state = false;

					let mut player_vel = self.world.get_mut::<components::Velocity>(self.player)?;
					player_vel.vel = Vector3::zeros();
					player_vel.dir_vel = 0.;

					*self.world.get_mut::<components::Team>(self.player)? =
						components::Team::Neutral;
				}
				else
				{
					let entries = grid.query_rect(
						Point2::new(pos.pos.x - 2. * TILE, pos.pos.z - 2. * TILE),
						Point2::new(pos.pos.x + 2. * TILE, pos.pos.z + 2. * TILE),
						|entry| {
							if let (Ok(other_team), Ok(vehicle)) = (
								self.world.get::<components::Team>(entry.inner.id),
								self.world.get::<components::Vehicle>(entry.inner.id),
							)
							{
								components::Team::Player.friendly(&other_team)
									&& vehicle.contents.is_none()
							}
							else
							{
								false
							}
						},
					);
					if let Some(entry) = entries.get(0)
					{
						let mut vehicle = self
							.world
							.get_mut::<components::Vehicle>(entry.inner.id)
							.unwrap();

						let health = (*self.world.get::<components::Health>(self.player)?).clone();
						let weapon_set =
							(*self.world.get::<components::WeaponSet>(self.player)?).clone();

						vehicle.contents = Some(Box::new(move |pos, dir, world| {
							spawn_player(pos, dir, health.clone(), weapon_set.clone(), world)
						}));

						to_die.push((false, self.player));
						*self
							.world
							.get_mut::<components::Team>(entry.inner.id)
							.unwrap() = components::Team::Player;

						self.player = entry.inner.id;
						self.enter_state = false;
					}
				}
				if let Some(spawn_fn) = spawn_fn
				{
					self.player = spawn_fn(
						pos.pos - 0.5 * TILE * utils::dir_vec3(dir),
						dir,
						&mut self.world,
					);
				}
			}

			if let Ok(mut weapon_set) = self.world.get_mut::<components::WeaponSet>(self.player)
			{
				if self.desired_weapon != 0
				{
					let new_weapon = match self.desired_weapon
					{
						1 => Some(components::WeaponType::SantaGun),
						2 => Some(components::WeaponType::FreezeGun),
						3 => Some(components::WeaponType::OrbGun),
						_ => None,
					};
					if let Some(new_weapon) = new_weapon
					{
						if weapon_set.weapons.contains_key(&new_weapon)
						{
							if weapon_set.weapons[&new_weapon].selectable
							{
								weapon_set.cur_weapon = new_weapon;
							}
						}
					}
					self.desired_weapon = 0;
				}
				weapon_set.want_to_fire = self.fire_state;
			}
		}

		// Weapon handling.
		let mut proj_spawns = vec![];
		for (_, (pos, weapon_set, solid)) in self.world.query_mut::<(
			&components::Position,
			&mut components::WeaponSet,
			&components::Solid,
		)>()
		{
			if !weapon_set.want_to_fire || weapon_set.weapons.is_empty()
			{
				continue;
			}

			let weapon = weapon_set.weapons.get_mut(&weapon_set.cur_weapon).unwrap();

			if weapon.time_to_fire > state.time() || weapon.ammo == 0
			{
				continue;
			}

			weapon.time_to_fire = state.time() + weapon.delay;
			weapon.ammo -= weapon.weapon_type.ammo_usage();

			let proj_size = weapon.weapon_type.proj_size();
			match weapon.weapon_type
			{
				components::WeaponType::SantaGun
				| components::WeaponType::FlameGun
				| components::WeaponType::RocketGun
				| components::WeaponType::OrbGun
				| components::WeaponType::FreezeGun =>
				{
					let spawn_pos =
						pos.pos + utils::dir_vec3(pos.dir) * (solid.size + proj_size + 1.);
					proj_spawns.push((spawn_pos, pos.dir, weapon.weapon_type));
				}
				components::WeaponType::BuggyGun =>
				{
					let forward = solid.size + proj_size + 1.;
					let dir = utils::dir_vec3(pos.dir);
					let left = 10. * Vector3::new(-dir.z, 0., dir.x);
					let spawn_pos = pos.pos + forward * dir + left;
					proj_spawns.push((spawn_pos, pos.dir, weapon.weapon_type));
					let spawn_pos = pos.pos + forward * dir - left;
					proj_spawns.push((spawn_pos, pos.dir, weapon.weapon_type));
				}
			}
			weapon_set.last_fire_time = state.time();
		}

		for (pos, dir, weapon_type) in proj_spawns
		{
			match weapon_type
			{
				components::WeaponType::SantaGun | components::WeaponType::BuggyGun =>
				{
					spawn_projectile(
						pos + Vector3::new(0., 8., 0.),
						dir,
						1.,
						state,
						&mut self.world,
					);
				}
				components::WeaponType::RocketGun =>
				{
					spawn_rocket(pos + Vector3::new(0., 8., 0.), dir, state, &mut self.world);
				}
				components::WeaponType::FlameGun =>
				{
					spawn_flame(pos + Vector3::new(0., 8., 0.), dir, state, &mut self.world);
				}
				components::WeaponType::FreezeGun =>
				{
					spawn_freeze(pos + Vector3::new(0., 8., 0.), dir, state, &mut self.world);
				}
				components::WeaponType::OrbGun =>
				{
					spawn_orb(pos + Vector3::new(0., 8., 0.), dir, state, &mut self.world);
				}
			}
		}

		// Ammo regen
		for (_, (ammo_regen, weapon_set)) in self
			.world
			.query_mut::<(&mut components::AmmoRegen, &mut components::WeaponSet)>()
		{
			if state.time() > ammo_regen.time_to_regen
			{
				if let Some(mut weapon) = weapon_set.weapons.get_mut(&ammo_regen.weapon_type)
				{
					weapon.ammo = utils::min(weapon.ammo + ammo_regen.ammount, weapon.max_ammo);
				}
				ammo_regen.time_to_regen = state.time() + 5.;
			}
		}

		// Gas cloud.
		for (_, (gas_cloud, creation_time, solid, drawable)) in self.world.query_mut::<(
			&components::GasCloud,
			&components::CreationTime,
			&mut components::Solid,
			&mut components::Drawable,
		)>()
		{
			let size = gas_cloud.base_size
				+ gas_cloud.growth_rate * (state.time() - creation_time.time) as f32;
			solid.size = size / 2.;
			drawable.size = size / 2.;
		}

		// Freezable
		for (id, freezable) in self.world.query::<&mut components::Freezable>().iter()
		{
			freezable.amount = utils::max(0., freezable.amount - 0.25 * utils::DT);
			if freezable.is_frozen()
			{
				if let Ok(mut weapon_set) = self.world.get_mut::<components::WeaponSet>(id)
				{
					weapon_set.want_to_fire = false;
				}
			}
		}

		// Friction.
		for (_, (vel, _)) in self
			.world
			.query_mut::<(&mut components::Velocity, &components::AffectedByFriction)>()
		{
			vel.vel *= 0.25_f32.powf(utils::DT as f32);
			vel.dir_vel *= 0.25_f32.powf(utils::DT as f32);
		}

		// Velocity handling.
		for (id, (pos, vel)) in self
			.world
			.query_mut::<(&mut components::Position, &components::Velocity)>()
		{
			if id == self.test
			{
				//~ dbg!(pos.clone());
			}
			pos.pos += utils::DT * vel.vel;
			pos.dir += utils::DT * vel.dir_vel;
			//pos.dir = pos.dir.fmod(2. * f32::pi());
		}

		// AI
		for (id, (pos, moveable, vel, team, health, ai)) in self
			.world
			.query::<(
				&components::Position,
				&components::Moveable,
				&mut components::Velocity,
				&components::Team,
				&components::Health,
				&mut components::AI,
			)>()
			.iter()
		{
			// HACK for explosion pushback...
			if health.health < 0.
			{
				continue;
			}
			if let Ok(freezable) = self.world.get::<components::Freezable>(id)
			{
				if freezable.is_frozen()
				{
					continue;
				}
			}
			//~ if id == self.test
			//~ {
			//~ println!("s---- {} {:?}", state.time(), ai.status);
			//~ }
			if ai.time_to_check_status < state.time()
			{
				let rot_speed = moveable.rot_speed;
				let speed = moveable.speed;
				let mut new_dir_vel = None;
				let mut new_vel = None;
				let mut do_attack = false;

				match ai.status
				{
					components::Status::Idle =>
					{
						let entries = grid.query_rect(
							Point2::new(pos.pos.x - ai.sense_range, pos.pos.z - ai.sense_range),
							Point2::new(pos.pos.x + ai.sense_range, pos.pos.z + ai.sense_range),
							|entry| {
								if let Ok(other_team) =
									self.world.get::<components::Team>(entry.inner.id)
								{
									if team.friendly(&other_team)
									{
										false
									}
									else if (entry.inner.pos.xz() - pos.pos.xz()).norm()
										< ai.sense_range
									{
										true
									}
									else
									{
										false
									}
								}
								else
								{
									false
								}
							},
						);

						let mut least_dist = f32::INFINITY;
						let mut best_id = None;
						for entry in entries
						{
							let new_dist = (entry.inner.pos - pos.pos).norm_squared();
							if new_dist < least_dist
							{
								best_id = Some(entry.inner.id);
								least_dist = new_dist;
							}
						}
						if let Some(id) = best_id
						{
							ai.status = components::Status::Attacking(id);
							break;
						}
					}
					components::Status::Attacking(target) =>
					{
						if !self.world.contains(target)
						{
							ai.status = components::Status::Idle;
						}
						else if self
							.world
							.get::<components::Team>(target)
							.map(|other_team| team.friendly(&other_team))
							.unwrap_or(true)
						{
							ai.status = components::Status::Idle;
						}
						else
						{
							let target_pos = self.world.get::<components::Position>(target)?;
							let dist = (target_pos.pos.xz() - pos.pos.xz()).norm();

							new_dir_vel =
								turn_towards(pos.pos.xz(), target_pos.pos.xz(), pos.dir, rot_speed);
							new_vel = Some(utils::dir_vec3(pos.dir) * speed);

							if dist > ai.disengage_range
							{
								ai.status = components::Status::Idle;
							}
							else if dist < ai.attack_range
							{
								let blocked = segment_check(
									pos.pos.xz(),
									target_pos.pos.xz(),
									*team,
									id,
									8.,
									&self.world,
									&grid,
								);
								let map_blocked =
									self.level
										.check_segment(pos.pos.xz(), target_pos.pos.xz(), 8.);
								if blocked
								{
									let mut rng = rand::thread_rng();
									let offset = Vector3::new(
										TILE * (2. * rng.gen::<f32>() - 1.),
										0.,
										TILE * (2. * rng.gen::<f32>() - 1.),
									);
									ai.status = components::Status::Searching(
										target,
										pos.pos + offset,
										state.time() + 2.,
									);
								}
								else if new_dir_vel.is_none() && !map_blocked
								{
									new_vel = None;
									do_attack = true;
								}
							}
						}
					}
					components::Status::Searching(target, search_target, time_to_stop) =>
					{
						let dist = (search_target.xz() - pos.pos.xz()).norm();
						new_dir_vel =
							turn_towards(pos.pos.xz(), search_target.xz(), pos.dir, rot_speed);
						new_vel = Some(utils::dir_vec3(pos.dir) * speed);

						if dist < speed * utils::DT || state.time() > time_to_stop
						{
							//~ println!("Switch to attacking timeout");
							ai.status = components::Status::Attacking(target);
						}
						else
						{
							if !self.world.contains(target)
							{
								ai.status = components::Status::Idle;
							}
							else
							{
								let target_pos = self.world.get::<components::Position>(target)?;
								let blocked = segment_check(
									pos.pos.xz(),
									target_pos.pos.xz(),
									*team,
									id,
									8.,
									&self.world,
									&grid,
								);
								let map_blocked =
									self.level
										.check_segment(pos.pos.xz(), target_pos.pos.xz(), 8.);
								if !blocked && !map_blocked
								{
									//~ println!("Switch to attacking not blocked");
									ai.status = components::Status::Attacking(target);
								}
							}
						}
					}
				}

				vel.dir_vel = new_dir_vel.unwrap_or(0.);
				if new_dir_vel.is_none()
				{
					vel.vel = new_vel.unwrap_or(Vector3::zeros());
				}
				if let Ok(mut weapon_set) = self.world.get_mut::<components::WeaponSet>(id)
				{
					weapon_set.want_to_fire = do_attack;
				}

				//~ if id == self.test
				//~ {
				//~ println!("e---- {}", state.time());
				//~ }
				//~ ai.time_to_check_status = state.time() + 1.;
			}
		}

		// Health
		let mut spawn_fns: Vec<(
			bool,
			Box<dyn FnOnce(&mut game_state::GameState, &mut hecs::World) -> hecs::Entity>,
		)> = vec![];
		for (id, health) in self.world.query::<&components::Health>().iter()
		{
			if health.health < 0.
			{
				to_die.push((true, id));
			}
		}

		// Spawner
		for (id, (pos, spawner)) in self
			.world
			.query::<(&components::Position, &mut components::Spawner)>()
			.iter()
		{
			if self
				.world
				.get::<components::Active>(id)
				.map(|a| a.active)
				.unwrap_or(true)
			{
				if state.time() > spawner.time_to_spawn
					&& (spawner.count < spawner.max_count || spawner.max_count == -1)
				{
					spawner.time_to_spawn = state.time() + spawner.delay;
					let point_pos = pos.pos.clone();
					let dir = pos.dir;
					let spawn_fn = spawner.spawn_fn.clone();
					spawner.count += 1;
					spawn_fns.push((
						false,
						Box::new(move |state, world| {
							spawn_fn(point_pos, dir, Vector3::zeros(), state, world)
						}),
					));
				}
			}
		}

		// Player start
		if self.want_spawn
		{
			if let Ok(mut team) = self.world.get_mut::<components::Team>(self.player)
			{
				// To get enemies to stop attacking corpses.
				*team = components::Team::Neutral;
			}

			for (_, (active, pos, _)) in self.world.query_mut::<(
				&components::Active,
				&components::Position,
				&components::PlayerStart,
			)>()
			{
				if active.active
				{
					let point_pos = pos.pos.clone();
					let health = self.saved_health.clone();
					let weapon_set = self.saved_weapon_set.clone();
					let dir = pos.dir;
					spawn_fns.push((
						true,
						Box::new(move |_, world| {
							spawn_player(point_pos, dir, health, weapon_set, world)
						}),
					));
				}
			}
			self.want_spawn = false;
		}

		// Area trigger
		let mut activate = vec![];
		for (id, area_trigger) in self.world.query::<&components::AreaTrigger>().iter()
		{
			if self.world.get::<components::Active>(id).map(|a| a.active)?
			{
				let entries = grid.query_rect(area_trigger.start, area_trigger.end, |entry| {
					if let Ok(team) = self.world.get::<components::Team>(entry.inner.id)
					{
						*team == components::Team::Player
					}
					else
					{
						false
					}
				});
				if !entries.is_empty()
				{
					for target in &area_trigger.targets
					{
						if let Some(&entity) = self.named_entities.get(target)
						{
							if self.world.contains(entity)
							{
								activate.push(entity);
							}
						}
					}
					to_die.push((true, id));
				}
			}
		}

		// Counter
		for (id, counter) in self.world.query::<&components::Counter>().iter()
		{
			if self.world.get::<components::Active>(id).map(|a| a.active)?
			{
				if counter.count >= counter.max_count
				{
					for target in &counter.targets
					{
						if let Some(&entity) = self.named_entities.get(target)
						{
							if self.world.contains(entity)
							{
								activate.push(entity);
							}
						}
					}
					to_die.push((true, id));
				}
			}
		}

		// Trigger
		for (id, trigger) in self.world.query::<&components::Trigger>().iter()
		{
			if self.world.get::<components::Active>(id).map(|a| a.active)?
			{
				if state.time() > trigger.time_to_trigger
				{
					for target in &trigger.targets
					{
						if let Some(&entity) = self.named_entities.get(target)
						{
							if self.world.contains(entity)
							{
								activate.push(entity);
							}
						}
					}
					to_die.push((true, id));
				}
			}
		}

		let mut save = false;
		for entity in activate
		{
			if let Ok(mut active) = self.world.get_mut::<components::Active>(entity)
			{
				active.active = !active.active;

				if active.active
				{
					if self.world.get::<components::PlayerStart>(entity).is_ok()
					{
						save = true;
					}
					if let Ok(mut trigger) = self.world.get_mut::<components::Trigger>(entity)
					{
						trigger.time_to_trigger = state.time() + trigger.delay;
					}
				}
			}
		}

		if save
		{
			if let Ok(health) = self.world.get::<components::Health>(self.player)
			{
				self.saved_health = (*health).clone();
			}
			if let Ok(weapon_set) = self.world.get::<components::WeaponSet>(self.player)
			{
				self.saved_weapon_set = (*weapon_set).clone();
			}
		}

		// Deleter
		for (id, deleter) in self.world.query::<&components::Deleter>().iter()
		{
			if self
				.world
				.get::<components::Active>(id)
				.map(|a| a.active)
				.unwrap_or(true)
			{
				for target in &deleter.targets
				{
					if let Some(&entity) = self.named_entities.get(target)
					{
						if self.world.contains(entity)
						{
							to_die.push((true, entity));
						}
					}
				}
				to_die.push((true, id));
			}
		}

		// Time to die
		for (id, time_to_die) in self.world.query_mut::<&components::TimeToDie>()
		{
			if state.time() > time_to_die.time_to_die
			{
				to_die.push((true, id));
			}
		}

		// On-death effects
		for &(trigger_on_death, id) in &to_die
		{
			if !trigger_on_death
			{
				continue;
			}
			if let Ok(pos) = self.world.get::<components::Position>(id)
			{
				let point_pos = pos.pos.clone();
				let dir = pos.dir;
				let point_vel = self
					.world
					.get::<components::Velocity>(id)
					.map(|v| v.vel.clone())
					.unwrap_or(Vector3::zeros());

				if let Ok(mut on_death_effect) = self.world.get_mut::<components::OnDeathEffect>(id)
				{
					for effect in on_death_effect.effects.drain(..)
					{
						match effect
						{
							components::DeathEffect::Spawn(spawn_fn) =>
							{
								spawn_fns.push((
									id == self.player,
									Box::new(move |state, world| {
										spawn_fn(point_pos, dir, point_vel, state, world)
									}),
								));
							}
							components::DeathEffect::DamageInRadius {
								damage,
								radius,
								push_strength,
							} =>
							{
								let entries = grid.query_rect(
									Point2::new(pos.pos.x - radius, pos.pos.z - radius),
									Point2::new(pos.pos.x + radius, pos.pos.z + radius),
									|entry| {
										if entry.inner.id == id
										{
											return false;
										}
										if let (Ok(_), Ok(other_pos)) = (
											self.world.get::<components::Health>(entry.inner.id),
											self.world.get::<components::Position>(entry.inner.id),
										)
										{
											(other_pos.pos.xz() - pos.pos.xz()).norm() < radius
										}
										else
										{
											false
										}
									},
								);

								for entry in entries
								{
									let mut health =
										self.world.get_mut::<components::Health>(entry.inner.id)?;
									health.health -= damage.amount;

									if damage.damage_type == components::DamageType::Cold
									{
										if let Ok(mut freezable) = self
											.world
											.get_mut::<components::Freezable>(entry.inner.id)
										{
											freezable.amount = utils::min(
												2.,
												freezable.amount + FREEZE_FACTOR * damage.amount,
											);
										}
									}

									if let (Ok(other_pos), Ok(solid), Ok(mut other_vel)) = (
										self.world.get::<components::Position>(entry.inner.id),
										self.world.get::<components::Solid>(entry.inner.id),
										self.world.get_mut::<components::Velocity>(entry.inner.id),
									)
									{
										let dir = (other_pos.pos.xz() - pos.pos.xz()).normalize();
										other_vel.vel += Vector3::new(dir.x, 0., dir.y)
											* push_strength / solid.mass;
									}
								}
							}
							components::DeathEffect::Orb =>
							{
								let n = 13;
								for i in 0..n
								{
									spawn_fns.push((
										false,
										Box::new(move |state, world| {
											spawn_orb_shard(
												point_pos,
												dir + (i as f32 / n as f32 + 1. / n as f32)
													* 2. * f32::pi(),
												0.25,
												state,
												world,
											)
										}),
									));
								}
							}
							components::DeathEffect::IncrementCounter { target } =>
							{
								if let Some(&entity) = self.named_entities.get(&target)
								{
									if self.world.contains(entity)
									{
										if let Ok(mut counter) =
											self.world.get_mut::<components::Counter>(entity)
										{
											counter.count += 1;
										}
									}
								}
							}
						}
					}
				}

				if let Ok(mut vehicle) = self.world.get_mut::<components::Vehicle>(id)
				{
					if let Some(spawn_fn) = vehicle.contents.take()
					{
						spawn_fns.push((
							id == self.player,
							Box::new(move |_, world| spawn_fn(point_pos, dir, world)),
						));
					}
				}
			}
		}

		for (new_player, spawn_fn) in spawn_fns
		{
			let entity = spawn_fn(state, &mut self.world);
			if new_player
			{
				self.player = entity;
			}
		}

		// Update camera anchor.
		if let Ok(player_pos) = self.world.get::<components::Position>(self.player)
		{
			self.camera_anchor = *player_pos;
		}

		to_die.sort();
		to_die.dedup();

		// Remove dead entities
		for (_, id) in to_die
		{
			dbg!("died", id);
			self.world.despawn(id)?;
		}

		// HACK.
		if self.clear_rot
		{
			self.rot_left_state = 0;
			self.rot_right_state = 0;
			self.clear_rot = false;
		}

		Ok(())
	}

	pub fn draw(&mut self, state: &game_state::GameState) -> Result<()>
	{
		state.core.set_depth_test(Some(DepthFunction::Less));
		state
			.core
			.use_projection_transform(&utils::mat4_to_transform(self.projection.into_inner()));
		unsafe {
			al_set_render_state(ALLEGRO_ALPHA_TEST_RS, 1);
			al_set_render_state(ALLEGRO_ALPHA_TEST_VALUE, 128);
			al_set_render_state(ALLEGRO_ALPHA_FUNCTION, ALLEGRO_RENDER_GREATER as i32);
		}

		let camera = self.make_camera();

		state
			.core
			.use_transform(&utils::mat4_to_transform(camera.to_homogeneous()));
		state
			.core
			.set_blender(BlendOperation::Add, BlendMode::One, BlendMode::InverseAlpha);

		let mut scene = Scene::new();

		self.level.draw(state, &mut scene);

		for (id, (pos, drawable)) in self
			.world
			.query::<(&components::Position, &components::Drawable)>()
			.iter()
		{
			let time_offset = self
				.world
				.get::<components::CreationTime>(id)
				.map(|v| v.time)
				.unwrap_or(0.);

			let mut color = Color::from_rgb_f(1., 1., 1.);
			let mut f = 1.;
			if let Ok(freezable) = self.world.get::<components::Freezable>(id)
			{
				if freezable.is_frozen()
				{
					color = Color::from_rgb_f(0.5, 0.5, 1.);
					f = 0.;
				}
			}

			let sheet = state.get_sprite_sheet(&drawable.sprite_sheet).unwrap();
			let bmp = &sheet
				.get_bitmap(
					f * (state.time() - time_offset),
					pos.dir,
					self.camera_anchor.dir,
					self.world
						.get::<components::WeaponSet>(id)
						.ok()
						.map(|v| v.last_fire_time),
					self.world.get::<components::Velocity>(id).ok().map(|v| *v),
				)
				.unwrap();

			draw_billboard(
				pos.pos,
				self.camera_anchor.dir,
				2. * drawable.size,
				bmp,
				color,
				&mut scene,
			);
		}

		//~ println!("{}", indices.len());

		unsafe {
			gl::Enable(gl::CULL_FACE);
			gl::CullFace(gl::BACK);
		}

		for (i, bucket) in scene.buckets.iter().enumerate()
		{
			state.prim.draw_indexed_prim(
				&bucket.vertices[..],
				Some(&state.atlas.pages[i].bitmap),
				&bucket.indices[..],
				0,
				bucket.indices.len() as u32,
				PrimType::TriangleList,
			);
		}

		let ortho_mat = Matrix4::new_orthographic(
			0.,
			self.display_width as f32,
			self.display_height as f32,
			0.,
			-1.,
			1.,
		);

		unsafe {
			gl::Disable(gl::CULL_FACE);
		}

		state
			.core
			.use_projection_transform(&utils::mat4_to_transform(ortho_mat));
		state.core.use_transform(&Transform::identity());

		let c_ui = Color::from_rgb_f(0.5, 0.5, 0.2);
		let dw = 96.;

		if let Ok(health) = self.world.get::<components::Health>(self.player)
		{
			state.core.draw_text(
				&state.ui_font,
				c_ui,
				48.,
				self.display_height - 72.,
				FontAlign::Centre,
				"HEALTH",
			);

			state.core.draw_text(
				&state.number_font,
				Color::from_rgb_f(0.4, 0.6, 0.4),
				48.,
				self.display_height - 64.,
				FontAlign::Centre,
				&format!("{:.0}", health.health),
			);

			state.core.draw_text(
				&state.ui_font,
				c_ui,
				dw + 48.,
				self.display_height - 72.,
				FontAlign::Centre,
				"ARMOUR",
			);

			state.core.draw_text(
				&state.number_font,
				Color::from_rgb_f(0.4, 0.4, 0.6),
				dw + 48.,
				self.display_height - 64.,
				FontAlign::Centre,
				&format!("{:.0}", health.armour),
			);

			state.core.draw_text(
				&state.ui_font,
				c_ui,
				2. * dw + 48.,
				self.display_height - 72.,
				FontAlign::Centre,
				"LIFES",
			);

			state.core.draw_text(
				&state.number_font,
				Color::from_rgb_f(0.6, 0.4, 0.4),
				2. * dw + 48.,
				self.display_height - 64.,
				FontAlign::Centre,
				&format!("{}", self.lifes),
			);
		}
		else
		{
			state.core.draw_text(
				&state.ui_font,
				c_ui,
				self.display_width / 2.,
				self.display_height / 2. - 16.,
				FontAlign::Centre,
				"YOU HAVE DIED",
			);

			if self.lifes > 0
			{
				state.core.draw_text(
					&state.ui_font,
					c_ui,
					self.display_width / 2.,
					self.display_height / 2. + 16.,
					FontAlign::Centre,
					"PRESS (R) TO RESPAWN",
				);
			}
			else
			{
				state.core.draw_text(
					&state.ui_font,
					c_ui,
					self.display_width / 2.,
					self.display_height / 2. + 16.,
					FontAlign::Centre,
					"PRESS (Q) TO QUIT",
				);
			}
		}

		if let Ok(weapon_set) = self.world.get::<components::WeaponSet>(self.player)
		{
			if let Some(weapon) = weapon_set.weapons.get(&components::WeaponType::OrbGun)
			{
				if weapon.selectable
				{
					state.core.draw_text(
						&state.ui_font,
						c_ui,
						self.display_width - 48.,
						self.display_height - 72.,
						FontAlign::Centre,
						"STARS",
					);

					state.core.draw_text(
						&state.number_font,
						Color::from_rgb_f(0.6, 0.6, 0.6),
						self.display_width - 48.,
						self.display_height - 64.,
						FontAlign::Centre,
						&format!("{}", weapon.ammo),
					);
				}
			}

			if let Some(weapon) = weapon_set.weapons.get(&components::WeaponType::BuggyGun)
			{
				if weapon.selectable
				{
					state.core.draw_text(
						&state.ui_font,
						c_ui,
						self.display_width - 48.,
						self.display_height - 72.,
						FontAlign::Centre,
						"BULLETS",
					);

					state.core.draw_text(
						&state.number_font,
						Color::from_rgb_f(0.6, 0.6, 0.6),
						self.display_width - 48.,
						self.display_height - 64.,
						FontAlign::Centre,
						&format!("{}", weapon.ammo),
					);
				}
			}

			if let Some(weapon) = weapon_set.weapons.get(&components::WeaponType::FreezeGun)
			{
				if weapon.selectable
				{
					state.core.draw_text(
						&state.ui_font,
						c_ui,
						self.display_width - 48. - dw,
						self.display_height - 72.,
						FontAlign::Centre,
						"FUEL",
					);

					state.core.draw_text(
						&state.number_font,
						Color::from_rgb_f(0.6, 0.6, 0.6),
						self.display_width - 48. - dw,
						self.display_height - 64.,
						FontAlign::Centre,
						&format!("{}", weapon.ammo),
					);
				}
			}

			if let Some(weapon) = weapon_set.weapons.get(&components::WeaponType::SantaGun)
			{
				if weapon.selectable
				{
					state.core.draw_text(
						&state.ui_font,
						c_ui,
						self.display_width - 48. - 2. * dw,
						self.display_height - 72.,
						FontAlign::Centre,
						"BULLETS",
					);

					state.core.draw_text(
						&state.number_font,
						Color::from_rgb_f(0.6, 0.6, 0.6),
						self.display_width - 48. - 2. * dw,
						self.display_height - 64.,
						FontAlign::Centre,
						&format!("{}", weapon.ammo),
					);
				}
			}
		}

		Ok(())
	}

	pub fn input(&mut self, event: &Event, _state: &mut game_state::GameState) -> Result<()>
	{
		match event
		{
			Event::MouseAxes { dx, .. } =>
			{
				if *dx < 0
				{
					self.rot_left_state = -*dx;
					self.rot_right_state = 0;
				}
				else if *dx > 0
				{
					self.rot_left_state = 0;
					self.rot_right_state = *dx;
				}
				self.clear_rot = true;
			}
			Event::MouseButtonDown { button, .. } =>
			{
				if *button == 1
				{
					self.fire_state = true;
				}
			}
			Event::MouseButtonUp { button, .. } =>
			{
				if *button == 1
				{
					self.fire_state = false;
				}
			}
			Event::KeyDown { keycode, .. } => match keycode
			{
				KeyCode::_1 =>
				{
					self.desired_weapon = 1;
				}
				KeyCode::_2 =>
				{
					self.desired_weapon = 2;
				}
				KeyCode::_3 =>
				{
					self.desired_weapon = 3;
				}
				KeyCode::_4 =>
				{
					self.desired_weapon = 4;
				}
				KeyCode::_5 =>
				{
					self.desired_weapon = 5;
				}
				KeyCode::W =>
				{
					self.up_state = true;
				}
				KeyCode::S =>
				{
					self.down_state = true;
				}
				KeyCode::A =>
				{
					self.left_state = true;
				}
				KeyCode::D =>
				{
					self.right_state = true;
				}
				KeyCode::E =>
				{
					self.enter_state = true;
				}
				KeyCode::R =>
				{
					if self.world.get::<components::Health>(self.player).is_err() && self.lifes > 0
					{
						self.lifes -= 1;
						self.want_spawn = true;
					}
				}
				KeyCode::Left =>
				{
					self.rot_left_state = 3;
				}
				KeyCode::Right =>
				{
					self.rot_right_state = 3;
				}
				KeyCode::Space =>
				{
					self.fire_state = true;
				}
				_ => (),
			},
			Event::KeyUp { keycode, .. } => match keycode
			{
				KeyCode::W =>
				{
					self.up_state = false;
				}
				KeyCode::S =>
				{
					self.down_state = false;
				}
				KeyCode::A =>
				{
					self.left_state = false;
				}
				KeyCode::D =>
				{
					self.right_state = false;
				}
				KeyCode::E =>
				{
					self.enter_state = false;
				}
				KeyCode::Left =>
				{
					self.rot_left_state = 0;
				}
				KeyCode::Right =>
				{
					self.rot_right_state = 0;
				}
				KeyCode::Space =>
				{
					self.fire_state = false;
				}
				_ => (),
			},
			_ => (),
		}
		Ok(())
	}
}
