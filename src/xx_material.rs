use cgmath::*;
use crate::shapes::Contact;
use crate::shapes::*;
use crate::util::random_in_unit_sphere;
use rand;

impl<S: BaseFloat> Scatterrer<S> for Lambertian<S>
where
    rand::distributions::Standard: rand::distributions::Distribution<S>,
{
    fn scatter(&self, _: &Ray3<S>, contact: Contact<Point3<S>>) -> Option<(Vector3<S>, Ray3<S>)> {
        // Note we could just as well only scatter with some probability p and have attenuation be albedo/p.
        Some((
            self.albedo,
            Ray3 {
                origin: contact.point,
                direction: (contact.normal + random_in_unit_sphere()).normalize(),
            },
        ))
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Metal<S: BaseNum> {
    albedo: Vector3<S>,
}

impl<S: BaseFloat> Metal<S> {
    fn reflect(v: &Vector3<S>, norm: &Vector3<S>) -> Vector3<S> {
        (v - norm * v.dot(*norm) * S::from(2).unwrap()).normalize()
    }
}

impl<S: BaseFloat> Scatterrer<S> for Metal<S> {
    fn scatter(&self, r: &Ray3<S>, contact: Contact<Point3<S>>) -> Option<(Vector3<S>, Ray3<S>)> {
        let reflected = Self::reflect(&r.direction, &contact.normal);
        let scattered = Ray3 { origin: contact.point, direction: reflected.normalize() };
        if scattered.direction.dot(contact.normal) > S::zero() {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}
