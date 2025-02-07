use bevy::{prelude::*, pbr::wireframe::Wireframe};
use std::f32;
// Contains the GLTF mesh name for the collidable geometry
#[derive(Component)]
pub struct Collidable(pub Vec<String>);

// splits a mesh up into 8 smaller bounding boxes recursively
#[derive(Default, Component)]
pub struct RecursiveAABB {
    aabb: AABB,
    next: Option<Vec<RecursiveAABB>>,
    enclosed: Vec<usize>, // list of indices into the triangle buffer
}

pub const ALL_ENCOMPASSING_AABB: AABB = AABB {
    min: Vec3 { x: f32::MIN, y: f32::MIN, z: f32::MIN },
    max: Vec3 { x: f32::MAX, y: f32::MAX, z: f32::MAX }
};

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

const TRIANGLE_LIMIT: usize = 25;

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
            let triangles: Vec<Triangle3d> = assets.get(mesh).expect("Failed to retrieve mesh data.").triangles().expect("Failed to create list of triangles.").collect();

            let all_indices: Vec<usize> = (0..(triangles.len() - 1)).collect();
            let root = find_aabb(&triangles);
            let mut recursive_aabb = RecursiveAABB { aabb: root, next: None, enclosed: all_indices };
            divide_aabb(&mut recursive_aabb, TRIANGLE_LIMIT, &triangles, &mut commands, entity);
            commands.entity(entity).insert(recursive_aabb);
        }
    }
}

