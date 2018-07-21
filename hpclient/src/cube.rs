use vertex::*;
use window::*;
use object::*;

use piston_window::*;
use piston_window::Window;
use gfx;
use gfx::*;
use gfx::traits::*;
use shader_version::Shaders;
use shader_version::glsl::GLSL;
use camera_controllers::{
    FirstPersonSettings,
    FirstPerson,
    CameraPerspective,
    model_view_projection
};
use vecmath;
use vecmath::*;
use std::io;
use std::vec::Vec;
use gfx_device_gl;

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    u_model_view_proj: gfx::Global<[[f32; 4]; 4]> = "u_model_view_proj",
    t_color: gfx::TextureSampler<[f32; 4]> = "t_color",
    out_color: gfx::RenderTarget<::gfx::format::Srgba8> = "o_Color",
    out_depth: gfx::DepthTarget<::gfx::format::DepthStencil> =
        gfx::preset::depth::LESS_EQUAL_WRITE,
});

pub struct Cube {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texels: Vec<[u8; 4]>,
    pub first_person: FirstPerson,
    pub model: Matrix4<f32>,
    pub projection: [[f32; 4]; 4],
    pub slice: Slice<gfx_device_gl::Resources>,
    pub pso: PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    pub data: pipe::Data<gfx_device_gl::Resources>,
}

impl Cube {
    pub fn new(pw: &mut _PistonWindow) -> Self {
        let cube_vertex_data = vec![
            //top (0, 0, 1)
            Vertex::new([-1, -1,  1], [0, 0]),
            Vertex::new([ 1, -1,  1], [1, 0]),
            Vertex::new([ 1,  1,  1], [1, 1]),
            Vertex::new([-1,  1,  1], [0, 1]),
            //bottom (0, 0, -1)
            Vertex::new([ 1,  1, -1], [0, 0]),
            Vertex::new([-1,  1, -1], [1, 0]),
            Vertex::new([-1, -1, -1], [1, 1]),
            Vertex::new([ 1, -1, -1], [0, 1]),
            //right (1, 0, 0)
            Vertex::new([ 1, -1, -1], [0, 0]),
            Vertex::new([ 1,  1, -1], [1, 0]),
            Vertex::new([ 1,  1,  1], [1, 1]),
            Vertex::new([ 1, -1,  1], [0, 1]),
            //left (-1, 0, 0)
            Vertex::new([-1,  1,  1], [0, 0]),
            Vertex::new([-1, -1,  1], [1, 0]),
            Vertex::new([-1, -1, -1], [1, 1]),
            Vertex::new([-1,  1, -1], [0, 1]),
            //front (0, 1, 0)
            Vertex::new([-1,  1, -1], [0, 0]),
            Vertex::new([ 1,  1, -1], [1, 0]),
            Vertex::new([ 1,  1,  1], [1, 1]),
            Vertex::new([-1,  1,  1], [0, 1]),
            //back (0, -1, 0)
            Vertex::new([ 1, -1,  1], [0, 0]),
            Vertex::new([-1, -1,  1], [1, 0]),
            Vertex::new([-1, -1, -1], [1, 1]),
            Vertex::new([ 1, -1, -1], [0, 1]),
        ];

        let cube_index_data: Vec<u16> = vec![
            0,  1,  2,  2,  3,  0, // top
            4,  6,  5,  6,  4,  7, // bottom
            8,  9, 10, 10, 11,  8, // right
            12, 14, 13, 14, 12, 15, // left
            16, 18, 17, 18, 16, 19, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        let cube_texel_data: Vec<[u8; 4]> = vec![
            [0xff, 0xff, 0xff, 0x00],
            [0xff, 0x00, 0x00, 0x00],
            [0x00, 0xff, 0x00, 0x00],
            [0x00, 0x00, 0xff, 0x00],
        ];


        let w = &pw.window;
        let ogl = pw.opengl;
        let ref mut f = w.factory.clone();
        let (vbuf, slice) = f.create_vertex_buffer_with_slice(&cube_vertex_data, cube_index_data.as_slice());
        let (_, texture_view) = f.create_texture_immutable::<gfx::format::Rgba8>(
            gfx::texture::Kind::D2(2, 2, gfx::texture::AaMode::Single),
                gfx::texture::Mipmap::Provided,
                &[cube_texel_data.as_slice()]).unwrap();

        let sinfo = gfx::texture::SamplerInfo::new(
        gfx::texture::FilterMethod::Bilinear,
        gfx::texture::WrapMode::Clamp);

        let glsl = ogl.to_glsl();
        let pso = f.create_pipeline_simple(
                Shaders::new()
                    .set(GLSL::V1_20, include_str!("../assets/cube_120.glslv"))
                    .set(GLSL::V1_50, include_str!("../assets/cube_150.glslv"))
                    .get(glsl).unwrap().as_bytes(),
                Shaders::new()
                    .set(GLSL::V1_20, include_str!("../assets/cube_120.glslf"))
                    .set(GLSL::V1_50, include_str!("../assets/cube_150.glslf"))
                    .get(glsl).unwrap().as_bytes(),
                pipe::new()
            ).unwrap();

        let model = vecmath::mat4_id();
        let projection = Self::get_projection(&w);
        let first_person = FirstPerson::new(
            [0.5, 0.5, 4.0],
            FirstPersonSettings::keyboard_wasd()
        );

        let data = pipe::Data {
            vbuf: vbuf.clone(),
            u_model_view_proj: [[0.0; 4]; 4],
            t_color: (texture_view, f.create_sampler(sinfo)),
            out_color: w.output_color.clone(),
            out_depth: w.output_stencil.clone(),
        };

        Self {
            name: String::from("cube"),
            vertices: cube_vertex_data,
            indices: cube_index_data,
            texels: cube_texel_data,
            first_person: first_person,
            model: model,
            projection: projection,
            slice: slice,
            pso: pso,
            data: data,
        }
    }

    pub fn get_projection(w: &PistonWindow) -> Matrix4<f32> {
        let draw_size = w.window.draw_size();
        CameraPerspective {
            fov: 90.0, near_clip: 0.1, far_clip: 1000.0,
            aspect_ratio: (draw_size.width as f32) / (draw_size.height as f32)
        }.projection()
    }

    pub fn reset(&mut self, w: &mut PistonWindow) -> io::Result<i32> {
        self.projection = Self::get_projection(&w);
        self.data.out_color = w.output_color.clone();
        self.data.out_depth = w.output_stencil.clone();

        Ok(0)
    }
}

impl Object for Cube {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn draw(&mut self, w: &mut PistonWindow, args: &RenderArgs) -> io::Result<i32> {
        self.data.u_model_view_proj = model_view_projection(
            self.model,
            self.first_person.camera(args.ext_dt).orthogonal(),
            self.projection
        );
        w.encoder.draw(&self.slice, &self.pso, &self.data);

        Ok(0)
    }
}
