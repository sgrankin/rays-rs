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
pub fn shuffle<T>(s: &mut [T]) {
    THREAD_RNG_KEY.with(|r| r.borrow_mut().shuffle(s))
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

pub fn stratified_samples(samples: usize) -> Vec<Vector2f> {
    let interval = 1.0 / samples as f64;
    let mut ys = Vec::with_capacity(samples);
    for i in 0..samples {
        ys.push((random() + i as f64) * interval);
    }
    shuffle(&mut ys);
    ys.iter().enumerate().map(|(i, y)| Vector2f::new(i as f64 * interval, *y)).collect()
}

pub fn stratified_samples_in_disk(samples: usize) -> Vec<Vector2f> {
    stratified_samples(samples)
        .iter()
        .map(|v| {
            let phi = v.x * PI * 2.0;
            let r = v.y.sqrt();
            Vector2f::new(r * f64::cos(phi), r * f64::sin(phi))
        }).collect()
}
