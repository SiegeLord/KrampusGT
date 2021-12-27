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

pub const TILE: f32 = 64.;

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
	pos: Point3<f32>, camera_angle: f32, size: f32, bitmap: &atlas::AtlasBitmap, scene: &mut Scene,
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

	let color = Color::from_rgb_f(1., 1., 1.);

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
	pub fn new(filename: &str) -> Result<Self>
	{
		let desc: LevelDesc = utils::load_config(filename)?;

		let tile_meshes = load_meshes(&desc.meshes);

		let map = tiled::parse_file(&Path::new(&desc.level))?;
		let layer_tiles = &map.layers[0].tiles;
		let layer_tiles = match layer_tiles
		{
			tiled::LayerData::Finite(layer_tiles) => layer_tiles,
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
				tiles.push((tile.gid - tileset.first_gid) as i32);
			}
		}

		let mut tile_meshes_vec = Vec::with_capacity(tile_meshes.len());
		for i in 0..tile_meshes.len()
		{
			tile_meshes_vec.push(tile_meshes[&i.to_string()].clone())
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
			.get_sprite_sheet("data/test.cfg")
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
							u: bmp.start.x + (vtx.x + 32.) / 64. * bmp.width(),
							v: bmp.start.y + (vtx.z + 32.) / 64. * bmp.height(),
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
				if tile == 0
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
				if tile == 0
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
	pos: Point3<f32>, dir: f32, speed: f32, lifetime: f64, size: f32,
	state: &mut game_state::GameState, world: &mut hecs::World,
) -> hecs::Entity
{
	let id = world.spawn((
		components::Position { pos: pos, dir: dir },
		components::Velocity {
			vel: speed * utils::dir_vec3(dir),
			dir_vel: 0.,
		},
		components::Drawable {
			size: size,
			sprite_sheet: "data/bullet.cfg".into(),
		},
		components::Solid {
			size: size,
			mass: 0.,
			collision_class: components::CollisionClass::Tiny,
		},
		components::TimeToDie {
			time_to_die: state.time() + lifetime,
		},
		components::OnContactEffect {
			effects: vec![
				components::ContactEffect::Die,
				components::ContactEffect::Hurt { damage: 6. },
			],
		},
		components::OnDeathEffect {
			effects: vec![components::DeathEffect::Spawn(Box::new(
				move |pos, _, state, world| {
					spawn_explosion(pos, size, "data/purple_explosion.cfg".into(), state, world)
				},
			))],
		},
	));

	dbg!(id, "spawn_projectile", pos);
	id
}

pub fn spawn_corpse(
	pos: Point3<f32>, size: f32, sprite_sheet: String, world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		components::Position { pos: pos, dir: 0. },
		components::Drawable {
			size: size,
			sprite_sheet: sprite_sheet,
		},
	))
}

pub fn spawn_explosion(
	pos: Point3<f32>, size: f32, sprite_sheet: String, state: &mut game_state::GameState,
	world: &mut hecs::World,
) -> hecs::Entity
{
	world.spawn((
		components::Position { pos: pos, dir: 0. },
		components::Drawable {
			size: size,
			sprite_sheet: sprite_sheet,
		},
		components::TimeToDie {
			time_to_die: state.time() + 0.25,
		},
		components::CreationTime { time: state.time() },
	))
}

pub fn spawn_player(
	pos: Point3<f32>, dir: f32, health_frac: f32, world: &mut hecs::World,
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
		components::WeaponSet {
			weapons: vec![components::Weapon::buggy_gun()],
			cur_weapon: 0,
			want_to_fire: false,
			last_fire_time: -f64::INFINITY,
		},
		components::Health {
			health: 100. * health_frac,
		},
		components::OnDeathEffect {
			effects: vec![components::DeathEffect::Spawn(Box::new(
				move |pos, _, _, world| {
					spawn_corpse(pos, size, "data/santa_corpse.cfg".into(), world)
				},
			))],
		},
		components::Team::Player,
	))
}

