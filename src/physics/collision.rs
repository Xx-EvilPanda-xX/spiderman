use bevy::{prelude::*, scene::SceneInstance, render::{mesh::{VertexAttributeValues, Indices}, render_resource::PrimitiveTopology}, pbr::wireframe::Wireframe, utils::tracing::field::debug, asset::RenderAssetUsages};

// Contains the GLTF mesh name for the collidable geometry
#[derive(Component)]
pub struct Collidable(pub Vec<String>);

// splits a mesh up into 8 smaller bounding boxes recursively
#[derive(Default)]
pub struct RecursiveAABB {
    aabb: AABB,
    next: Option<Vec<RecursiveAABB>>,
    enclosed_indices: usize,
}

#[derive(Default, Clone, Copy, Debug, Component)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max,
        }
    }
}

#[derive(Component, Debug)]
pub struct ShouldRenderCollider(bool);

const VERTEX_LIMIT: usize = 25;

// IMPORTANT: under the hood asset server spawns child entities for both the meshes and the nodes of the object, both of which have a Name component.
// NODES ARE PARENTS OF MESHES
// THERE IS ONE MORE ROOT NODE THAT IS A CHILD TO THE SCENEROOT, WHICH HAS THE NODES AS CHILDREN
//
// SceneRoot -- RootNode -- Node(s) -- Mesh(s)
//
pub fn construct_collision_trees(
    meshes: Query<(Entity, &Parent, &Mesh3d, &Name), Added<Mesh3d>>, // filtered for only new arrivals of 'Mesh3d' 
    all_parents: Query<&Parent>, // filter doesn't matter, we just need pointers traverse up the heirarchy
    scenes: Query<&Collidable>,
    assets: Res<Assets<Mesh>>,
    mut commands: Commands,
) {

    for (entity, node_id, mesh, name) in meshes.iter() {

        let collidable_mesh_names = &scenes.get(
            **all_parents.get(
                **all_parents.get(**node_id).unwrap()
            ).unwrap()
        ).unwrap().0;

        if collidable_mesh_names.contains(&String::from(name.as_str())) { // does the list of collidable meshes in the scene contain the mesh in question?
            if let Some(VertexAttributeValues::Float32x3(mesh_data)) = assets.get(mesh).expect("Failed to retrieve mesh data").attribute(Mesh::ATTRIBUTE_POSITION) {
                let (root, total_verts) = find_aabb_bounded(&mesh_data, None);
                let mut recursive_aabb = RecursiveAABB { aabb: root, next: None, enclosed_indices: total_verts };
                divide_aabb(&mut recursive_aabb, 2, &mesh_data, &mut commands, entity);
            }
        }
    }
}

// spawn aabb entity -> calculate divided aabbs -> count vertices -> construct new -> repeat
// divide into 8ths, halve each dimension
fn divide_aabb(aabb: &mut RecursiveAABB, vertex_limit: usize, mesh_data: &[[f32; 3]], commands: &mut Commands, parent: Entity) {
    let new_parent = commands.spawn((aabb.aabb, ShouldRenderCollider(false), Transform::default())).id(); // oh shit id doesn work
    commands.entity(parent).add_child(new_parent);

    // if aabb.enclosed_indices <= vertex_limit {
    //     return;
    // }

    if vertex_limit == 0 {
        commands.entity(new_parent).insert(ShouldRenderCollider(aabb.enclosed_indices != 0));
        return;
    }

    let mut next_aabbs = [(AABB::default(), 0); 8];
    let mut index = 0;
    let min = aabb.aabb.min;
    let max = aabb.aabb.max;
    // go thru each min of the next aabbs, constructing each one from that
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let i = i as f32;
                let j = j as f32;
                let k = k as f32;

                let x_half = (max.x - min.x) / 2.0;
                let y_half = (max.y - min.y) / 2.0;
                let z_half = (max.z - min.z) / 2.0;

                let min_x = min.x + k * x_half;
                let min_y = min.y + j * y_half;
                let min_z = min.z + i * z_half;

                let max_x = min_x + x_half;
                let max_y = min_y + y_half;
                let max_z = min_z + z_half;

                let next_aabb_bound = AABB::new(Vec3::new(min_x, min_y, min_z), Vec3::new(max_x, max_y, max_z));
                next_aabbs[index] = find_aabb_bounded(mesh_data, Some(next_aabb_bound));
                index += 1;
            }
        }
    }

    let iter = next_aabbs.iter().map(
        |(aabb, vertex_count)| RecursiveAABB { aabb: *aabb, next: None, enclosed_indices: *vertex_count }
    );

    aabb.next = Some(iter.collect());

    for next in aabb.next.as_mut().unwrap().iter_mut() {
        divide_aabb(next, vertex_limit - 1, mesh_data, commands, new_parent);
    }
}

// build the mesh for an AABB. an AABB is its own Entity, as created by 'divide_aabb'
pub fn add_collider_wireframes(
    meshes: Query<(Entity, &AABB, &ShouldRenderCollider), Added<AABB>>,
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    for (entity, aabb, should_render_collider) in meshes.iter() {
        let center = Vec3::new(
            (aabb.min.x + aabb.max.x) / 2.0,
            (aabb.min.y + aabb.max.y) / 2.0,
            (aabb.min.z + aabb.max.z) / 2.0,
        );

        if should_render_collider.0 {
            commands.entity(entity).insert((
                Mesh3d(mesh_assets.add(Cuboid::from_corners(aabb.min, aabb.max))), // HOLY FUCK CUBOID DOCS ARE ASS
                Transform::from_translation(center), // Offset the fucky cubiod bs
                Wireframe,
            ));
        }
    }
}

// finds the AABB of a mesh and number of vertices included, optionally constrained by the sub bound 'bound'
fn find_aabb_bounded(mesh_data: &[[f32; 3]], bound: Option<AABB>) -> (AABB, usize) {
    let mut min = [0.0; 3];
    let mut max = [0.0; 3];

    let bound_unwrapped = bound.clone().unwrap_or(AABB::default());
    let bound_min: [f32; 3] = bound_unwrapped.min.into();
    let bound_max: [f32; 3] = bound_unwrapped.max.into();

    // find AABB with optional constraint
    for vertex in mesh_data.iter() {
        if bound.is_some() {
            for i in 0..3 {
                if vertex[i] > max[i] && vertex[i] <= bound_max[i] {
                    max[i] = vertex[i];
                }

                if vertex[i] < min[i] && vertex[i] >= bound_min[i] {
                    min[i] = vertex[i];
                }
            }
        } else {
            for i in 0..3 {
                if vertex[i] > max[i] {
                    max[i] = vertex[i];
                }

                if vertex[i] < min[i] {
                    min[i] = vertex[i];
                }
            }
        }
    }

    let mut num_contained = 0;

    // count vertices within it
    for vertex in mesh_data.iter() {
        let mut contained = true;
        for i in 0..3 {
            if vertex[i] > max[i] {
                contained = false;
                break;
            }

            if vertex[i] < min[i] {
                contained = false;
                break;
            }
        }

        if contained { num_contained += 1 }
    }

    debug!("{:?}", AABB::new(min.into(), max.into()));
    (AABB::new(min.into(), max.into()), num_contained)
}