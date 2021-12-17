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

	player_pos: Point3<f32>,
	player_angle: f32,
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
			player_angle: 0.,
		})
	}

	fn make_camera(&self) -> Isometry3<f32>
	{
		let rot = Rotation2::new(self.player_angle);
		let offt = rot * Vector2::new(0., -2. * TILE);
		let height = TILE * 2.;

		utils::camera_project(
			self.player_pos.x + offt.x,
			height,
			self.player_pos.z + offt.y,
			self.player_pos.x,
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

		draw_billboard(
			Point3::new(0., 0., 2048. / 8.),
			self.player_angle,
			256.,
			128.,
			&mut vertices,
		);
		draw_billboard(
			Point3::new(2048. / 8., 0., 2048. / 8.),
			self.player_angle,
			256.,
			512.,
			&mut vertices,
		);
		draw_billboard(
			Point3::new(-2048. / 8., 0., 2048. / 8.),
			self.player_angle,
			256.,
			128.,
			&mut vertices,
		);

		draw_billboard(self.player_pos, self.player_angle, 64., 64., &mut vertices);

		//~ let player_x = self.player_pos.x - TILE / 2.;
		//~ let player_z = self.player_pos.z;
		//~ let color = Color::from_rgb_f(1., 0.5, 0.5);

		//~ vertices.push(Vertex {
		//~ x: player_x + 0.,
		//~ y: 0.,
		//~ z: player_z,
		//~ u: 0.,
		//~ v: 0.,
		//~ color: color,
		//~ });
		//~ vertices.push(Vertex {
		//~ x: player_x + TILE,
		//~ y: 0.,
		//~ z: player_z,
		//~ u: bmp_width,
		//~ v: 0.,
		//~ color: color,
		//~ });
		//~ vertices.push(Vertex {
		//~ x: player_x + TILE,
		//~ y: TILE,
		//~ z: player_z,
		//~ u: bmp_width,
		//~ v: bmp_height,
		//~ color: color,
		//~ });

		//~ vertices.push(Vertex {
		//~ x: player_x + 0.,
		//~ y: 0.,
		//~ z: player_z,
		//~ u: 0.,
		//~ v: 0.,
		//~ color: color,
		//~ });
		//~ vertices.push(Vertex {
		//~ x: player_x + TILE,
		//~ y: TILE,
		//~ z: player_z,
		//~ u: bmp_width,
		//~ v: bmp_height,
		//~ color: color,
		//~ });
		//~ vertices.push(Vertex {
		//~ x: player_x + 0.,
		//~ y: TILE,
		//~ z: player_z,
		//~ u: 0.,
		//~ v: bmp_height,
		//~ color: color,
		//~ });

		state.prim.draw_prim(
			&vertices[..],
			Some(bmp),
			0,
			vertices.len() as u32,
			PrimType::TriangleList,
		);

		Ok(())
	}

	pub fn logic(&mut self, state: &mut game_state::GameState) -> Result<()>
	{
		Ok(())
	}

	pub fn input(&mut self, event: &Event, state: &mut game_state::GameState) -> Result<()>
	{
		match event
		{
			Event::KeyDown { keycode, .. } => match keycode
			{
				KeyCode::W =>
				{
					let rot = Rotation2::new(self.player_angle);
					let diff = rot * Vector2::new(0., 1000000. * utils::DT);
					self.player_pos += utils::DT * Vector3::new(diff.x, 0., diff.y);
				}
				KeyCode::S =>
				{
					let rot = Rotation2::new(self.player_angle);
					let diff = rot * Vector2::new(0., -1000000. * utils::DT);
					self.player_pos += utils::DT * Vector3::new(diff.x, 0., diff.y);
				}
				KeyCode::A =>
				{
					self.player_angle -= utils::DT * 10. * 3.14;
				}
				KeyCode::D =>
				{
					self.player_angle += utils::DT * 10. * 3.14;
				}
				_ => (),
			},
			_ => (),
		}
		Ok(())
	}
}
