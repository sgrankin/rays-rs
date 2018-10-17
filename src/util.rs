use cgmath::*;
use rand::*;
use std::cell::RefCell;

thread_local!(
    static THREAD_RNG_KEY: RefCell<rngs::SmallRng> = {
        RefCell::new(rngs::SmallRng::from_entropy())
    }
);

pub fn random<S: BaseFloat>() -> S {
    THREAD_RNG_KEY.with(|r| S::from(r.borrow_mut().gen::<f64>()).unwrap())
}

pub fn new_random(seed: u8) -> Box<FnMut() -> f64> {
    let mut rng = rngs::SmallRng::from_seed([seed; 16]);
    Box::new(move || rng.gen())
}

pub fn random_in_unit_sphere<S: BaseFloat>() -> Vector3<S> {
    loop {
        let p = Vector3::new(random(), random(), random()) * S::from(2).unwrap()
            - Vector3::from_value(S::one());
        if p.magnitude2() < S::one() {
            return p;
        }
    }
}

pub fn random_in_unit_disk<S: BaseFloat>() -> Vector2<S> {
    loop {
        let p = Vector2::new(random(), random()) * S::from(2).unwrap()
            - Vector2::new(S::one(), S::one());
        if p.magnitude2() < S::one() {
            return p;
        }
    }
}
