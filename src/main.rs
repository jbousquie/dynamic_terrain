pub mod ribbon;
pub mod terrain;
pub mod wireframe;

// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}


use crate::wireframe::wireframe::apply_wireframe;
use terrain::terrain::*;
use three_d::*;

const GROUNDFILE : &str = "assets/ground.jpeg";
const GROUNDASSET : &str = "ground";
const CLEARCOLOR : (f32, f32, f32, f32, f32) = (0.7, 0.8, 0.98, 1.0, 1.0);


pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 800.0, 1000.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        10000.0,
    );
    let mut control = OrbitControl::new(camera.target(), 1.0, 100.0);
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
    cpu_texture.data.to_color();
    let cpu_material = CpuMaterial {
        albedo: Srgba { r: 220, g: 220, b: 255, a: 255, },
        //albedo_texture: Some(cpu_texture),
        ..Default::default()
    };
    let material = PhysicalMaterial::new_opaque(&context, &cpu_material);




    let cpu_mesh: CpuMesh = create_map_terrain();
    let wireframe = apply_wireframe(&context, &cpu_mesh);

    // Mesh
    let mut mesh = Gm::new(
        Mesh::new(&context, &cpu_mesh),
        material,
    );
    mesh.set_transformation(Mat4::from_nonuniform_scale(50.0, 20.0, 50.0));


    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);
        frame_input
            .screen()
            .clear(ClearState::color_and_depth(CLEARCOLOR.0, CLEARCOLOR.1, CLEARCOLOR.2, CLEARCOLOR.3, CLEARCOLOR.4))
            .render(
                &camera,
                &wireframe,
                &[&light0, &ambient],
            );

        FrameOutput::default()
    });
}




