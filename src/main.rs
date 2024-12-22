pub mod ribbon;
pub mod dt;
pub mod wireframe;

// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}


use crate::wireframe::wireframe::apply_wireframe;
use three_d::*;
use std::rc::Rc;

const GROUNDFILE : &str = "assets/earthDouble.png";
const GROUNDASSET : &str = "earthDouble";
const CLEARCOLOR : (f32, f32, f32, f32, f32) = (0.7, 0.8, 0.98, 1.0, 1.0);


pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "Dynamic Terrain".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 200.0, 400.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        60000.0,
    );
    //let mut control = OrbitControl::new(camera.target(), 0.5, 10000.0);
    let mut control = FlyControl::new(2.0);
    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.0, -0.1, -0.25));
    let ambient = AmbientLight::new(&context, 0.2, Srgba::WHITE);


    let mut loaded = three_d_asset::io::load_async(&[GROUNDFILE]).await.unwrap();
    let mut cpu_texture: CpuTexture = loaded.deserialize(GROUNDASSET).unwrap();
    let mipmap = Some(Mipmap { max_ratio: 1, max_levels: 8, filter: Interpolation::Nearest, });
    cpu_texture.min_filter = Interpolation::Nearest;
    cpu_texture.mag_filter = Interpolation::Nearest;
    cpu_texture.wrap_s = Wrapping::Repeat;
    cpu_texture.wrap_t = Wrapping::Repeat;
    cpu_texture.mipmap = mipmap;
    //cpu_texture.data.to_color();

    

    // map material
    let cpu_material_map = CpuMaterial {
        albedo: Srgba { r: 220, g: 220, b: 255, a: 255, },
        //albedo_texture: Some(cpu_texture),
        ..Default::default()
    };
   let material_map = PhysicalMaterial::new_opaque(&context, &cpu_material_map);
   
   // terrain material
   let cpu_material_terrain = CpuMaterial {
        albedo: Srgba { r: 255, g: 255, b: 255, a: 255, },
        albedo_texture: Some(cpu_texture),

        ..Default::default()
    };
    

    let map = Rc::new(dt::terrain::Map::new());
    let map_mesh: CpuMesh = map.create_mesh(&map.coords, &map.uvs);
    let mut terrain = dt::terrain::Terrain::new(&context, Rc::clone(&map), 200, cpu_material_terrain);
    //let mut wireframe = apply_wireframe(&context, &map_mesh);
    //wireframe.set_transformation(Matrix4::from_translation(vec3(0.0, -500.0, 0.0))); // slide down the wireframe

    // Map mesh
    let mut mesh = Gm::new(Mesh::new(&context, &map_mesh), material_map);
    mesh.set_transformation(Matrix4::from_translation(vec3(0.0, -500.0, 0.0))); // slide down the map ribbon

    terrain.camera_pos.x = terrain.position.x;
    terrain.camera_pos.z = terrain.position.z;

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);
        terrain.camera_pos.x += 3.0;
        terrain.camera_pos.z += 15.0;
        //terrain.mesh.set_transformation(Matrix4::from_translation(vec3(terrain.position.x, 0.0, terrain.position.z)));
        //Texture offset or rotation example
        // if let Some(texture) = terrain.mesh.material.albedo_texture.as_mut() {
        //     texture.transformation = Mat3::from_translation(vec2(terrain.camera_pos.x, terrain.camera_pos.z));
        // }
        terrain.update();
        frame_input
            .screen()
            .clear(ClearState::color_and_depth(CLEARCOLOR.0, CLEARCOLOR.1, CLEARCOLOR.2, CLEARCOLOR.3, CLEARCOLOR.4))
            .render(
                &camera,
                [&mesh, &terrain.mesh],
                //.chain(&wireframe),
                &[&light0, &ambient],
            );

        FrameOutput::default()
    });
}