pub fn spawn_buggy(
	pos: Point3<f32>, dir: f32, health_frac: f32, world: &mut hecs::World,
) -> hecs::Entity
{
	let size = 4. * TILE / 8.;
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
			health: 50. * health_frac,
		},
		components::OnDeathEffect {
			effects: vec![components::DeathEffect::Spawn(Box::new(
				move |pos, _, state, world| {
					spawn_explosion(pos, size, "data/purple_explosion.cfg".into(), state, world)
				},
			))],
		},
		components::Team::Neutral,
		components::WeaponSet {
			weapons: vec![components::Weapon::buggy_gun()],
			cur_weapon: 0,
			want_to_fire: false,
			last_fire_time: -f64::INFINITY,
		},
		components::Vehicle { contents: None },
	))
}

pub fn spawn_monster(
	pos: Point3<f32>, dir: f32, health_frac: f32, world: &mut hecs::World,
) -> hecs::Entity
{
	let size = 2. * TILE / 8.;
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
			health: 10. * health_frac,
		},
		components::OnDeathEffect {
			effects: vec![components::DeathEffect::Spawn(Box::new(
				move |pos, _, _, world| {
					spawn_corpse(pos, size, "data/cat_corpse.cfg".into(), world)
				},
			))],
		},
		components::Team::Monster,
		components::WeaponSet {
			weapons: vec![components::Weapon::santa_gun()],
			cur_weapon: 0,
			want_to_fire: false,
			last_fire_time: -f64::INFINITY,
		},
		components::AI {
			sense_range: TILE * 4.,
			attack_range: TILE * 3.,
			disengage_range: TILE * 5.,
			status: components::Status::Idle,
			time_to_check_status: 0.,
		},
	))
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
				if let (Some(pos_iter), Some(norm_iter)) =
					(reader.read_positions(), reader.read_normals())
				{
					for (pos, _) in pos_iter.zip(norm_iter)
					{
						vtxs.push(Vertex {
							x: pos[0],
							y: pos[1] + dy,
							z: pos[2],
							u: 0.,
							v: 0.,
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

	test: hecs::Entity,

	world: hecs::World,
}

impl Map
{
	pub fn new(
		state: &mut game_state::GameState, display_width: f32, display_height: f32,
	) -> Result<Self>
	{
		let mut world = hecs::World::default();

		let player_pos = components::Position {
			pos: Point3::new(0., 0., 0.),
			dir: 0.,
		};
		let player = spawn_player(player_pos.pos, player_pos.dir, 1., &mut world);
		let mut test = player;
		//~ for z in [0]
		for z in -1..=1
		//~ for z in [0, -1]
		{
			//~ break;
			//~ for x in [0]
			for x in -1..=1
			{
				let pos = Point3::new(x as f32 * 32., 0., 256. + z as f32 * 32.);
				let monster = spawn_monster(pos, 0., 1., &mut world);
				if z == 0
				{
					test = monster;
					dbg!(test);
				}
			}
		}

		spawn_buggy(Point3::new(-512., 0., 512.), 0., 1., &mut world);

		state.cache_sprite_sheet("data/purple_explosion.cfg")?;
		state.cache_sprite_sheet("data/buggy.cfg")?;
		state.cache_sprite_sheet("data/cat.cfg")?;
		state.cache_sprite_sheet("data/cat_corpse.cfg")?;
		state.cache_sprite_sheet("data/santa.cfg")?;
		state.cache_sprite_sheet("data/santa_corpse.cfg")?;
		state.cache_sprite_sheet("data/bullet.cfg")?;
		state.cache_sprite_sheet("data/test.cfg")?;
		//~ state.atlas.dump_pages();

		Ok(Self {
			test: test,
			projection: utils::projection_transform(display_width, display_height),
			display_width: display_width,
			display_height: display_height,
			level: Level::new("data/level.cfg")?,
			player: player,
			camera_anchor: player_pos,
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

				let diff = 0.9 * diff * (solid1.size + solid2.size - diff_norm) / diff_norm;
				let diff = Vector3::new(diff.x, 0., diff.y);

				let f = 1. - solid1.mass / (solid2.mass + solid1.mass);
				if f32::is_finite(f)
				{
					self.world.get_mut::<components::Position>(id1)?.pos -= diff * f;
					self.world.get_mut::<components::Position>(id2)?.pos += diff * (1. - f);
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

		// Player controller.
		if self.world.contains(self.player)
		{
			let rot_left_right = self.rot_right_state - (self.rot_left_state as i32);
			let left_right = self.left_state as i32 - (self.right_state as i32);
			let up_down = self.up_state as i32 - (self.down_state as i32);

			let pos = *self.world.get::<components::Position>(self.player)?;
			let dir = pos.dir;
			let rot = Rotation2::new(dir);
			let speed = 100.;
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
						vehicle.contents = Some(Box::new(move |pos, dir, world| {
							spawn_player(pos, dir, 1., world)
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
			if !weapon_set.want_to_fire
				|| weapon_set.weapons.is_empty()
				|| weapon_set.weapons[weapon_set.cur_weapon].time_to_fire > state.time()
			{
				continue;
			}

			let weapon = &mut weapon_set.weapons[weapon_set.cur_weapon];
			weapon.time_to_fire = state.time() + weapon.delay;

			let proj_size = 4.;

			match weapon.weapon_type
			{
				components::WeaponType::SantaGun =>
				{
					let spawn_pos =
						pos.pos + utils::dir_vec3(pos.dir) * (solid.size + proj_size + 1.);
					proj_spawns.push((spawn_pos, pos.dir, proj_size));
				}
				components::WeaponType::BuggyGun =>
				{
					let forward = solid.size + proj_size + 1.;
					let dir = utils::dir_vec3(pos.dir);
					let left = 10. * Vector3::new(-dir.z, 0., dir.x);
					let spawn_pos = pos.pos + forward * dir + left;
					proj_spawns.push((spawn_pos, pos.dir, proj_size));
					let spawn_pos = pos.pos + forward * dir - left;
					proj_spawns.push((spawn_pos, pos.dir, proj_size));
				}
			}
			weapon_set.last_fire_time = state.time();
		}

		for (pos, dir, proj_size) in proj_spawns
		{
			spawn_projectile(
				pos + Vector3::new(0., 8., 0.),
				dir,
				256.,
				2.,
				proj_size,
				state,
				&mut self.world,
			);
		}

		// Velocity handling.
		for (_, (pos, vel)) in self
			.world
			.query_mut::<(&mut components::Position, &components::Velocity)>()
		{
			pos.pos += utils::DT * vel.vel;
			pos.dir += utils::DT * vel.dir_vel;
			//pos.dir = pos.dir.fmod(2. * f32::pi());
		}

		// AI
		for (id, (pos, vel, team, ai)) in self
			.world
			.query::<(
				&components::Position,
				&mut components::Velocity,
				&components::Team,
				&mut components::AI,
			)>()
			.iter()
		{
			//~ if id == self.test
			//~ {
			//~ println!("s---- {} {:?}", state.time(), ai.status);
			//~ }
			if ai.time_to_check_status < state.time()
			{
				let rot_speed = f32::pi();
				let speed = 50.;
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
							health.health -= damage;
						}
					}
					_ => (),
				}
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

				if let Ok(mut on_death_effect) = self.world.get_mut::<components::OnDeathEffect>(id)
				{
					for effect in on_death_effect.effects.drain(..)
					{
						match effect
						{
							components::DeathEffect::Spawn(spawn_fn) =>
							{
								dbg!(id, "spawning", point_pos);
								spawn_fns.push((
									false,
									Box::new(move |state, world| {
										spawn_fn(point_pos, dir, state, world)
									}),
								));
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

			let sheet = state.get_sprite_sheet(&drawable.sprite_sheet).unwrap();
			let bmp = &sheet
				.get_bitmap(
					state.time() - time_offset,
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
				&mut scene,
			);
		}

		//~ println!("{}", indices.len());

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
