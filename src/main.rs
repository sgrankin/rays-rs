use std::alloc::System;
#[global_allocator]
static GLOBAL: System = System;

use log::*;
use simple_logger;

use cgmath::*;
use image;
use rand::*;
use rayon::prelude::*;
use std::error::Error;
use std::ops::*;

mod util;

mod geom {
    use cgmath::*;

    #[derive(Copy, Clone, PartialEq, Debug)]
    pub struct Ray<S, P, V>
    where
        S: BaseNum,
        V: VectorSpace<Scalar = S>,
        P: EuclideanSpace<Scalar = S, Diff = V>,
    {
        pub origin: P,
        pub direction: V,
    }
    pub type Ray3<S> = Ray<S, Point3<S>, Vector3<S>>;

    pub trait Shape<S: BaseFloat> {
        fn intersect(&self, _: &Ray3<S>) -> Option<(Point3<S>, Vector3<S>)>;
    }

    pub struct Sphere<S: BaseFloat> {
        pub center: Point3<S>,
        pub radius: S,
    }

    impl<S: BaseFloat> Shape<S> for Sphere<S> {
        fn intersect(&self, r: &Ray3<S>) -> Option<(Point3<S>, Vector3<S>)> {
            // TODO credit collision crate for the source of the formula; rewrite for readability: this can be explained via geometry, but needs better names and a block comment.
            let s = self;
            let l = s.center - r.origin;
            let tca = l.dot(r.direction);
            if tca < S::zero() {
                return None;
            }
            let d2 = l.dot(l) - tca * tca;
            if d2 > s.radius * s.radius {
                return None;
            }
            let thc = (s.radius * s.radius - d2).sqrt();
            let p = r.origin + r.direction * (tca - thc);
            Some((p, (p - s.center).normalize()))
        }
    }
}

mod material {
    use super::geom::*;
    use super::util;
    use cgmath::*;
    use rand;

    type PixelXform<S /*: BaseFloat*/> = Box<dyn Fn(Vector3<S>) -> Vector3<S>>;

    pub trait Material<S: BaseFloat> {
        fn scatter(
            &self,
            in_: &Ray3<S>,
            point: &Point3<S>,
            normal: &Vector3<S>,
        ) -> Option<(Ray3<S>, PixelXform<S>)>;
    }

    #[derive(Copy, Clone, PartialEq)]
    pub struct Lambertian<S: BaseFloat> {
        pub albedo: Vector3<S>,
    }

    fn attenuate<S: BaseFloat + 'static>(albedo: Vector3<S>) -> PixelXform<S> {
        Box::new(move |pixel| {
            Vector3::new(pixel.x * albedo.x, pixel.y * albedo.y, pixel.z * albedo.z)
        })
    }

    impl<S: BaseFloat + 'static> Material<S> for Lambertian<S>
    where
        rand::distributions::Standard: rand::distributions::Distribution<S>,
    {
        fn scatter(
            &self,
            _in_: &Ray3<S>,
            point: &Point3<S>,
            normal: &Vector3<S>,
        ) -> Option<(Ray3<S>, PixelXform<S>)> {
            // Note we could just as well only scatter with some probability p and have attenuation be albedo/p.

            let ray = Ray3 {
                origin: *point,
                direction: (normal + util::random_in_unit_sphere()).normalize(),
            };
            Some((ray, attenuate(self.albedo)))
        }
    }

    #[derive(Copy, Clone, PartialEq, Debug)]
    pub struct Metal<S: BaseNum> {
        pub albedo: Vector3<S>,
        pub fuzz: S,
    }

    impl<S: BaseFloat> Metal<S> {
        fn reflect(v: &Vector3<S>, norm: &Vector3<S>) -> Vector3<S> {
            (v - norm * v.dot(*norm) * S::from(2).unwrap()).normalize()
        }
    }

    impl<S: BaseFloat + 'static> Material<S> for Metal<S>
    where
        rand::distributions::Standard: rand::distributions::Distribution<S>,
    {
        fn scatter(
            &self,
            in_: &Ray3<S>,
            point: &Point3<S>,
            normal: &Vector3<S>,
        ) -> Option<(Ray3<S>, PixelXform<S>)> {
            let reflected = Self::reflect(&in_.direction, &normal);
            let scattered = Ray3 {
                origin: *point,
                direction: (reflected + util::random_in_unit_sphere() * self.fuzz).normalize(),
            };
            if scattered.direction.dot(*normal) > S::zero() {
                Some((scattered, attenuate(self.albedo)))
            } else {
                None
            }
        }
    }

}

mod prims {
    use super::geom::*;
    use super::material::*;

    use cgmath::*;
    use std::marker::PhantomData;

    pub trait Primitive<S: BaseFloat> {
        fn intersect(&self, _: &Ray3<S>) -> Option<SurfaceInteraction<'_, S>>;
    }

    pub struct SurfaceInteraction<'a, S: BaseFloat + 'a> {
        pub prim: &'a dyn Primitive<S>,
        pub point: Point3<S>,
        pub normal: Vector3<S>,
        pub material: &'a dyn Material<S>,
    }

    pub struct ShapePrimitive<S: BaseFloat, Sh: Shape<S>, M: Material<S>> {
        pub shape: Sh,
        pub material: M,
        __: PhantomData<fn(_: ()) -> (S)>,
    }
    impl<S: BaseFloat, Sh: Shape<S>, M: Material<S>> ShapePrimitive<S, Sh, M> {
        pub fn new(shape: Sh, material: M) -> ShapePrimitive<S, Sh, M> {
            ShapePrimitive { shape, material, __: PhantomData }
        }
    }

