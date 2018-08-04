use device_state::DeviceState;
use glsl_to_spirv;
use hal::pass::Subpass;
use hal::pso::{Face, FrontFace, PolygonMode, Rasterizer, Specialization};
use hal::{format as f, pso, Backend, Device, Primitive};
use std;
use std::cell::RefCell;
use std::fs;
use std::io::Read;
use std::mem::size_of;
use std::rc::Rc;
use vertex::Vertex;

const ENTRY_NAME: &str = "main";

pub struct PipelineState<B: Backend> {
  pub pipeline: Option<B::GraphicsPipeline>,
  pub pipeline_layout: Option<B::PipelineLayout>,
  pub device: Rc<RefCell<DeviceState<B>>>,
}

impl<B: Backend> PipelineState<B> {
  pub fn new<IS>(
    desc_layouts: IS,
    render_pass: &B::RenderPass,
    device_ptr: Rc<RefCell<DeviceState<B>>>,
  ) -> Self
  where
    IS: IntoIterator,
    IS::Item: std::borrow::Borrow<B::DescriptorSetLayout>,
  {
    let device = &device_ptr.borrow().device;
    let pipeline_layout =
      device.create_pipeline_layout(desc_layouts, &[(pso::ShaderStageFlags::VERTEX, 0..8)]);

    let pipeline = {
      let vs_module = {
        let glsl = fs::read_to_string("data/shaders/cube.vert").unwrap();
        let spirv: Vec<u8> = glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Vertex)
          .unwrap()
          .bytes()
          .map(|b| b.unwrap())
          .collect();
        device.create_shader_module(&spirv).unwrap()
      };
      let fs_module = {
        let glsl = fs::read_to_string("data/shaders/cube.frag").unwrap();
        let spirv: Vec<u8> = glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Fragment)
          .unwrap()
          .bytes()
          .map(|b| b.unwrap())
          .collect();
        device.create_shader_module(&spirv).unwrap()
      };

      let pipeline = {
        let (vs_entry, fs_entry) = (
          pso::EntryPoint::<B> {
            entry: ENTRY_NAME,
            module: &vs_module,
            specialization: &[Specialization {
              id: 0,
              value: pso::Constant::F32(1.0),
            }],
          },
          pso::EntryPoint::<B> {
            entry: ENTRY_NAME,
            module: &fs_module,
            specialization: &[],
          },
        );

        let shader_entries = pso::GraphicsShaderSet {
          vertex: vs_entry,
          hull: None,
          domain: None,
          geometry: None,
          fragment: Some(fs_entry),
        };

        let subpass = Subpass {
          index: 0,
          main_pass: render_pass,
        };

        let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
          shader_entries,
          Primitive::TriangleList,
          Rasterizer {
            polygon_mode: PolygonMode::Fill,
            cull_face: <Face>::BACK,
            // cull_face: <Face>::NONE,
            front_face: FrontFace::CounterClockwise,
            // front_face: FrontFace::Clockwise,
            depth_clamping: false,
            depth_bias: None,
            conservative: false,
          },
          // pso::Rasterizer::FILL,
          &pipeline_layout,
          subpass,
        );
        pipeline_desc.blender.targets.push(pso::ColorBlendDesc(
          pso::ColorMask::ALL,
          pso::BlendState::ALPHA,
          // pso::BlendState::ADD,
        ));
        pipeline_desc.vertex_buffers.push(pso::VertexBufferDesc {
          binding: 0,
          stride: size_of::<Vertex>() as u32,
          rate: 0,
        });

        pipeline_desc.attributes.push(pso::AttributeDesc {
          location: 0,
          binding: 0,
          element: pso::Element {
            format: f::Format::Rgb32Float, // 3D
            // format: f::Format::Rg32Float,  // 2D
            offset: 0,
          },
        });
        pipeline_desc.attributes.push(pso::AttributeDesc {
          location: 1,
          binding: 0,
          element: pso::Element {
            format: f::Format::Rg32Float,
            offset: 12, // 3D (8 for 2D)
          },
        });

        device.create_graphics_pipeline(&pipeline_desc)
      };

      device.destroy_shader_module(vs_module);
      device.destroy_shader_module(fs_module);

      pipeline.unwrap()
    };

    PipelineState {
      pipeline: Some(pipeline),
      pipeline_layout: Some(pipeline_layout),
      device: Rc::clone(&device_ptr),
    }
  }
}

impl<B: Backend> Drop for PipelineState<B> {
  fn drop(&mut self) {
    let device = &self.device.borrow().device;
    device.destroy_graphics_pipeline(self.pipeline.take().unwrap());
    device.destroy_pipeline_layout(self.pipeline_layout.take().unwrap());
  }
}
