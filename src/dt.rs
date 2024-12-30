pub mod terrain {
    use fastnoise_lite::*;

    use crate::ribbon::ribbon::*;
    use three_d::{vec2, vec3, Context, CpuMaterial, CpuMesh, Gm, Mesh, PhysicalMaterial, Vec2, Vec3};
    use std::rc::Rc;
    use image::ImageReader;

    // Number of points in the map
    // set the same value for both WIDTH and HEIGHT
    const WIDTH: usize = 1280;
    const HEIGHT: usize = 1280;

    pub struct Map {
        pub coords: Vec<Vec<Vec3>>,
        pub uvs: Vec<Vec2>,
        pub length: f32,
        pub subdivisions: usize,
        pub average_sub_size: f32

    }
    impl Map {

        pub fn new() -> Self {
            //let (coords, uvs) = Self::create_map();
            let (coords, uvs) = Self::create_heightmap_from_file("assets/worldHeightMapDouble.png", 5.0, 80.0);
            let l = coords.len();
            let length = (coords[0][l - 1].x - coords[0][0].x).abs();
            let average_sub_size = length / l as f32;
            Map {
                coords,
                uvs,
                length,
                subdivisions: l,
                average_sub_size
            }
        }
        
        pub fn create_map() -> (Vec<Vec<Vec3>>, Vec<Vec2>) {
            let scl_x = 5.0;
            let scl_y: f32 = 100.0;
            let scl_z = scl_x;
            let hw = WIDTH as f32 * 0.5;
            let hh = HEIGHT as f32 * 0.5;
            let noise_data = Self::create_noise();
            let mut paths = Vec::new();
            for j in 0..WIDTH {
               let mut path = Vec::new();
               for i in 0..HEIGHT {
                    let x = (i as f32 - hw) * scl_x;
                    let y = noise_data[j][i] * scl_y * ((i as f32 + j as f32) * 0.01).sin();
                    let z = (j as f32 - hh) * scl_z;
                   path.push(vec3(x, y, z));
               }
               paths.push(path);
            }
            let uvs = Vec::new();
            (paths, uvs)
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
        pub fn create_mesh(&self, coords: &Vec<Vec<Vec3>>, uvs: &Vec<Vec2>) -> CpuMesh {
            let ribbon = create_ribbon(&coords, &uvs);
            ribbon.into()
        }


        // idée : stocker les données dans un fichier 
        // https://docs.rs/image/latest/image/type.RgbImage.html
        pub fn create_heightmap_from_file(file: &str, offset: f32, altitude_factor: f32) -> (Vec<Vec<Vec3>>, Vec<Vec2>) {
            let filter_r = 0.3;
            let filter_g = 0.59;
            let filter_b = 0.11;
            let dyn_img = ImageReader::open(file).unwrap().decode().unwrap();
            let img = dyn_img.into_rgb8();
            let half_width = img.width() as f32 * 0.5;
            let half_height = img.height() as f32 * 0.5;
            let mut data = Vec::new();
            let mut uvs = Vec::new();
            img.enumerate_rows().for_each(|(j, row)| {
                let mut path = Vec::new();
                row.enumerate().for_each(|(i, pixel)| {
                    let rgb = pixel.2   ;
                    let r = rgb[0] as f32;
                    let g = rgb[1] as f32;  
                    let b = rgb[2] as f32;
                    let gradient = (r * filter_r + g * filter_g + b * filter_b) / 255.0;
                    let altitude = gradient * altitude_factor;
                    let x = (i as f32 - half_width) * offset;
                    let z = (j as f32 - half_height) * offset;
                    let v3 = vec3(x, altitude, z);
                    let u = i as f32 / img.width() as f32;
                    let v = 1.0 - j as f32 / img.height() as f32;
                    let uv = vec2(u, v);
                    path.push(v3);
                    uvs.push(uv);
                });
                data.push(path);
            });

            
            (data, uvs)
        }
    }



    pub struct Terrain {
        pub map: Rc<Map>,
        pub size: usize,            // nb of cells in the terrain edge
        pub length: f32,            // length of the terrain edge
        pub cpu_mesh: CpuMesh,
        pub cpu_material: CpuMaterial,
        pub mesh: Gm<Mesh, PhysicalMaterial>,
        pub paths: Vec<Vec<Vec3>>,
        pub uvs: Vec<Vec2>,
        pub position: Vec3,       // mesh logical coordinates
        pub sub_tolerance: i32,   // how many cells flyable over by the camera on the terrain axis before trigger an update
        pub camera_pos: Vec3,
        delta_sub_x: i32,         // how many cells flought over thy the camera on the terrain x axis 
        delta_sub_z: i32,         // how many cells flought over thy the camera on the terrain x axis 
    }
    impl Terrain {
        pub fn new(context: &Context, map: Rc<Map>, size: usize, cpu_material: CpuMaterial) -> Self {
            let (cpu_mesh, paths, uvs) = Self::create_cpu_mesh(&map.coords, &map.uvs, size);
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
            Terrain {
                map,
                size,
                length,
                cpu_mesh,
                cpu_material,
                mesh,
                paths,
                uvs,
                position,
                sub_tolerance: 1,
                camera_pos: Vec3::new(0.0, 0.0, 0.0),
                delta_sub_x,
                delta_sub_z,
            }
        }
        // create a terrain mesh
        pub fn create_cpu_mesh(coords: &Vec<Vec<Vec3>>, map_uvs: &Vec<Vec2>, size: usize) -> (CpuMesh, Vec<Vec<Vec3>>, Vec<Vec2>) {
            let ht = (size as f32 * 0.5) as usize;
            let hm = (coords.len() as f32 * 0.5) as usize;
            let start_index = hm - ht;
            let nb_vertices = size + 1;
            let mut paths = Vec::new();
            let mut uvs = Vec::new();
            let l = map_uvs.len();
            for i in 0..nb_vertices{
                let mut path = Vec::new();
                for j in 0..nb_vertices {
                    let v3 = coords[start_index + i][start_index + j].clone();
                    path.push(v3);
                    if l > 0 {
                        let uv = map_uvs[(start_index + j) * nb_vertices + start_index + i].clone();
                        uvs.push(uv);
                    }
                }
                paths.push(path);
            }
            let ribbon = create_ribbon(&paths, &uvs);
            (ribbon.into(), paths, uvs)
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
            let nb_vertices = self.size + 1;
            for i in 0..nb_vertices {
                let map_i = Self::modulo(self.delta_sub_z + i as i32, self.map.subdivisions as i32);
                for j in 0..nb_vertices {
                    let map_j = Self::modulo(self.delta_sub_x + j as i32, self.map.subdivisions as i32);
                    let v3 = self.map.coords[map_i as usize][map_j as usize];
                    self.paths[i][j].y = v3.y;
                    self.uvs[i * nb_vertices + j].x = self.map.uvs[map_i as usize * self.map.subdivisions + map_j as usize].x;
                    self.uvs[i * nb_vertices + j].y = self.map.uvs[map_i as usize * self.map.subdivisions + map_j as usize].y;
                }
            }
            morph_ribbon(&mut self.mesh.geometry, &mut &self.paths, &self.uvs);
        }
    }

}
