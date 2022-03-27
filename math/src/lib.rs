use glam::{Quat, Vec3A};

pub struct QuatExt {}

impl QuatExt {
    pub fn look_forward(mut forward: Vec3A, mut up: Vec3A) -> Quat {
        forward = forward.normalize();
        up = up.normalize();
        let right = up.cross(forward).normalize();

        let rx = right.x;
        let ry = right.y;
        let rz = right.z;

        let ux = up.x;
        let uy = up.y;
        let uz = up.z;

        let fx = forward.x;
        let fy = forward.y;
        let fz = forward.z;

        let sumRxUyFz = rx + uy + fz;

        if sumRxUyFz > 0.0 {
            let scl = f32::sqrt(sumRxUyFz + 1.0);
            let w = scl * 0.5;
            let scl_inv = 0.5 / scl;
            let x = (uz - fy) * scl_inv;
            let y = (fx - rz) * scl_inv;
            let z = (ry - ux) * scl_inv;
            return Quat::from_xyzw(x, y, z, w);
        }
        if rx >= uy && rx >= fz {
            let scl = f32::sqrt(1.0 + rx - uy - fz);
            let scl_inv = 0.5 / scl;
            let w = (uz - fy) * scl_inv;
            let x = scl * 0.5;
            let y = (ry + ux) * scl_inv;
            let z = (fx + rz) * scl_inv;
            return Quat::from_xyzw(x, y, z, w);
        }
        if uy >= fz {
            let scl = f32::sqrt(1.0 + uy - rx - fz);
            let scl_inv = 0.5 / scl;
            let w = (fx - rz) * scl_inv;
            let x = (ry + ux) * scl_inv;
            let y = scl * 0.5;
            let z = (uz + fy) * scl_inv;
            return Quat::from_xyzw(x, y, z, w);
        }
        let scl = f32::sqrt(1.0 + fz - rx - uy);
        let scl_inv = 0.5 / scl;
        let w = (ry - ux) * scl_inv;
        let x = (fx + rz) * scl_inv;
        let y = (uz + fy) * scl_inv;
        let z = scl * 0.5;
        return Quat::from_xyzw(x, y, z, w);
    }
}