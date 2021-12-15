use crate::error::Result;
use crate::{game_state, utils};

use allegro::*;
use allegro_font::*;
use allegro_primitives::*;
use na::{
	Isometry3, Matrix4, Perspective3, Point2, Point3, Quaternion, RealField, Rotation2, Rotation3,
	Unit, Vector2, Vector3, Vector4,
};
use nalgebra as na;

pub const TILE: f32 = 64.;

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

	player_pos: Point3<f32>,
}

impl Map
{
	pub fn new(
		state: &mut game_state::GameState, display_width: f32, display_height: f32,
	) -> Result<Self>
	{
		state.cache_bitmap("data/test.png")?;

		Ok(Self {
			projection: utils::projection_transform(display_width, display_height),
			display_width: display_width,
			display_height: display_height,
			level: Level::new(256, 256),
			player_pos: Point3::new(0., 0., 0.),
		})
	}

	fn make_camera(&self) -> Isometry3<f32>
	{
		let height = TILE * 2.;
		utils::camera_project(
			self.player_pos.x,
			height,
			self.player_pos.z + 2. * TILE,
			self.player_pos.z,
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
		let bmp_width = bmp.get_width() as f32;
		let bmp_height = bmp.get_width() as f32;

		let shift_x = 0.;
		let shift_z = 0.;
		let mut vertices = vec![];

		self.level.draw(&mut vertices);

		let player_x = self.player_pos.x - TILE / 2.;
		let player_z = self.player_pos.z;
		let color = Color::from_rgb_f(1., 0.5, 0.5);

		vertices.push(Vertex {
			x: player_x + 0.,
			y: 0.,
			z: player_z,
			u: 0.,
			v: 0.,
			color: color,
		});
		vertices.push(Vertex {
			x: player_x + TILE,
			y: 0.,
			z: player_z,
			u: bmp_width,
			v: 0.,
			color: color,
		});
		vertices.push(Vertex {
			x: player_x + TILE,
			y: TILE,
			z: player_z,
			u: bmp_width,
			v: bmp_height,
			color: color,
		});

		vertices.push(Vertex {
			x: player_x + 0.,
			y: 0.,
			z: player_z,
			u: 0.,
			v: 0.,
			color: color,
		});
		vertices.push(Vertex {
			x: player_x + TILE,
			y: TILE,
			z: player_z,
			u: bmp_width,
			v: bmp_height,
			color: color,
		});
		vertices.push(Vertex {
			x: player_x + 0.,
			y: TILE,
			z: player_z,
			u: 0.,
			v: bmp_height,
			color: color,
		});

		state.prim.draw_prim(
			&vertices[..],
			Some(bmp),
			0,
			vertices.len() as u32,
			PrimType::TriangleList,
		);

		Ok(())
	}

	pub fn logic(&mut self, logic: &mut game_state::GameState) -> Result<()>
	{
		Ok(())
	}
}
