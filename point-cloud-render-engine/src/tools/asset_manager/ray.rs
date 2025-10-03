use bevy::prelude::*;

pub fn ray_hits_obb(origin: Vec3, dir: Vec3, xf: GlobalTransform, size: Vec3) -> Option<f32> {
    let inv = xf.compute_matrix().inverse();
    let o_local = inv.transform_point3(origin);
    let d_local = inv.transform_vector3(dir);
    let he = size * 0.5;
    ray_aabb_hit_t(o_local, d_local, -he, he)
}

// Slab-method ray–AABB intersection, returns Some(t) or None
pub fn ray_aabb_hit_t(ray_origin: Vec3, ray_direction: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let inv = Vec3::new(
        if ray_direction.x != 0.0 { 1.0 / ray_direction.x } else { f32::INFINITY },
        if ray_direction.y != 0.0 { 1.0 / ray_direction.y } else { f32::INFINITY },
        if ray_direction.z != 0.0 { 1.0 / ray_direction.z } else { f32::INFINITY },
    );

    let (mut tmin, mut tmax) = ((min.x - ray_origin.x) * inv.x, (max.x - ray_origin.x) * inv.x);
    if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }

    let (mut tymin, mut tymax) = ((min.y - ray_origin.y) * inv.y, (max.y - ray_origin.y) * inv.y);
    if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }

    if (tmin > tymax) || (tymin > tmax) { return None; }
    if tymin > tmin { tmin = tymin; }
    if tymax < tmax { tmax = tymax; }

    let (mut tzmin, mut tzmax) = ((min.z - ray_origin.z) * inv.z, (max.z - ray_origin.z) * inv.z);
    if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }

    if (tmin > tzmax) || (tzmin > tmax) { return None; }
    if tzmin > tmin { tmin = tzmin; }
    if tzmax < tmax { tmax = tzmax; }

    if tmax < 0.0 { return None; }
    Some(if tmin >= 0.0 { tmin } else { tmax })
}
