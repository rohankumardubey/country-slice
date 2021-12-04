use asset_libraries::mesh_library::AssetMeshLibrary;
use asset_libraries::shader_library::AssetShaderLibrary;
use asset_libraries::vao_library::AssetVAOLibrary;
use asset_libraries::Handle;

use bevy_app::App;
use bevy_ecs::prelude::*;

use glam::Vec3;
use glutin::event_loop::ControlFlow;

use render::camera::MainCamera;

use render::shader::ShaderProgram;
use render::shaderwatch::*;
use window_events::{process_window_events, CursorMoved, WindowSize};

use crate::systems::*;

mod asset_libraries;
mod components;
mod geometry;
mod render;
mod render_loop;
mod setup;
mod systems;
mod utils;
mod window_events;

// https://github.com/bwasty/learn-opengl-rs
// https://learnopengl.com/Getting-started/Hello-Triangle

// settings
const SCR_WIDTH: u32 = 1600;
const SCR_HEIGHT: u32 = 1200;

// Mark the cube that is the preview of mouse raycast intersection
pub struct MousePreviewCube;

pub struct CursorRaycast(pub Vec3);

pub struct DisplayTestMask;

// TODO: Arches:
// 1. Bricks along the arch
//  - keep userdrawn curves of roads
//  - find intersection of roads with walls (I can find just the uv position, and perform the operation in curve space (projecting will be way easier :P))
//  - in places of intersection, we need to _somehow_ arrange bricks (probably should be separate entity from walls, bc we dont want to boolean bricks)
// 2. Removing intersecting bricks
//  - hook up with my sdf
//  - _somehow_ create a shape of an arch there
//  - discard fragments that are inside
//  - NOPE! I need to construct an sdf in curve space! otherwise, even if the sdf is grazing against the wall, the fragments are discarded... unless I store direction too?
// Q: can I drive the shape of an arch parametrically?

struct ComputeTest {
    compute_program: Handle<ShaderProgram>,
    texture: u32,
    texture_dims: (i32, i32),
}

fn main() {
    let (mut windowed_context, event_loop) =
        setup::setup_glutin_and_opengl((SCR_WIDTH, SCR_HEIGHT));

    let mut temp_shaderwatch = ShaderWatch::new();
    let mut temp_assets_shader = AssetShaderLibrary::new();
    // COMPUTE SHADER -------------------------------------------
    let compute_test = unsafe {
        let texture_dims = (512, 512);
        // Create texture
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA32F as i32,
            texture_dims.0,
            texture_dims.1,
            0,
            gl::RGBA,
            gl::FLOAT,
            std::ptr::null(),
        );
        // create shader program
        let shader_program = ShaderProgram::new_compute("shaders/compute_test.glsl").unwrap();

        temp_shaderwatch.watch(&shader_program);
        let handle = temp_assets_shader.add(shader_program.into());

        ComputeTest {
            compute_program: handle,
            texture,
            texture_dims,
        }
    };

    // ----------------------------------------------------------

    let mut app = App::build();
    app.add_plugin(bevy_core::CorePlugin::default())
        .add_plugin(bevy_input::InputPlugin::default())
        .add_event::<CursorMoved>() // add these events, to avoid loading the whole bevy_window plugin
        .insert_resource(WindowSize::new(SCR_WIDTH, SCR_HEIGHT))
        .insert_resource(MainCamera::new(SCR_WIDTH as f32 / SCR_HEIGHT as f32))
        .insert_resource(temp_shaderwatch)
        .insert_resource(WallManager::new())
        .insert_resource(CursorRaycast(Vec3::ZERO))
        .insert_resource(AssetMeshLibrary::new())
        .insert_resource(AssetVAOLibrary::new())
        .insert_resource(temp_assets_shader)
        .insert_resource(compute_test)
        .add_stage_after(
            bevy_app::CoreStage::PreUpdate,
            "opengl",
            SystemStage::single_threaded(),
        )
        .add_stage_after(
            "opengl",
            "main_singlethread",
            SystemStage::single_threaded(),
        )
        .add_system_to_stage("opengl", shaderwatch.system().label("reload_shaders"))
        .add_system_to_stage("opengl", build_missing_vaos.system().label("build_vaos"))
        .add_system_to_stage("opengl", rebuild_vaos.system().after("build_vaos"))
        .add_system(main_camera_update.system())
        .add_system(mouse_raycast.system())
        .add_system(draw_curve.system().label("usercurve"))
        .add_system_to_stage(
            "main_singlethread",
            walls_update.system().after("usercurve"),
        );

    systems::startup(&mut app.world_mut());

    // main loop
    // -----------
    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't dispatched any events
        *control_flow = ControlFlow::Poll;

        app.app.update();

        process_window_events(
            event,
            &mut windowed_context,
            control_flow,
            &mut app.world_mut(),
        );
    });
}