    impl<S: BaseFloat, Sh: Shape<S>, M: Material<S>> Primitive<S> for ShapePrimitive<S, Sh, M> {
        fn intersect(&self, r: &Ray3<S>) -> Option<SurfaceInteraction<'_, S>> {
            self.shape.intersect(r).map(|(point, normal)| SurfaceInteraction {
                point,
                normal,
                prim: self,
                material: &self.material,
            })
        }
    }
}

use self::geom::*;
use self::material::*;
use self::prims::*;

struct Scene<S: BaseFloat> {
    pub aggregate: dyn Primitive<S>,
}

struct Aggregate<S: BaseFloat> {
    prims: Vec<Box<dyn Primitive<S> + Sync>>,
}

impl<S: BaseFloat> Primitive<S> for Aggregate<S> {
    fn intersect(&self, r: &Ray3<S>) -> Option<SurfaceInteraction<'_, S>> {
        // TODO: rewrite this faster. for loop & two if statements
        self.prims.iter().fold(None, |best, p| match (best, p.intersect(r)) {
            (None, int) => int,
            (Some(best), None) => Some(best),
            (Some(best), Some(int)) => {
                let t_int = (int.point - r.origin).magnitude();
                let t_best = (best.point - r.origin).magnitude();
                Some(if t_int < t_best { int } else { best })
            }
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init()?;
    info!("starting");
    let width = 960;
    let height = 600;
    let samples = 100;
    let bounces = 50;

    let world = Aggregate::<f64> {
        prims: vec![
            Box::new(ShapePrimitive::new(
                Sphere { center: Point3::new(0.0, 0.0, -1.0), radius: 0.5f64 },
                Lambertian { albedo: Vector3::new(0.8, 0.3, 0.3) },
            )),
            Box::new(ShapePrimitive::new(
                Sphere { center: Point3::new(0.0, -100.5, -1.0), radius: 100.0 },
                Lambertian { albedo: Vector3::new(0.8, 0.8, 0.0) },
            )),
            Box::new(ShapePrimitive::new(
                Sphere { center: Point3::new(1.0, 0.0, -1.0), radius: 0.5 },
                Metal { albedo: Vector3::new(0.8, 0.6, 0.2), fuzz: 1.0 },
            )),
            Box::new(ShapePrimitive::new(
                Sphere { center: Point3::new(-1.0, 0.0, -1.0), radius: 0.5 },
                Metal { albedo: Vector3::new(0.8, 0.8, 0.8), fuzz: 0.3 },
            )),
        ],
    };

    let c = Camera::new();
    let mut imgbuf = image::RgbImage::new(width, height);
    imgbuf
        .enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut image::Rgb<u8>)>>()
        .par_iter_mut()
        // .iter_mut()
        .for_each(|(x, y, pixel)| {
            let mut col = Vector3::zero();
            for _ in 0..samples {
                let u = (f64::from(*x) + random::<f64>() - 0.5) / f64::from(width);
                let v = (f64::from(height - *y) + random::<f64>() - 0.5) / f64::from(height);
                let r = c.get_ray(u, v);
                col += color(&r, &world, bounces);
            }
            col /= f64::from(samples);
            col = col.map(|x| x.sqrt()); // gamma correction
            **pixel = image::Rgb([
                (col[0] * 255.99) as u8,
                (col[1] * 255.99) as u8,
                (col[2] * 255.99) as u8,
            ]);
        });

    image::ImageRgb8(imgbuf).save("out.png")?;
    info!("done");
    Ok(())
}

fn color<S: BaseFloat>(r: &Ray3<S>, world: &dyn Primitive<S>, bounces: u64) -> Vector3<S>
where
    distributions::Standard: distributions::Distribution<S>,
{
    match world.intersect(r) {
        Some(ref hit) if bounces > 0
        // && (r.origin - hit.point).magnitude() > S::from(0.00001).unwrap()
        =>
        // hit.normal.map(|x| x + S::one()) / S::from(2).unwrap(),
        {
            // Hit a thing!
            match hit.material.scatter(r, &hit.point, &hit.normal) {
                None => Vector3::<S>::zero(),
                Some((ray, xform)) => xform(color(&ray, world, bounces-1)),
            }
        }

        _ => {
            // No hit - eval against background radiation.
            let t = (r.direction.y + S::one()) / S::from(2).unwrap();
            Vector3::new(S::one(), S::one(), S::one()) * (S::one() - t)
                + Vector3::new(S::from(0.5).unwrap(), S::from(0.7).unwrap(), S::one()) * t
        }
    }
}

struct Camera<S> {
    origin: Point3<S>,
    direction: Vector3<S>,
    image_distance: S,
    image_bounds: Vector2<S>,
}
impl<S> Camera<S>
where
    S: BaseFloat,
{
    fn new() -> Camera<S> {
        Camera {
            origin: Point3::origin(),
            direction: Vector3::unit_z().neg(),
            image_distance: S::one(),
            image_bounds: Vector2::new(S::from(3.2).unwrap(), S::from(2.0).unwrap()),
        }
    }
    fn get_ray(&self, u: S, v: S) -> Ray3<S> {
        // TODO: The math for the direction *depends* on direction only having a z component.
        // Figure out how to transform the screen bounds correctly into the direction's coordinate system.
        Ray3 {
            origin: self.origin,
            direction: (self.direction * self.image_distance
                + self.origin.to_vec()
                + Vector3::new(
                    self.image_bounds.x * (u - S::from(0.5).unwrap()),
                    self.image_bounds.y * (v - S::from(0.5).unwrap()),
                    S::zero(),
                )).normalize(),
        }
    }
}
