use crate::error::Result;
use crate::{components, game_state, utils};

use allegro::*;
use allegro_font::*;
use allegro_primitives::*;
use na::{
	Isometry3, Matrix4, Perspective3, Point2, Point3, Quaternion, RealField, Rotation2, Rotation3,
	Unit, Vector2, Vector3, Vector4,
};
use nalgebra as na;
use serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;
use std::path::Path;

pub const TILE: f32 = 64.;

fn draw_billboard(
	pos: Point3<f32>, camera_angle: f32, width: f32, height: f32, vertices: &mut Vec<Vertex>,
	indices: &mut Vec<i32>,
)
{
	let rot = Rotation2::new(camera_angle);
	let diff = rot * Vector2::new(0., 1.);
	let horiz_offt = width / 2. * Vector3::new(-diff.y, 0., diff.x);
	let vert_offt = height * Vector3::new(0., 1., 0.);

	let pos1 = pos - horiz_offt + vert_offt;
	let pos2 = pos + horiz_offt + vert_offt;
	let pos3 = pos + horiz_offt;
	let pos4 = pos - horiz_offt;

	let bmp_width = 256.;
	let bmp_height = 256.;

	let color = Color::from_rgb_f(1., 1., 1.);

	let idx = vertices.len() as i32;
	vertices.push(Vertex {
		x: pos1.x,
		y: pos1.y,
		z: pos1.z,
		u: 0.,
		v: 0.,
		color: color,
	});
	vertices.push(Vertex {
		x: pos2.x,
		y: pos2.y,
		z: pos2.z,
		u: bmp_width,
		v: 0.,
		color: color,
	});
	vertices.push(Vertex {
		x: pos3.x,
		y: pos3.y,
		z: pos3.z,
		u: bmp_width,
		v: bmp_height,
		color: color,
	});
	vertices.push(Vertex {
		x: pos4.x,
		y: pos4.y,
		z: pos4.z,
		u: 0.,
		v: bmp_height,
		color: color,
	});

	indices.extend([idx + 0, idx + 1, idx + 2, idx + 0, idx + 2, idx + 3]);
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

	pub fn draw(&self, vertices: &mut Vec<Vertex>, indices: &mut Vec<i32>)
	{
		for z in 0..self.height
		{
			for x in 0..self.width
			{
				let shift_x = x as f32 * TILE + TILE / 2.;
				let shift_z = z as f32 * TILE + TILE / 2.;

				let idx = vertices.len() as i32;

				let mesh = &self.tile_meshes[self.tiles[(x + z * self.width) as usize] as usize];

				for vtx in &mesh.vtxs
				{
					vertices.push(Vertex {
						x: vtx.x + shift_x,
						y: vtx.y,
						z: vtx.z + shift_z,
						u: vtx.u,
						v: vtx.v,
						color: Color::from_rgb_f(vtx.x / 64., vtx.z / 64., vtx.y / 64.),
					});
				}
				for vec_idx in &mesh.idxs
				{
					indices.push(vec_idx + idx);
				}
			}
		}
	}

	pub fn check_collision(&self, loc: Point3<f32>, size: f32) -> Option<Vector3<f32>>
	{
		let center_x = (loc.x / TILE).floor() as i32;
		let center_z = (loc.z / TILE).floor() as i32;

		let mut res = Vector3::zeros();
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
				let nearest_point = Point3::new(nearest_point.x, 0., nearest_point.y);

				let nearest_dist = utils::max(1e-20, (loc - nearest_point).norm());
				let inside = utils::is_inside_poly(&vs, loc.xz());
				if nearest_dist < size || inside
				{
					let new_dir = if inside
					{
						(nearest_point - loc) * (nearest_dist + size) / nearest_dist
					}
					else
					{
						//~ dbg!(nearest_point, (loc - nearest_point) * (size - nearest_dist) / nearest_dist);
						(loc - nearest_point) * (size - nearest_dist) / nearest_dist
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
			Some(res)
		}
		else
		{
			None
		}
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

	world: hecs::World,
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
		let player = world.spawn((
			player_pos,
			components::Velocity {
				vel: Vector3::zeros(),
				dir_vel: 0.,
			},
			components::Drawable,
			components::Solid {
				size: TILE / 8.,
				mass: 1.,
			},
		));

		for z in 0..=20
		{
			for x in -1..=1
			{
				world.spawn((
					components::Position {
						pos: Point3::new(x as f32 * 32., 0., 256. + z as f32 * 32.),
						dir: 0.,
					},
					components::Drawable,
					components::Solid {
						size: TILE / 8.,
						mass: 1. * z as f32,
					},
				));
			}
		}

		state.cache_bitmap("data/test.png")?;

		Ok(Self {
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
		})
	}

	fn make_camera(&self) -> Isometry3<f32>
	{
		let rot = Rotation2::new(self.camera_anchor.dir);
		let offt = rot * Vector2::new(0., -TILE / 2.);
		let height = TILE / 2.;

		utils::camera_project(
			self.camera_anchor.pos.x + offt.x,
			height,
			self.camera_anchor.pos.z + offt.y,
			self.camera_anchor.pos.x,
			self.camera_anchor.pos.z,
		)
	}

	pub fn draw(&mut self, state: &game_state::GameState) -> Result<()>
	{
		state.core.set_depth_test(Some(DepthFunction::Less));
		state
			.core
			.use_projection_transform(&utils::mat4_to_transform(self.projection.into_inner()));

		let camera = self.make_camera();

		state
			.core
			.use_transform(&utils::mat4_to_transform(camera.to_homogeneous()));

		let bmp = state.get_bitmap("data/test.png").unwrap();

		let mut vertices = vec![];
		let mut indices = vec![];

		self.level.draw(&mut vertices, &mut indices);

		for (_, (pos, _)) in self
			.world
			.query::<(&components::Position, &components::Drawable)>()
			.iter()
		{
			draw_billboard(
				pos.pos,
				self.camera_anchor.dir,
				16.,
				16.,
				&mut vertices,
				&mut indices,
			);
		}

		//~ println!("{}", indices.len());

		state.prim.draw_indexed_prim(
			&vertices[..],
			Some(bmp),
			&indices[..],
			0,
			indices.len() as u32,
			PrimType::TriangleList,
		);

		Ok(())
	}

	pub fn logic(&mut self, _state: &mut game_state::GameState) -> Result<()>
	{
		// Player controller.
		if self.world.contains(self.player)
		{
			let rot_left_right = self.rot_right_state - (self.rot_left_state as i32);
			let left_right = self.left_state as i32 - (self.right_state as i32);
			let up_down = self.up_state as i32 - (self.down_state as i32);

			let dir = self.world.get::<components::Position>(self.player)?.dir;
			let rot = Rotation2::new(dir);
			let speed = 100.;
			let vel = rot * Vector2::new(left_right as f32 * speed, up_down as f32 * speed);

			let mut player_vel = self.world.get_mut::<components::Velocity>(self.player)?;
			player_vel.vel = Vector3::new(vel.x, 0., vel.y);
			player_vel.dir_vel = rot_left_right as f32 * f32::pi() / 2.;
		}

		// Velocity handling.
		for (_, (pos, vel)) in self
			.world
			.query::<(&mut components::Position, &components::Velocity)>()
			.iter()
		{
			pos.pos += utils::DT * vel.vel;
			pos.dir += utils::DT * vel.dir_vel;
			//pos.dir = pos.dir.fmod(2. * f32::pi());
		}

		// Collision detection.
		let mut boxes = vec![];
		for (id, (pos, solid)) in self
			.world
			.query::<(&components::Position, &components::Solid)>()
			.iter()
		{
			let r = solid.size;
			let x = pos.pos.x;
			let z = pos.pos.z;
			boxes.push(broccoli::bbox(
				broccoli::rect(x - r, x + r, z - r, z + r),
				id,
			));
		}

		let mut tree = broccoli::new(&mut boxes);
		let mut colliding_pairs = vec![];
		tree.find_colliding_pairs_mut(|a, b| {
			colliding_pairs.push((a.inner, b.inner));
		});

		for _ in 0..5
		{
			for &(id1, id2) in &colliding_pairs
			{
				let pos1 = self.world.get::<components::Position>(id1)?.pos;
				let pos2 = self.world.get::<components::Position>(id2)?.pos;
				let solid1 = *self.world.get::<components::Solid>(id1)?;
				let solid2 = *self.world.get::<components::Solid>(id2)?;

				let diff = pos2 - pos1;
				let diff_norm = utils::max(0.1, diff.norm());

				if diff_norm > solid1.size + solid2.size
				{
					continue;
				}

				let diff = 0.9 * diff * (solid1.size + solid2.size - diff_norm) / diff_norm;

				let mut f = 1. - solid1.mass / (solid2.mass + solid1.mass);
				if !f32::is_finite(solid1.mass)
				{
					f = 0.;
				}
				else if !f32::is_finite(solid2.mass)
				{
					f = 1.;
				}

				self.world.get_mut::<components::Position>(id1)?.pos -= diff * f;
				self.world.get_mut::<components::Position>(id2)?.pos += diff * (1. - f);
			}

			for (_, (pos, solid)) in self
				.world
				.query::<(&mut components::Position, &components::Solid)>()
				.iter()
			{
				if let Some(resolve_diff) = self.level.check_collision(pos.pos, solid.size)
				{
					pos.pos += 0.9 * resolve_diff;
				}
			}
		}

		// Update camera anchor.
		if let Ok(player_pos) = self.world.get::<components::Position>(self.player)
		{
			self.camera_anchor = *player_pos;
		}

		// HACK.
		self.rot_left_state = 0;
		self.rot_right_state = 0;

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
				_ => (),
			},
			_ => (),
		}
		Ok(())
	}
}
