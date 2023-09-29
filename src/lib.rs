use wgpu::{util::DeviceExt, Device, ShaderModel};
use cgmath::{prelude::*, Quaternion, Vector3, Euler, Deg};

use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

mod texture;
mod model;
mod resources;

use model::{DrawModel, Vertex};

use std::{thread, time::{self, Duration}, collections::HashMap};

use winit::window::Window;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
	1.0, 0.0, 0.0, 0.0,
	0.0, 1.0, 0.0, 0.0,
	0.0, 0.0, 0.5, 0.0,
	0.0, 0.0, 0.5, 1.0,
);

pub struct Instance {
	pub position: cgmath::Vector3<f32>,
	pub rotation: cgmath::Quaternion<f32>,
	pub scale: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
	model: [[f32; 4]; 4],
}

impl Instance {
	fn to_raw(&self) -> InstanceRaw {
		InstanceRaw {
			model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation) * cgmath::Matrix4::from_scale(self.scale)).into(),
		}
	}
}

impl InstanceRaw {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		use std::mem;
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
			// We need to switch from using a step mode of Vertex to Instance
			// This means that our shaders will only change to use the next
			// instance when the shader starts processing a new instance
			step_mode: wgpu::VertexStepMode::Instance,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					// While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
					// be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
					shader_location: 5,
					format: wgpu::VertexFormat::Float32x4,
				},
				// A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
				// for each vec4. We'll have to reassemble the mat4 in
				// the shader.
				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
					shader_location: 6,
					format: wgpu::VertexFormat::Float32x4,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
					shader_location: 7,
					format: wgpu::VertexFormat::Float32x4,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
					shader_location: 8,
					format: wgpu::VertexFormat::Float32x4,
				},
			],
		}
	}
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
	// We can't use cgmath with bytemuck directly so we'll have
	// to convert the Matrix4 into a 4x4 f32 array
	view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
	fn new() -> Self {
		use cgmath::SquareMatrix;
		Self {
			view_proj: cgmath::Matrix4::identity().into(),
		}
	}

	fn update_view_proj(&mut self, camera: &Camera) {
		self.view_proj = camera.build_view_projection_matrix().into();
	}
}
 
struct Camera {
	eye: cgmath::Point3<f32>,
	target: cgmath::Point3<f32>,
	up: cgmath::Vector3<f32>,
	aspect: f32,
	fovy: f32,
	znear: f32,
	zfar: f32,
}

impl Camera {
	fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
		let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
		let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

		return OPENGL_TO_WGPU_MATRIX * proj * view;
	}
}
struct CameraController {
	speed: f32,
	is_forward_pressed: bool,
	is_backward_pressed: bool,
	is_left_pressed: bool,
	is_right_pressed: bool,
}

impl CameraController {
	fn new(speed: f32) -> Self {
		Self {
			speed,
			is_forward_pressed: false,
			is_backward_pressed: false,
			is_left_pressed: false,
			is_right_pressed: false,
		}
	}

	fn process_events(&mut self, event: &WindowEvent) -> bool {
		match event {
			WindowEvent::KeyboardInput {
				input: KeyboardInput {
					state,
					virtual_keycode: Some(keycode),
					..
				},
				..
			} => {
				let is_pressed = *state == ElementState::Pressed;
				match keycode {
					VirtualKeyCode::W | VirtualKeyCode::Up => {
						self.is_forward_pressed = is_pressed;
						true
					}
					VirtualKeyCode::A | VirtualKeyCode::Left => {
						self.is_left_pressed = is_pressed;
						true
					}
					VirtualKeyCode::S | VirtualKeyCode::Down => {
						self.is_backward_pressed = is_pressed;
						true
					}
					VirtualKeyCode::D | VirtualKeyCode::Right => {
						self.is_right_pressed = is_pressed;
						true
					}
					_ => false,
				}
			}
			_ => false,
		}
	}

	fn update_camera(&self, camera: &mut Camera) {
		use cgmath::InnerSpace;
		let forward = camera.target - camera.eye;
		let forward_norm = forward.normalize();
		let forward_mag = forward.magnitude();

		// Prevents glitching when camera gets too close to the
		// center of the scene.
		if self.is_forward_pressed && forward_mag > self.speed {
			camera.eye += forward_norm * self.speed;
		}
		if self.is_backward_pressed {
			camera.eye -= forward_norm * self.speed;
		}

		let right = forward_norm.cross(camera.up);

		// Redo radius calc in case the fowrard/backward is pressed.
		let forward = camera.target - camera.eye;
		let forward_mag = forward.magnitude();

		if self.is_right_pressed {
			// Rescale the distance between the target and eye so 
			// that it doesn't change. The eye therefore still 
			// lies on the circle made by the target and eye.
			camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
		}
		if self.is_left_pressed {
			camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
		}
	}
}

use redkar_chess::*;


