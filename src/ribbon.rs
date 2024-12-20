pub mod ribbon {

    use three_d::Indices;
    use three_d::renderer::geometry::Mesh;
    use three_d::Positions;
    use three_d::Vec2;
    use three_d_asset::TriMesh;
    use three_d::Vec3;
    use three_d::vec2;
    use three_d::InnerSpace;
    ///
    /// Returns a ribbon mesh from the passed parameter "paths"
    /// Paths is a vector of paths, where each path is a vector of Vec3.
    /// Each path is a list of successive points. Paths will be connected with triangles to make a surface or a volume.
    /// All the paths are required to have the same number of points.
    /// At leas        ///t two paths are required to create a ribbon.
    /// At least each path should have two points.
    /// 
    pub fn create_ribbon(paths: &Vec<Vec<Vec3>>, map_uvs: &Vec<Vec2>) -> TriMesh {
        // path lengths
        let p = paths.len();
        if p < 2 {
            panic!("At least two paths are required to create a ribbon");
        }
        let l = paths[0].len();
        if l < 2 {
            panic!("At least each path should have two points");
            
        }
        for i in 1..p {
            if paths[i].len() != l {
                panic!("All the paths are required to have the same number of points");
            }
        }

        // vertex data arrays
        let mut positions = Vec::new();
        let mut indices: Vec<u32> = Vec::new();             // indices with U32 because it could exceed 65535 on large meshes 
        // uvs coordinates are computed according to the distance between path points if the passed Vector is empty
        let mut map_uvs_given = true;
        let mut ribbon_uvs = Vec::new();
        if map_uvs.len() == 0 {
            map_uvs_given = false;
        }

        // variables for uv mapping
        let mut u_distances = vec![vec![0.0; l]; p]; // distance along the horizontal paths
        let mut v_distances = vec![vec![0.0; p]; l]; // distance along the vertical paths
        let mut u_total_distance = 0.0;
        let mut v_total_distance = 0.0;

        // positions
        for i in 0..p {
            u_total_distance = 0.0;
            for j in 0..l {
                let v3 = paths[i][j].clone();
                positions.push(v3);
                if map_uvs_given {
                    ribbon_uvs.push(map_uvs[i * l + j].clone());
                }
                else if j > 0 {
                    u_total_distance += (paths[i][j] - paths[i][j - 1]).magnitude();
                    u_distances[i][j] = u_total_distance;
                }
            }
        }


        // uvs
        // compute vertical distances for v values  
        if !map_uvs_given {
            for j in 0..l {
                v_total_distance = 0.0;
                for i in 0..p {
                    if i > 0 {
                        v_total_distance += (paths[i][j] - paths[i - 1][j]).magnitude();
                        v_distances[j][i] = v_total_distance;
                    }
                }
            }
            // compute uvs
            for i in 0..p {
                for j in 0..l {
                    let u = u_distances[i][j] / u_total_distance;
                    let v = v_distances[j][i] / v_total_distance;
                    ribbon_uvs.push(vec2(u,v));
                }
            }
        }
        
        // indices
        for i in 0..p - 1 {
            for j in 0..l - 1 {
                let i0 = i * l + j;
                let i1 = i * l + j + 1;
                let j0 = (i + 1) * l + j;
                let j1 = (i + 1) * l + j + 1;

                indices.push(i0 as u32);
                indices.push(i1 as u32);
                indices.push(j1 as u32);

                indices.push(j1 as u32);
                indices.push(j0 as u32);
                indices.push(i0 as u32);
            }
        }

        // TriMesh
        let mut mesh = TriMesh {
            positions: Positions::F32(positions),
            indices: Indices::U32(indices),
            uvs: Some(ribbon_uvs),
            ..Default::default()
        };
        mesh.compute_normals();
        mesh.compute_tangents();
        mesh
    }


    pub fn morph_ribbon(mesh: &mut Mesh, paths: &mut &Vec<Vec<Vec3>>, uvs: &Vec<Vec2>) {
        let mut positions = Vec::new();
        for i in 0..paths.len() {
            for j in 0..paths[i].len() {
                let v3 = paths[i][j].clone();
                positions.push(v3);
            }
        }
        let vb_pos = mesh.positions_mut();
        
        vb_pos.fill(&positions);
        if let Some(vb_uvs) = mesh.uvs_mut() {
            vb_uvs.fill(&uvs);
        }
    }

}