// spawn aabb entity -> calculate divided aabbs -> count vertices -> construct new -> repeat
// divide into 8ths, halve each dimension
fn divide_aabb(aabb: &mut RecursiveAABB, triangle_limit: usize, triangles: &[Triangle3d], commands: &mut Commands, parent: Entity) {
    let new_parent = commands.spawn((aabb.aabb, ShouldRenderCollider(false), Transform::default())).id();
    commands.entity(parent).add_child(new_parent);

    if aabb.enclosed.len() <= triangle_limit {
        commands.entity(new_parent).insert(ShouldRenderCollider(aabb.enclosed.len() != 0)); // only show ones that have vertices
        return;
    }

    let mut next_aabbs = [AABB::default(); 8];
    let mut next_encloseds = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()]; // lol Vec doesnt implement Copy

    let mut index = 0;
    let min = aabb.aabb.min;
    let max = aabb.aabb.max;
    // go thru each min of the next aabbs, constructing each one from that
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                // split the aabb into an 8th, 1/2 of each axis in one of the 8 possible locations, construct an aabb from that
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
                let next_enclosed = find_triangles_within_bound(&triangles, &aabb.enclosed, next_aabb_bound);

                next_aabbs[index] = next_aabb_bound;
                next_encloseds[index] = next_enclosed;
                index += 1;
            }
        }
    }

    let iter = next_aabbs.iter().zip(next_encloseds.iter()).map(
        |(aabb, enclosed)| RecursiveAABB { aabb: *aabb, next: None, enclosed: enclosed.clone() }
    );

    aabb.next = Some(iter.collect());

    for next in aabb.next.as_mut().unwrap().iter_mut() {
        divide_aabb(next, triangle_limit, triangles, commands, new_parent);
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

// finds the AABB of a mesh, constrained to only points described in 'indices'
fn find_aabb(triangles: &[Triangle3d]) -> AABB {
    let mut min = [f32::MAX; 3]; // set to max and min f32 to ensure no points are left unconsiderd.
    let mut max = [f32::MIN; 3];

    // find AABB with optional constraint
    for triangle in triangles {
        for v in triangle.vertices {
            for i in 0..3 { // do any of the three components of this vertex achieve a new high or low value?
                if v[i] > max[i] {
                    max[i] = v[i];
                }

                if v[i] < min[i] {
                    min[i] = v[i];
                }
            }
        }
    }

    AABB::new(min.into(), max.into())
}

// find all triangles contained in a certain aabb, only using triangles included in 'indices'
fn find_triangles_within_bound(triangles: &[Triangle3d], indices: &[usize], bound: AABB) -> Vec<usize> {
    let mut contained = Vec::new();

    // count triangles within it
    for (triangle, index) in indices.iter().map(|i| (&triangles[*i], i)) {
        // are any of the triangle vertices directly in the aabb?
        if
            point_in_aabb(triangle.vertices[0], bound) ||
            point_in_aabb(triangle.vertices[1], bound) ||
            point_in_aabb(triangle.vertices[2], bound)
        {
            contained.push(*index);
        } else { // second case, when a portion of the triangle passes though the aabb, with including a vertex
            // find vertices of the aabb, start at min and rotate clockwise at the bottom, then max and rotate clockwise
            let p1 = bound.min;
            let p2 = Vec3::new(bound.min.x, bound.min.y, bound.max.z);
            let p3 = Vec3::new(bound.max.x, bound.min.y, bound.max.z);
            let p4 = Vec3::new(bound.max.x, bound.min.y, bound.min.z);

            let p5 = bound.max;
            let p6 = Vec3::new(bound.max.x, bound.max.y, bound.min.z);
            let p7 = Vec3::new(bound.min.x, bound.max.y, bound.min.z);
            let p8 = Vec3::new(bound.min.x, bound.max.y, bound.max.z);

            // find whether edge of the aabb intersects the triangle
            // edges start with the bottom vertices connected, then the top vertices connected, then the top and bottom connected
            let e1 = line_intersects_triangle(p1, p2, triangle);
            let e2 = line_intersects_triangle(p2, p3, triangle);
            let e3 = line_intersects_triangle(p3, p4, triangle);
            let e4 = line_intersects_triangle(p4, p1, triangle);

            let e5 = line_intersects_triangle(p5, p6, triangle);
            let e6 = line_intersects_triangle(p6, p7, triangle);
            let e7 = line_intersects_triangle(p7, p8, triangle);
            let e8 = line_intersects_triangle(p8, p5, triangle);

            let e9 = line_intersects_triangle(p1, p7, triangle);
            let e10 = line_intersects_triangle(p2, p8, triangle);
            let e11 = line_intersects_triangle(p3, p5, triangle);
            let e12 = line_intersects_triangle(p4, p6, triangle);

            if e1 || e2 || e3 || e4 || e5 || e6 || e7 || e8 || e9 || e10 || e11 || e12 {
                contained.push(*index);
            }
        }
    }

    contained
}

fn point_in_aabb(point: Vec3, aabb: AABB) -> bool {
    let min = aabb.min;
    let max = aabb.max;

    let mut within = true;
    for i in 0..3 {
        if point[i] > max[i] {
            within = false;
            break;
        }

        if point[i] < min[i] {
            within = false;
            break;
        }
    }

    within
}

// returns whether the line between the two points intersects the triangle, where it does so, and "when" (t value) it does so
// only returns true if the intersection is BOTH in the triangle in between the two points;
fn line_intersects_triangle(p1: Vec3, p2: Vec3, triangle: &Triangle3d) -> bool {
    use crate::math;

    let ray = math::ray_3d_from_points(p1, p2);
    let plane = math::plane_from_points(triangle.vertices[0], triangle.vertices[1], triangle.vertices[2]);
    let t = math::ray_plane_intersect(ray, plane);
    let intersection = ray.at(t);
    let in_triangle = point_in_tri(intersection, triangle);
    let between_points = t >= 0.0 && t <= 1.0; // special property of rays

    in_triangle && between_points
}

// `p` is assumed to lie on the same plane as `tri`
pub fn point_in_tri(p: Vec3, tri: &Triangle3d) -> bool {
    let (a, b, c) = (tri.vertices[0], tri.vertices[1], tri.vertices[2]);
    let ab = (b - a).normalize();
    let ba = (a - b).normalize();
    let ac = (c - a).normalize();
    let bc = (c - b).normalize();

    let ap = (p - a).normalize();
    let bp = (p - b).normalize();

    // angle at point a and point b on our tri
    let theta_a = ab.dot(ac);
    let theta_b = ba.dot(bc);

    // angles between our point and the sides of our tri
    let theta_iab = ap.dot(ab);
    let theta_iac = ap.dot(ac);
    let theta_iba = bp.dot(ba);
    let theta_ibc = bp.dot(bc);

    // we invert the comparison becuase cos is, in some sense, proportional to the negative of the angle
    (theta_iab > theta_a && theta_iac > theta_a) && (theta_iba > theta_b && theta_ibc > theta_b)
}

#[test]
fn test_triangles_within_bound() {
    let triangles = [Triangle3d::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.0), Vec3::new(0.0, 1.0, 0.0))];
    let aabb = AABB {
        min: Vec3::new(-0.75, -0.75, -0.75),
        max: Vec3::new(0.75, 0.75, 0.75),
    };

    let indices = find_triangles_within_bound(&triangles, &[0], aabb);
    // assert_eq!(indices, Vec::<usize>::new());
    assert_eq!(indices, vec![0]);

    let triangles = [Triangle3d::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, -1.0, 0.0), Vec3::new(0.0, -1.0, 0.0))];
    let aabb = AABB {
        min: Vec3::new(-0.75, -0.75, -0.75),
        max: Vec3::new(0.75, 0.75, 0.75),
    };

    let indices = find_triangles_within_bound(&triangles, &[0], aabb);
    // assert_eq!(indices, Vec::<usize>::new());
    assert_eq!(indices, vec![0]);

    let triangles = [Triangle3d::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 0.0))];
    let aabb = AABB {
        min: Vec3::new(-0.75, -0.75, -0.75),
        max: Vec3::new(0.75, 0.75, 0.75),
    };

    let indices = find_triangles_within_bound(&triangles, &[0], aabb);
    // assert_eq!(indices, Vec::<usize>::new());
    assert_eq!(indices, vec![0]);

    let triangles = [Triangle3d::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, -1.0, 1.0), Vec3::new(0.0, -1.0, 0.0))];
    let aabb = AABB {
        min: Vec3::new(-0.75, -0.75, -0.75),
        max: Vec3::new(0.75, 0.75, 0.75),
    };

    let indices = find_triangles_within_bound(&triangles, &[0], aabb);
    // assert_eq!(indices, Vec::<usize>::new());
    assert_eq!(indices, vec![0]);

    let triangles = [Triangle3d::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0))];
    let aabb = AABB {
        min: Vec3::new(-0.75, -0.75, -0.75),
        max: Vec3::new(0.75, 0.75, 0.75),
    };

    let indices = find_triangles_within_bound(&triangles, &[0], aabb);
    // assert_eq!(indices, Vec::<usize>::new());
    assert_eq!(indices, vec![0]);

    let triangles = [Triangle3d::new(Vec3::new(-1.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0))];
    let aabb = AABB {
        min: Vec3::new(-0.75, -0.75, -0.75),
        max: Vec3::new(0.75, 0.75, 0.75),
    };

    let indices = find_triangles_within_bound(&triangles, &[0], aabb);
    // assert_eq!(indices, Vec::<usize>::new());
    assert_eq!(indices, vec![0]);
}