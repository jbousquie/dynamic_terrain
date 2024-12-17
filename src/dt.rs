pub mod terrain {
    use fastnoise_lite::*;

    use crate::ribbon::ribbon::*;
    use three_d::{vec3, Context, CpuMaterial, CpuMesh, Gm, Mesh, PhysicalMaterial, Vec3};

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
            let length = (coords[0][l - 1].x - coords[0][0].x).abs();
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
            let scl_y = 50.0;
            let scl_z = 5.0;
            let hw = WIDTH as f32 * 0.5;
            let hh = HEIGHT as f32 * 0.5;
            let noise_data = Self::create_noise();
            let mut paths = Vec::new();
            for x in 0..WIDTH {
               let mut path = Vec::new();
               for y in 0..HEIGHT {
                   path.push(vec3((x as f32 - hw) * scl_x, noise_data[x][y] * scl_y, (y as f32 - hh) * scl_z));
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



    pub struct Terrain<'a> {
        pub map: &'a Map,
        pub size: usize,            // nb of cells in the terrain edge
        pub length: f32,            // length of the terrain edge
        pub cpu_mesh: CpuMesh,
        pub cpu_material: CpuMaterial,
        pub mesh: Mesh,
        pub material: PhysicalMaterial,
        pub position: Vec3,
        pub sub_tolerance: i32,   // how many cells flyable over by the camera on the terrain axis before trigger an update
        camera_pos: Vec3,
        delta_sub_x: i32,         // how many cells flought over thy the camera on the terrain x axis 
        delta_sub_z: i32,         // how many cells flought over thy the camera on the terrain x axis 
    }
    impl<'a> Terrain<'a> {
        pub fn new(context: &Context, map: &'a Map, size: usize, cpu_material: CpuMaterial) -> Self {
            let cpu_mesh: CpuMesh = Self::create_mesh(&map.coords, size);
            let ht = (size as f32 * 0.5) as usize;
            let hm = (map.subdivisions as f32 * 0.5) as usize;
            let terrain_index = hm - ht;
            let length = (map.coords[0][terrain_index + size - 1].x - map.coords[0][terrain_index].x).abs();
            let material = PhysicalMaterial::new_opaque(&context, &cpu_material);
            let mesh = Gm::new(Mesh::new(&context, &cpu_mesh), &material).geometry;
            Terrain {
                map,
                size,
                length,
                cpu_mesh,
                cpu_material,
                mesh,
                material,
                position: vec3(0.0, 0.0, 0.0),
                sub_tolerance: 1,
                camera_pos: vec3(0.0, 0.0, 0.0),
                delta_sub_x: 0,
                delta_sub_z: 0,
            }
        }
        // create a terrain mesh
        pub fn create_mesh(coords :&Vec<Vec<Vec3>>, size: usize) -> CpuMesh {
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
            let map_shift = self.map.average_sub_size * self.sub_tolerance as f32;  // threshold to trigger the terrain update in every direction x or z
            let mut needs_update = false;
            if delta_x.abs() > map_shift {
                let map_flgt_nb_x: i32 = (delta_x / self.map.average_sub_size).abs() as i32;    // number (+/-) of map cells on the x axis flought over by the camera in the delta shift
                self.position.x  += map_shift * map_flgt_nb_x as f32;
                self.delta_sub_x += map_flgt_nb_x  * self.sub_tolerance;
                needs_update = true;
            } 
            if delta_z.abs() > map_shift {
                let map_flgt_nb_z = (delta_z / self.map.average_sub_size).abs() as i32;    // number (+/-) of map cells on the z axis flought over by the camera in the delta shift
                self.position.z  += map_shift * map_flgt_nb_z as f32;
                self.delta_sub_z += map_flgt_nb_z  * self.sub_tolerance;
                needs_update = true;
            } 

            if needs_update {
                self.delta_sub_x = Self::modulo(self.delta_sub_x, self.map.subdivisions as i32);
                self.delta_sub_z = Self::modulo(self.delta_sub_z, self.map.subdivisions as i32);
                self.update_mesh();
            }

        }

        fn modulo(a: i32, b: i32) -> i32 {
            ((a % b)  + b) % b
        }

        pub fn update_mesh(&mut self) {
            let map_index_x = Self::modulo(self.delta_sub_x, self.map.subdivisions as i32);
            let map_index_z = Self::modulo(self.delta_sub_z, self.map.subdivisions as i32);
            let mut paths = vec!();
            for i in 0..self.size {
                let mut path = vec!();
                for j in 0..self.size {
                    let v3 = self.map.coords[map_index_x as usize + i][map_index_z as usize + j];
                    path.push(v3);
                }
                paths.push(path);
            }
            morph_ribbon(&mut self.mesh, &mut &paths);
        }
    }





}
