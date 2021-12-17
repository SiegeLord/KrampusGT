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

pub const TILE: f32 = 64.;

fn draw_billboard(
	pos: Point3<f32>, camera_angle: f32, width: f32, height: f32, vertices: &mut Vec<Vertex>,
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
		x: pos1.x,
		y: pos1.y,
		z: pos1.z,
		u: 0.,
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
}

pub struct Level
{
	width: i32,
	height: i32,
}

impl Level
{
	pub fn new(width: i32, height: i32) -> Self
	{
		Level {
			width: width,
			height: height,
		}
	}

	pub fn draw(&self, vertices: &mut Vec<Vertex>)
	{
		let bmp_width = 256.;
		let bmp_height = 256.;

		let offt_x = self.width as f32 * TILE / 2.;
		let offt_z = self.height as f32 * TILE / 2.;

		for z in 0..self.height
		{
			for x in 0..self.width
			{
				let color = Color::from_rgb_f(0.5 + 0.5 * z as f32 / self.height as f32, 1., 1.);

				let shift_x = x as f32 * TILE + TILE / 2. - offt_x;
				let shift_z = z as f32 * TILE + TILE / 2. - offt_z;

				vertices.push(Vertex {
					x: shift_x + 0.,
					y: 0.,
					z: shift_z + 0.,
					u: 0.,
					v: 0.,
					color: color,
				});
				vertices.push(Vertex {
					x: shift_x + TILE,
					y: 0.,
					z: shift_z + 0.,
					u: bmp_width,
					v: 0.,
					color: color,
				});
				vertices.push(Vertex {
					x: shift_x + TILE,
					y: 0.,
					z: shift_z + TILE,
					u: bmp_width,
					v: bmp_height,
					color: color,
				});

				vertices.push(Vertex {
					x: shift_x + 0.,
					y: 0.,
					z: shift_z + 0.,
					u: 0.,
					v: 0.,
					color: color,
				});
				vertices.push(Vertex {
					x: shift_x + TILE,
					y: 0.,
					z: shift_z + TILE,
					u: bmp_width,
					v: bmp_height,
					color: color,
				});
				vertices.push(Vertex {
					x: shift_x + 0.,
					y: 0.,
					z: shift_z + TILE,
					u: 0.,
					v: bmp_height,
					color: color,
				});
			}
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
				size: TILE / 2.,
				mass: 1.,
			},
		));

		for z in 0..=20
		{
			for x in -1..=1
			{
				world.spawn((
					components::Position {
						pos: Point3::new(x as f32 * 128., 0., 256. + z as f32 * 128.),
						dir: 0.,
					},
					components::Drawable,
					components::Solid {
						size: TILE / 2.,
						mass: f32::INFINITY, //1. * z as f32,
					},
				));
			}
		}

		state.cache_bitmap("data/test.png")?;

		Ok(Self {
			projection: utils::projection_transform(display_width, display_height),
			display_width: display_width,
			display_height: display_height,
			level: Level::new(256, 256),
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
		let offt = rot * Vector2::new(0., -2. * TILE);
		let height = TILE * 2.;

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

		self.level.draw(&mut vertices);

		for (_, (pos, _)) in self
			.world
			.query::<(&components::Position, &components::Drawable)>()
			.iter()
		{
			draw_billboard(pos.pos, self.camera_anchor.dir, 64., 64., &mut vertices);
		}

		state.prim.draw_prim(
			&vertices[..],
			Some(bmp),
			0,
			vertices.len() as u32,
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
			let speed = 500.;
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

			//~ dbg!(id);
		}
		//~ println!();

		let mut tree = broccoli::new(&mut boxes);
		let mut colliding_pairs = vec![];
		tree.find_colliding_pairs_mut(|a, b| {
			colliding_pairs.push((a.inner, b.inner));
		});

		for _ in 0..5
		{
			for &(id1, id2) in &colliding_pairs
			{
				//~ dbg!(id1, id2);
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
