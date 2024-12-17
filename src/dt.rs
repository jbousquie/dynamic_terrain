pub mod terrain {
    use fastnoise_lite::*;

    use crate::ribbon::ribbon::*;
    use three_d::{vec3, CpuMesh, Vec3};

    // Number of points in the map
    // set the same value for both WIDTH and HEIGHT
    const WIDTH: usize = 500;
    const HEIGHT: usize = 500;

    pub struct Map {
        pub coords: Vec<Vec<Vec3>>,
    }
    impl Map {

        pub fn new() -> Self {
            let coords = Self::create_map();
            Map {
                coords,
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
            noise.set_seed(Some(200));
            noise.set_frequency(Some(0.05));
            
            let mut noise_data = vec![vec![0.; HEIGHT]; WIDTH];
            
            // Sample noise pixels
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    pub fn create_map() -> Vec<Vec<Vec3>> {
                        let scl_x = 5.0;
                        let scl_y = 50.0;
                        let scl_z = 5.0;
                        let hw = WIDTH as f32 * 0.5;
                        let hh = HEIGHT as f32 * 0.5;
                        let noise_data = Map::create_noise();
                        let mut paths = Vec::new();
                        for x in 0..WIDTH {
                           let mut path = Vec::new();
                           for y in 0..HEIGHT {
                               path.push(vec3((x as f32 - hw) * scl_x, noise_data[x][y] * scl_y, (y as f32 - hh) * scl_z));
                           }
                           paths.push(path);
                        }
                        paths
                    }   // Domain warp can optionally be employed to transform the coordinates before sampling:
                    //let (x1, y1) = noise.domain_warp_2d(x as f32, y as f32);
                    
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
        pub size: usize,
        pub mesh: CpuMesh,
    }
    impl<'a> Terrain<'a> {
        pub fn new(map: &'a Map, size: usize) -> Self {
            let mesh = Self::create_mesh(&map.coords, size);
            Terrain {
                map,
                size,
                mesh,
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

        
    }



    pub fn move_terrain(terrain: &mut CpuMesh, map: &Vec<Vec<Vec3>>, direction: Vec3, delta: f32) {
        let mut positions = terrain.positions.to_f32();


    }
}
