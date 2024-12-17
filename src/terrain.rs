pub mod terrain {
    use fastnoise_lite::*;

    use crate::ribbon::ribbon::*;
    use three_d::{vec3, CpuMesh, Vec3};

    // Number of points in the map
    // set the same value for both WIDTH and HEIGHT
    const WIDTH: usize = 128;
    const HEIGHT: usize = 128;

    // Create and configure the FastNoise object
    pub fn create_noise() -> [[f32; HEIGHT]; WIDTH] {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::OpenSimplex2));
        noise.set_seed(Some(200));
        noise.set_frequency(Some(0.05));
        
        let mut noise_data = [[0.; HEIGHT]; WIDTH];
        
        // Sample noise pixels
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                // Domain warp can optionally be employed to transform the coordinates before sampling:
                //let (x1, y1) = noise.domain_warp_2d(x as f32, y as f32);
                
                let negative_1_to_1 = noise.get_noise_2d(x as f32, y as f32);
                // You may want to remap the -1..1 range data to the 0..1 range:
                noise_data[x][y] = (negative_1_to_1 + 1.) / 2.;                
            }
        }
        noise_data
    }

    pub fn create_map() -> Vec<Vec<Vec3>> {
        let scl_x = 5.0;
        let scl_y = 50.0;
        let scl_z = 5.0;
        let hw = WIDTH as f32 * 0.5;
        let hh = HEIGHT as f32 * 0.5;
        let noise_data = create_noise();
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

    pub fn create_map_terrain(map :&Vec<Vec<Vec3>>) -> CpuMesh {
        let ribbon = create_ribbon(&map);
        ribbon.into()
    }

    pub fn create_terrain(map :&Vec<Vec<Vec3>>, size: usize) -> CpuMesh {
        let ht = (size as f32 * 0.5) as usize;
        let hm = (map.len() as f32 * 0.5) as usize;
        let start_index = hm - ht;
        let mut paths = Vec::new();
        for i in 0..size {
            let mut path = Vec::new();
            for j in 0..size {
                let v3 = map[start_index + i][start_index + j].clone();
                path.push(v3);
            }
            paths.push(path);
        }
        let ribbon = create_ribbon(&paths);
        ribbon.into()
    }
}