#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CompareFunction {
	Undefined = 0,
	Never = 1,
	Less = 2,
	Equal = 3,
	LessEqual = 4,
	Greater = 5,
	NotEqual = 6,
	GreaterEqual = 7,
	Always = 8,
}

pub fn piece_to_key(piece: Piece) -> String {
	match piece {
		Piece { color: Color::White, piece: PieceType::Pawn } => "pawn_white",
		Piece { color: Color::White, piece: PieceType::Knight } => "knight_white",
		Piece { color: Color::White, piece: PieceType::Bishop } => "bishop_white",
		Piece { color: Color::White, piece: PieceType::Rook } => "rook_white",
		Piece { color: Color::White, piece: PieceType::Queen } => "queen_white",
		Piece { color: Color::White, piece: PieceType::King } => "king_white",
		Piece { color: Color::Black, piece: PieceType::Pawn } => "pawn_black",
		Piece { color: Color::Black, piece: PieceType::Knight } => "knight_black",
		Piece { color: Color::Black, piece: PieceType::Bishop } => "bishop_black",
		Piece { color: Color::Black, piece: PieceType::Rook } => "rook_black",
		Piece { color: Color::Black, piece: PieceType::Queen } => "queen_black",
		Piece { color: Color::Black, piece: PieceType::King } => "king_black",
	}.to_string()
}

pub fn key_to_piece(key: &str) -> Piece {
	match key {
		"pawn_white" => Piece { color: Color::White, piece: PieceType::Pawn },
		"knight_white" => Piece { color: Color::White, piece: PieceType::Knight },
		"bishop_white" => Piece { color: Color::White, piece: PieceType::Bishop },
		"rook_white" => Piece { color: Color::White, piece: PieceType::Rook },
		"queen_white" => Piece { color: Color::White, piece: PieceType::Queen },
		"king_white" => Piece { color: Color::White, piece: PieceType::King },
		"pawn_black" => Piece { color: Color::Black, piece: PieceType::Pawn },
		"knight_black" => Piece { color: Color::Black, piece: PieceType::Knight },
		"bishop_black" => Piece { color: Color::Black, piece: PieceType::Bishop },
		"rook_black" => Piece { color: Color::Black, piece: PieceType::Rook },
		"queen_black" => Piece { color: Color::Black, piece: PieceType::Queen },
		"king_black" => Piece { color: Color::Black, piece: PieceType::King },
		_ => panic!("Invalid key"),
	}
}

pub struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	pub size: winit::dpi::PhysicalSize<u32>,
	window: Window,
	render_pipeline: wgpu::RenderPipeline,
	diffuse_bind_group: wgpu::BindGroup,
	diffuse_texture: texture::Texture,
	camera: Camera,
	camera_uniform: CameraUniform,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
	camera_controller: CameraController,
	depth_texture: texture::Texture,
	obj_model: HashMap<String, model::Model>,
}

impl State {
	// Creating some of the wgpu types requires async code

	pub fn window(&self) -> &Window {
		&self.window
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
		}
		
