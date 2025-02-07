use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Ray3d {
    pub origin: Vec3,
    pub dir: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

impl Ray3d {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.dir
    }
}

// create a ray (an origin and direction) from two points
pub fn ray_3d_from_points(p1: Vec3, p2: Vec3) -> Ray3d {
    let origin = p1;
    let dir = p2 - p1;

    Ray3d {
        origin,
        dir,
    }
}

// finds plane in the form of ax + by + cz = d from three points
pub fn plane_from_points(p1: Vec3, p2: Vec3, p3: Vec3) -> Plane {
    let v1 = p2 - p1;
    let v2 = p3 - p1;

    let n = v1.cross(v2);
    let d = n.x * p1.x + n.y * p1.y + n.z * p1.z;

    Plane {
        a: n.x,
        b: n.y,
        c: n.z,
        d,
    }
}

// calculate t value of intersection between a plane and a ray
pub fn ray_plane_intersect(ray: Ray3d, plane: Plane) -> f32 {
    let a = plane.a;
    let b = plane.b;
    let c = plane.c;
    let d = plane.d;

    let o = ray.origin;
    let r = ray.dir;

    (d - a * o.x - b * o.y - c * o.z) / (a * r.x + b * r.y + c * r.z)
}

