use super::geom::*;
use super::util::*;
use cgmath::*;

pub trait Material<S: BaseFloat> {
    fn scatter(
        &self,
        in_: Ray3<S>,
        point: Point3<S>,
        normal: Vector3<S>,
    ) -> Option<(Ray3<S>, Vector3<S>)>;
}

#[derive(Copy, Clone, PartialEq)]
pub struct Lambertian<S: BaseFloat> {
    pub albedo: Vector3<S>,
}

impl<S: BaseFloat + 'static> Material<S> for Lambertian<S> {
    fn scatter(
        &self,
        _in_: Ray3<S>,
        point: Point3<S>,
        normal: Vector3<S>,
    ) -> Option<(Ray3<S>, Vector3<S>)> {
        // Note we could just as well only scatter with some probability p and have attenuation be albedo/p.
        let ray = Ray3 { origin: point, direction: (normal + random_in_unit_sphere()).normalize() };
        Some((ray, self.albedo))
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Metal<S: BaseNum> {
    pub albedo: Vector3<S>,
    pub fuzz: S,
}

fn reflect<S: BaseFloat>(v: Vector3<S>, norm: Vector3<S>) -> Vector3<S> {
    (v - norm * v.dot(norm) * S::from(2).unwrap()).normalize()
}

impl<S: BaseFloat + 'static> Material<S> for Metal<S> {
    fn scatter(
        &self,
        in_: Ray3<S>,
        point: Point3<S>,
        normal: Vector3<S>,
    ) -> Option<(Ray3<S>, Vector3<S>)> {
        let reflected = reflect(in_.direction, normal);
        let scattered = Ray3 {
            origin: point,
            direction: (reflected + random_in_unit_sphere() * self.fuzz).normalize(),
        };
        if scattered.direction.dot(normal) > S::zero() {
            Some((scattered, self.albedo))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Dielectric<S: BaseNum> {
    pub ref_index: S,
}

fn refract<S: BaseFloat>(v: Vector3<S>, norm: Vector3<S>, ni_over_nt: S) -> Option<Vector3<S>> {
    let uv = v.normalize();
    let dt = uv.dot(norm);
    let discriminant = S::one() - ni_over_nt * ni_over_nt * (S::one() - dt * dt);
    if discriminant > S::zero() {
        Some((uv - norm * dt) * ni_over_nt - norm * discriminant.sqrt())
    } else {
        None
    }
}
fn shlick<S: BaseFloat>(cosine: S, ref_idx: S) -> S {
    let r0 = (S::one() - ref_idx) / (S::one() + ref_idx);
    let r0 = r0 * r0;
    r0 + (S::one() - r0) * (S::one() - cosine).powf(S::from(5.0).unwrap())
}

impl<S: BaseFloat + 'static> Material<S> for Dielectric<S> {
    fn scatter(
        &self,
        in_: Ray3<S>,
        point: Point3<S>,
        normal: Vector3<S>,
    ) -> Option<(Ray3<S>, Vector3<S>)> {
        let reflected = reflect(in_.direction, normal);
        let (outward_normal, ni_over_nt, cosine) = if in_.direction.dot(normal) > S::zero() {
            (
                -normal,
                self.ref_index,
                self.ref_index * in_.direction.dot(normal) / in_.direction.magnitude(),
            )
        } else {
            (
                normal,
                S::one() / self.ref_index,
                -in_.direction.dot(normal) / in_.direction.magnitude(),
            )
        };
        let attenuation = Vector3::from_value(S::one());
        let reflect_p = shlick(cosine, self.ref_index);
        let out = match refract(in_.direction, outward_normal, ni_over_nt) {
            Some(refracted) if random::<S>() >= reflect_p => refracted,
            _ => reflected,
        };
        Some((Ray3::new(point, out), attenuation))
    }
}