		self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
	}

	pub fn input(&mut self, event: &WindowEvent) -> bool {
		self.camera_controller.process_events(event)
	}

	pub fn update(&mut self) {
		self.camera_controller.update_camera(&mut self.camera);
		self.camera_uniform.update_view_proj(&self.camera);
		self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
	}

	pub fn render(&mut self, instances: &Vec<Instance>, instance_role: Piece) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		});

		let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
		let instance_buffer = self.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Instance Buffer"),
				contents: bytemuck::cast_slice(&instance_data),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.1,
							g: 0.2,
							b: 0.3,
							a: 0.0,
						}),
						store: true,
					},
				})],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
					view: &self.depth_texture.view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: true,
					}),
					stencil_ops: None,
				}),
			});

			render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.draw_model_instanced(&self.obj_model[&piece_to_key(instance_role)], 0..instances.len() as u32, &self.camera_bind_group);
		}

		// submit will accept anything that implements IntoIter
		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		Ok(())
	}

	pub async fn new(window: Window) -> Self {
		let size = window.inner_size();
		let camera_controller = CameraController::new(0.2);

		// The instance is a handle to our GPU
		// Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			dx12_shader_compiler: Default::default(),
		});
		
		// # Safety
		//
		// The surface needs to live as long as the window that created it.
		// State owns the window so this should be safe.
		let surface = unsafe { instance.create_surface(&window) }.unwrap();

		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			},
		).await.unwrap();

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				// WebGL doesn't support all of wgpu's features, so if
				// we're building for the web we'll have to disable some.
				limits: if cfg!(target_arch = "wasm32") {
					wgpu::Limits::downlevel_webgl2_defaults()
				} else {
					wgpu::Limits::default()
				},
				label: None,
			},
			None, // Trace path
		).await.unwrap();

		let surface_caps = surface.get_capabilities(&adapter);
		// Shader code in this tutorial assumes an sRGB surface texture. Using a different
		// one will result all the colors coming out darker. If you want to support non
		// sRGB surfaces, you'll need to account for that when drawing to the frame.
		let surface_format = surface_caps.formats.iter()
			.copied()
			.filter(|f| f.describe().srgb)
			.next()
			.unwrap_or(surface_caps.formats[0]);
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.width,
			height: size.height,
			present_mode: surface_caps.present_modes[0],
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![],
		};
		surface.configure(&device, &config);

		let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
		});

		let surface_caps = surface.get_capabilities(&adapter);
		// Shader code in this tutorial assumes an sRGB surface texture. Using a different
		// one will result all the colors coming out darker. If you want to support non
		// sRGB surfaces, you'll need to account for that when drawing to the frame.
		let surface_format = surface_caps.formats.iter()
			.copied()
			.filter(|f| f.describe().srgb)
			.next()
			.unwrap_or(surface_caps.formats[0]);
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.width,
			height: size.height,
			present_mode: surface_caps.present_modes[0],
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![],
		};
		surface.configure(&device, &config);

		let diffuse_bytes = include_bytes!("happy-tree.png");
		let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

		let texture_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						// This should match the filterable field of the
						// corresponding Texture entry above.
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
				label: Some("texture_bind_group_layout"),
			});

		let mut obj_model = HashMap::<String, model::Model>::new();
		obj_model.insert(
			piece_to_key(Piece { color: Color::White, piece: PieceType::Pawn }),
			resources::load_model("Pawn_w.obj", &device, &queue, &texture_bind_group_layout)
				.await
				.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::White, piece: PieceType::Knight }), resources::load_model("Knight_w.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::White, piece: PieceType::Bishop }), resources::load_model("Bishop_w.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::White, piece: PieceType::Rook }), resources::load_model("Rook_w.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::White, piece: PieceType::Queen }), resources::load_model("Queen_w.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::White, piece: PieceType::King }), resources::load_model("King_w.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::Black, piece: PieceType::Pawn }), resources::load_model("Pawn_b.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::Black, piece: PieceType::Knight }), resources::load_model("Knight_b.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::Black, piece: PieceType::Bishop }), resources::load_model("Bishop_b.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::Black, piece: PieceType::Rook }), resources::load_model("Rook_b.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::Black, piece: PieceType::Queen }), resources::load_model("Queen_b.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());
		obj_model.insert(piece_to_key(Piece { color: Color::Black, piece: PieceType::King }), resources::load_model("King_b.obj", &device, &queue, &texture_bind_group_layout).await.unwrap());

		let diffuse_bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				layout: &texture_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
					}
				],
				label: Some("diffuse_bind_group"),
			}
		);

		let camera = Camera {
			// position the camera one unit up and 2 units back
			// +z is out of the screen
			eye: (0.0, 0.0, 70.0).into(),
			// have it look at the origin
			target: (0.0, 0.0, 0.0).into(),
			// which way is "up"
			up: cgmath::Vector3::unit_y(),
			aspect: config.width as f32 / config.height as f32,
			fovy: 45.0,
			znear: 0.1,
			zfar: 10000.0,
		};

		let mut camera_uniform = CameraUniform::new();
		camera_uniform.update_view_proj(&camera);

		let camera_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Camera Buffer"),
				contents: bytemuck::cast_slice(&[camera_uniform]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			}
		);

		let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}
			],
			label: Some("camera_bind_group_layout"),
		});

		let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &camera_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: camera_buffer.as_entire_binding(),
				}
			],
			label: Some("camera_bind_group"),
		});

		let render_pipeline_layout = device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[
					&texture_bind_group_layout,
					&camera_bind_group_layout,
				],
				push_constant_ranges: &[],
			}
		);
		
		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main", // 1.
				buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
			},
			fragment: Some(wgpu::FragmentState { // 3.
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState { // 4.
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList, // 1.
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw, // 2.
				cull_mode: Some(wgpu::Face::Back),
				// Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
				polygon_mode: wgpu::PolygonMode::Fill,
				// Requires Features::DEPTH_CLIP_CONTROL
				unclipped_depth: false,
				// Requires Features::CONSERVATIVE_RASTERIZATION
				conservative: false,
			},
			multisample: wgpu::MultisampleState {
				count: 1, // 2.
				mask: !0, // 3.
				alpha_to_coverage_enabled: false, // 4.
			},
			multiview: None, // 5.
			depth_stencil: Some(wgpu::DepthStencilState {
				format: texture::Texture::DEPTH_FORMAT,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less, // 1.
				stencil: wgpu::StencilState::default(), // 2.
				bias: wgpu::DepthBiasState::default(),
			}),
		});

		Self {
			window,
			surface,
			device,
			queue,
			config,
			size,
			render_pipeline,
			diffuse_bind_group,
			diffuse_texture,
			camera,
			camera_uniform,
			camera_buffer,
			camera_bind_group,
			camera_controller,
			depth_texture,
			obj_model,
		}
	}
}

