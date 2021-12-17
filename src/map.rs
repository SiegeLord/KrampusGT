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

	left_state: bool,
	right_state: bool,
	up_state: bool,
	down_state: bool,

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
		));

		for z in 0..=40
		{
			for x in -20..=20
			{
				world.spawn((
					components::Position {
						pos: Point3::new(x as f32 * 128., 0., 256. + z as f32 * 128.),
						dir: 0.,
					},
					components::Drawable,
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
			left_state: false,
			right_state: false,
			up_state: false,
			down_state: false,
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
		if self.world.contains(self.player)
		{
			let left_right = self.right_state as i32 - (self.left_state as i32);
			let up_down = self.up_state as i32 - (self.down_state as i32);

			let dir = self.world.get::<components::Position>(self.player)?.dir;
			let rot = Rotation2::new(dir);
			let vel = rot * Vector2::new(0., up_down as f32 * 1000.);

			let mut player_vel = self.world.get_mut::<components::Velocity>(self.player)?;
			player_vel.vel = Vector3::new(vel.x, 0., vel.y);
			player_vel.dir_vel = left_right as f32 * f32::pi();
		}

		for (_, (pos, vel)) in self
			.world
			.query::<(&mut components::Position, &components::Velocity)>()
			.iter()
		{
			pos.pos += utils::DT * vel.vel;
			pos.dir += utils::DT * vel.dir_vel;
			//pos.dir = pos.dir.fmod(2. * f32::pi());
		}

		if let Ok(player_pos) = self.world.get::<components::Position>(self.player)
		{
			self.camera_anchor = *player_pos;
		}

		Ok(())
	}

	pub fn input(&mut self, event: &Event, _state: &mut game_state::GameState) -> Result<()>
	{
		match event
		{
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
