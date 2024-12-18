pub mod terrain {
    use fastnoise_lite::*;

    use crate::ribbon::ribbon::*;
    use three_d::{context::DELETE_STATUS, vec3, Context, CpuMaterial, CpuMesh, Gm, Mesh, PhysicalMaterial, Vec3};
    use std::rc::Rc;

    // Number of points in the map
    // set the same value for both WIDTH and HEIGHT
    const WIDTH: usize = 500;
    const HEIGHT: usize = 500;

    pub struct Map {
        pub coords: Vec<Vec<Vec3>>,
        pub length: f32,
        pub subdivisions: usize,
        pub average_sub_size: f32

    }
    impl Map {

        pub fn new() -> Self {
            let coords = Self::create_map();
            let l = coords.len();
            let length = (coords[0][l - 1].z - coords[0][0].z).abs();
            let average_sub_size = length / l as f32;
            Map {
                coords,
                length,
                subdivisions: l,
                average_sub_size
            }
        }
        
        pub fn create_map() -> Vec<Vec<Vec3>> {
            let scl_x = 5.0;
            let scl_y: f32 = 60.0;
            let scl_z = 5.0;
            let hw = WIDTH as f32 * 0.5;
            let hh = HEIGHT as f32 * 0.5;
            let noise_data = Self::create_noise();
            let mut paths = Vec::new();
            for x in 0..WIDTH {
               let mut path = Vec::new();
               for y in 0..HEIGHT {
                   path.push(vec3((x as f32 - hw) * scl_x, noise_data[x][y] * scl_y * ((x as f32 + y as f32) * 0.01).sin(), (y as f32 - hh) * scl_z));
               }
               paths.push(path);
            }
            paths
        }

        pub fn create_noise() -> Vec<Vec<f32>> {
            let mut noise = FastNoiseLite::new();
            noise.set_noise_type(Some(NoiseType::OpenSimplex2));
            noise.set_seed(Some(20));
            noise.set_frequency(Some(0.015));
            
            let mut noise_data = vec![vec![0.; HEIGHT]; WIDTH];
           
            // Sample noise pixels
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    // Domain warp can optionally be employed to transform the coordinates before sampling:
                    // let (x, y) = noise.domain_warp_2d(x as f32, y as f32);
                    
                    let negative_1_to_1 = noise.get_noise_2d(x as f32, y as f32);
                    // You may want to remap the -1..1 range data to the 0..1 range:
                    noise_data[x][y] = (negative_1_to_1 + 1.) / 2.;                   
                }
            }
            noise_data
        }

        // create a ribbon mesh from the map
        pub fn create_mesh(&self, map :&Vec<Vec<Vec3>>) -> CpuMesh {
            let ribbon = create_ribbon(&map);
            ribbon.into()
        }
    }
    // Create and configure the FastNoise object



    pub struct Terrain {
        pub map: Rc<Map>,
        pub size: usize,            // nb of cells in the terrain edge
        pub length: f32,            // length of the terrain edge
        pub cpu_mesh: CpuMesh,
        pub cpu_material: CpuMaterial,
        pub mesh: Gm<Mesh, PhysicalMaterial>,
        pub position: Vec3,
        pub sub_tolerance: i32,   // how many cells flyable over by the camera on the terrain axis before trigger an update
        pub camera_pos: Vec3,
        delta_sub_x: i32,         // how many cells flought over thy the camera on the terrain x axis 
        delta_sub_z: i32,         // how many cells flought over thy the camera on the terrain x axis 
    }
    impl Terrain {
        pub fn new(context: &Context, map: Rc<Map>, size: usize, cpu_material: CpuMaterial) -> Self {
            let cpu_mesh: CpuMesh = Self::create_cpu_mesh(&map.coords, size);
            let ht = (size as f32 * 0.5) as usize;                      // half size of the terrain in quads
            let hm = (map.subdivisions as f32 * 0.5) as usize;          // half size of the map in quads
            let terrain_index = hm - ht;                                // index of the first quad of the terrain in the map
            let length = (map.coords[0][terrain_index + size - 1].x - map.coords[0][terrain_index].x).abs();    // length of the terrain edge
            let material = PhysicalMaterial::new_transparent(&context, &cpu_material);
            let mesh = Gm::new(Mesh::new(&context, &cpu_mesh), material);
            // initial terrain coordinates 
            let x = map.coords[terrain_index][terrain_index].x + length * 0.5;
            let z = map.coords[terrain_index][terrain_index].z + length * 0.5;
            let position = vec3(x, 0.0, z);
            // initial deltas of the terrain in the map
            let delta_nb_sub_x = (x - map.coords[0][0].x) / map.average_sub_size as f32;
            let delta_nb_sub_z = (z - map.coords[0][0].z) / map.average_sub_size as f32;
            let delta_sub_x = if delta_nb_sub_x > 0.0 { delta_nb_sub_x as i32 } else { delta_nb_sub_x as i32 + 1 };
            let delta_sub_z = if delta_nb_sub_z > 0.0 { delta_nb_sub_z as i32 } else { delta_nb_sub_z as i32 + 1 };
            println!("delta_sub_x: {}, delta_sub_z: {}", delta_sub_x, delta_sub_z);
            Terrain {
                map,
                size,
                length,
                cpu_mesh,
                cpu_material,
                mesh,
                position,
                sub_tolerance: 1,
                camera_pos: vec3(0.0, 0.0, 0.0),
                delta_sub_x,
                delta_sub_z,
            }
        }
        // create a terrain mesh
        pub fn create_cpu_mesh(coords: &Vec<Vec<Vec3>>, size: usize) -> CpuMesh {
            let ht = (size as f32 * 0.5) as usize;
            let hm = (coords.len() as f32 * 0.5) as usize;
            let start_index = hm - ht;
            let mut paths = Vec::new();
            for i in 0..size {
                let mut path = Vec::new();
                for j in 0..size {
                    let v3 = coords[start_index + i][start_index + j].clone();
                    path.push(v3);
                }
                paths.push(path);
            }
            let ribbon = create_ribbon(&paths);
            ribbon.into()
        }

        // https://github.com/BabylonJS/Extensions/blob/master/DynamicTerrain/src/babylon.dynamicTerrain.ts#L470
        pub fn update(&mut self, ) {
            let delta_x= self.position.x - self.camera_pos.x; 
            let delta_z= self.position.z - self.camera_pos.z;
            let threshold = self.map.average_sub_size * self.sub_tolerance as f32;  // threshold to trigger the terrain update in every direction x or z
            let mut needs_update = false;
            if delta_x.abs() > threshold {
                let map_flgt_nb_x: i32 = (delta_x / threshold) as i32;    // number (+/-) of map cells on the x axis flought over by the camera in the delta shift
                self.position.x  += threshold * map_flgt_nb_x as f32;
                self.delta_sub_x += map_flgt_nb_x * self.sub_tolerance;
                needs_update = true;
            } 
            if delta_z.abs() > threshold {
                let map_flgt_nb_z = (delta_z / threshold) as i32;    // number (+/-) of map cells on the z axis flought over by the camera in the delta shift
                self.position.z  += threshold * map_flgt_nb_z as f32;
                self.delta_sub_z += map_flgt_nb_z * self.sub_tolerance;
                needs_update = true;
            } 

            if needs_update {
                self.delta_sub_x = Self::modulo(self.delta_sub_x, self.map.subdivisions as i32);
                self.delta_sub_z = Self::modulo(self.delta_sub_z, self.map.subdivisions as i32);
                self.position.x = self.camera_pos.x;
                self.position.z = self.camera_pos.z;
                self.update_mesh();
            }

        }

        fn modulo(a: i32, b: i32) -> i32 {
            ((a % b)  + b) % b
        }

        pub fn update_mesh(&mut self) {
            let mut paths = vec!();
            for i in 0..self.size {
                let mut path = vec!();
                let map_i = Self::modulo(self.delta_sub_x + i as i32, self.map.subdivisions as i32);
                for j in 0..self.size {
                    let map_j = Self::modulo(self.delta_sub_z + j as i32, self.map.subdivisions as i32);
                    let v3 = self.map.coords[map_i as usize][map_j as usize];
                    path.push(v3);
                }
                paths.push(path);
            }
            morph_ribbon(&mut self.mesh.geometry, &mut &paths);
        }
    }

}
