pub mod terrain {
    use fastnoise_lite::*;

    use crate::ribbon::ribbon::*;
    use three_d::{vec3, CpuMesh};

    const WIDTH: usize = 512;
    const HEIGHT: usize = 512;
    
    // Create and configure the FastNoise object
    pub fn create_noise() -> [[f32; HEIGHT]; WIDTH] {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::OpenSimplex2));
        noise.set_seed(Some(2000));
        noise.set_frequency(Some(0.018));
        

        let mut noise_data = [[0.; HEIGHT]; WIDTH];
        
        // Sample noise pixels
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                // Domain warp can optionally be employed to transform the coordinates before sampling:
                //let (x1, y1) = noise.domain_warp_2d(x as f32, y as f32);
                
                let negative_1_to_1 = noise.get_noise_2d(x as f32, y as f32);
                // You may want to remap the -1..1 range data to the 0..1 range:
                noise_data[x][y] = (negative_1_to_1 + 1.) / 2.;
                
                // (Uses of `as f32` above should become `as f64` if you're using FNL with the "f64" feature flag)
            }
        }
        noise_data
    }

    pub fn create_terrain() -> CpuMesh {
        let noise_data = create_noise();
        let hh = HEIGHT as f32 * 0.5 ;
        let hw = WIDTH as f32 * 0.5;
        let mut paths = Vec::new();
        for x in 0..WIDTH {
           let mut path = Vec::new();
           for y in 0..HEIGHT {
               path.push(vec3(x as f32 - hw, noise_data[x][y] * 10.0, y as f32 - hh));
           }
           paths.push(path);
        }
        let ribbon = create_ribbon(&paths);
        ribbon.into()
    }
}
