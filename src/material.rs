use super::geom::*;
use super::util::*;
use crate::types::*;

pub trait Material {
    fn scatter(&self, in_: Ray3f, point: Point3f, normal: Vector3f) -> Option<(Ray3f, Vector3f)>;
}

#[derive(Copy, Clone, PartialEq)]
pub struct Lambertian {
    pub albedo: Vector3f,
}

impl Material for Lambertian {
    fn scatter(&self, _in_: Ray3f, point: Point3f, normal: Vector3f) -> Option<(Ray3f, Vector3f)> {
        // Note we could just as well only scatter with some probability p and have attenuation be albedo/p.
        let ray =
            Ray3f { origin: point, direction: (normal + random_in_unit_sphere()).normalize() };
        Some((ray, self.albedo))
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Metal {
    pub albedo: Vector3f,
    pub fuzz: Float,
}

fn reflect(v: Vector3f, norm: Vector3f) -> Vector3f {
    (v - norm * v.dot(norm) * 2.0).normalize()
}

impl Material for Metal {
    fn scatter(&self, in_: Ray3f, point: Point3f, normal: Vector3f) -> Option<(Ray3f, Vector3f)> {
        let reflected = reflect(in_.direction, normal);
        let scattered = Ray3f {
            origin: point,
            direction: (reflected + random_in_unit_sphere() * self.fuzz).normalize(),
        };
        if scattered.direction.dot(normal) > 0.0 {
            Some((scattered, self.albedo))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Dielectric {
    pub ref_index: Float,
}

fn refract(v: Vector3f, norm: Vector3f, ni_over_nt: Float) -> Option<Vector3f> {
    let uv = v.normalize();
    let dt = uv.dot(norm);
    let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);
    if discriminant > 0.0 {
        Some((uv - norm * dt) * ni_over_nt - norm * discriminant.sqrt())
    } else {
        None
    }
}
fn shlick(cosine: Float, ref_idx: Float) -> Float {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

impl Material for Dielectric {
    fn scatter(&self, in_: Ray3f, point: Point3f, normal: Vector3f) -> Option<(Ray3f, Vector3f)> {
        let reflected = reflect(in_.direction, normal);
        let (outward_normal, ni_over_nt, cosine) = if in_.direction.dot(normal) > 0.0 {
            (
                -normal,
                self.ref_index,
                self.ref_index * in_.direction.dot(normal) / in_.direction.magnitude(),
            )
        } else {
            (normal, 1.0 / self.ref_index, -in_.direction.dot(normal) / in_.direction.magnitude())
        };
        let attenuation = Vector3f::from_value(1.0);
        let reflect_p = shlick(cosine, self.ref_index);
        let out = match refract(in_.direction, outward_normal, ni_over_nt) {
            Some(refracted) if random() >= reflect_p => refracted,
            _ => reflected,
        };
        Some((Ray3f::new(point, out), attenuation))
    }
}
