use bevy::{prelude::*, scene::SceneInstance, core_pipeline::motion_blur::node};

#[derive(Component)]

// Contains the GLTF mesh name for the collidable geometry
pub struct Collidable(pub String);

// splits a mesh up into 8 smaller bounding boxes recursively
pub struct MeshColliderTree {
    boxes: [RecursiveAABB; 8],
}

pub struct RecursiveAABB {
    p1: Vec3,
    p2: Vec3,
    next: Option<Box<[RecursiveAABB; 8]>>,
    enclosed_indices: usize,
}

// IMPORTANT: under the hood asset server spawns child entities for both the meshes and the nodes of the object, both of which have a Name component.
// NODES ARE PARENTS OF MESHES
// THERE IS ONE MORE ROOT NODE THAT IS A CHILD TO THE SCENEROOT, WHICH HAS THE NODES AS CHILDREN
//
// SceneRoot -- RootNode -- Node(s) -- Mesh(s)
//
pub fn construct_collision_trees(
    meshes: Query<(&Parent, &Mesh3d, &Name), Added<Mesh3d>>, // filtered for only new arrivals of 'Mesh3d' 
    all_parents: Query<&Parent>, // filter doesn't matter, we just need pointers traverse up the heirarchy
    scenes: Query<&Collidable>,
    assets: Res<Assets<Mesh>>,
) {

    for (node_id, mesh, name) in meshes.iter() {

        let collidable_mesh_name = &scenes.get(
            **all_parents.get(
                **all_parents.get(**node_id).unwrap()
            ).unwrap()
        ).unwrap().0;

        if name.as_str() == collidable_mesh_name {
            debug!("PPPPPPPPPPPPPPPP {:?}", mesh); // YESSSSSSSSSSSSSSS FINALLY
        }
    }
}