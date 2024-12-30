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
        max_size: Some((1600, 940)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 50.0, 0.0),
        vec3(0.0, 50.0, -50.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        60000.0,
    );
    //let mut control = OrbitControl::new(camera.target(), 0.5, 10000.0);
    //let mut control = FlyControl::new(2.0);
    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.0, -0.1, -0.25));
    let ambient = AmbientLight::new(&context, 0.2, Srgba::WHITE);


    let mut loaded = three_d_asset::io::load_async(&[GROUNDFILE]).await.unwrap();
    let mut cpu_texture: CpuTexture = loaded.deserialize(GROUNDASSET).unwrap();
    let mipmap = Some(Mipmap { max_ratio: 1, max_levels: 8, filter: Interpolation::Nearest, });
    cpu_texture.min_filter = Interpolation::Linear;
    cpu_texture.mag_filter = Interpolation::Linear;
    cpu_texture.wrap_s = Wrapping::Repeat;
    cpu_texture.wrap_t = Wrapping::Repeat;
    cpu_texture.mipmap = mipmap;
    //cpu_texture.data.to_color();

    

   
   // terrain material
   let cpu_material_terrain = CpuMaterial {
        albedo: Srgba { r: 255, g: 255, b: 255, a: 255, },
        albedo_texture: Some(cpu_texture),

        ..Default::default()
    };
    

    let map = Rc::new(dt::terrain::Map::new());
    let mut terrain = dt::terrain::Terrain::new(&context, Rc::clone(&map), 300, cpu_material_terrain);
    //let mut wireframe = apply_wireframe(&context, &map_mesh);
    //wireframe.set_transformation(Matrix4::from_translation(vec3(0.0, -500.0, 0.0))); // slide down the wireframe


    terrain.camera_pos.x = terrain.position.x;
    terrain.camera_pos.z = terrain.position.z;

    let speed: f32 = 4.0 ;
    let delta_ang_y: f32 = speed * 0.008;   // roll speed
    let mut ang_y: f32 = 0.0;
    let mut ang_x: f32 = 0.0;
    let mut ang_z: f32 = 0.0;
    let dir = camera.view_direction();
    let cam_up  = camera.up();


    let mut pointer_distance_x = 0.0;
    let mut pointer_distance_y = 0.0;

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        //control.handle_events(&mut camera, &mut frame_input.events);
        
        if terrain.camera_pos.y < 1.0 {
            terrain.camera_pos.y = 10.0;
            println!("camera_pos.y < 10.0");
        }
        if terrain.camera_pos.y > 300.0 {
            terrain.camera_pos.y = 280.0;
            println!("camera_pos.y > 300.0");
        }


        for event in frame_input.events.iter() {

            if let Event::MouseMotion {button: _, delta, position, modifiers: _, handled: _ } = *event {
                let width = frame_input.viewport.width as f32;
                let height = frame_input.viewport.height as f32;
                pointer_distance_x = (1.0 - 2.0 *  position.x / width) * 0.5;
                pointer_distance_y = (1.0 - 2.0 * position.y / height) * 0.5;
            } 
        }
        ang_x = pointer_distance_y.atan();
        ang_z = pointer_distance_x.atan(); 
        ang_y += delta_ang_y * ang_z;
        let cam_pos = camera.position();

        let rot_x = Matrix3::from_angle_x(Rad(ang_x));
        let rot_y = Matrix3::from_angle_y(Rad(ang_y));
        let rot_z = Matrix3::from_angle_z(Rad(ang_z * 0.75));
        let rotation = rot_y * rot_x * rot_z;
        let mut rotated_dir = rotation * dir;

        let rotated_up = rotation * cam_up;
        camera.set_view(
            cam_pos, 
            cam_pos + rotated_dir,
            //cam_up,
            rotated_up,
        );


        let direction = camera.view_direction().normalize_to(speed);
        terrain.camera_pos -= direction;


        terrain.update();
        frame_input
            .screen()
            .clear(ClearState::color_and_depth(CLEARCOLOR.0, CLEARCOLOR.1, CLEARCOLOR.2, CLEARCOLOR.3, CLEARCOLOR.4))
            .render(
                &camera,
                &terrain.mesh,
                //.chain(&wireframe),
                &[&light0, &ambient],
            );

        FrameOutput::default()
    });
}




