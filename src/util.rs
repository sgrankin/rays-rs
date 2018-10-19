use rand::*;
use std::cell::RefCell;

use crate::types::*;

thread_local!(
    static THREAD_RNG_KEY: RefCell<rngs::SmallRng> = {
        RefCell::new(rngs::SmallRng::from_entropy())
    }
);

pub fn random() -> Float {
    THREAD_RNG_KEY.with(|r| Float::from(r.borrow_mut().gen::<f64>()))
}

pub fn new_random(seed: u8) -> Box<FnMut() -> f64> {
    let mut rng = rngs::SmallRng::from_seed([seed; 16]);
    Box::new(move || rng.gen())
}

pub fn random_in_unit_sphere() -> Vector3f {
    loop {
        let p = Vector3f::new(random(), random(), random()) * 2.0 - Vector3f::from_value(1.0);
        if p.magnitude2() < 1.0 {
            return p;
        }
    }
}

pub fn random_in_unit_disk() -> Vector2f {
    loop {
        let p = Vector2f::new(random(), random()) * 2.0 - Vector2f::from_value(1.0);
        if p.magnitude2() < 1.0 {
            return p;
        }
    }
}
