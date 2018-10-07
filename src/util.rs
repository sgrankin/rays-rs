use cgmath::*;
use rand::*;

pub fn random_in_unit_sphere<S: BaseFloat>() -> Vector3<S>
where
    distributions::Standard: distributions::Distribution<S>,
{
    loop {
        let p = Vector3::new(random::<S>(), random::<S>(), random::<S>()) * S::from(2).unwrap()
            - Vector3::from_value(S::one());
        if p.magnitude2() < S::one() {
            return p;
        }
    }
}
