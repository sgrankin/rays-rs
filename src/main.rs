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
    impl<S: BaseNum> Ray3<S> {
        pub fn new(origin: Point3<S>, direction: Vector3<S>) -> Ray3<S> {
            Ray3 { origin, direction }
        }
    }

    pub trait Shape<S: BaseFloat> {
        fn intersect(&self, _: Ray3<S>) -> Option<(Point3<S>, Vector3<S>)>;
    }

    pub struct Sphere<S: BaseFloat> {
        pub center: Point3<S>,
        pub radius: S,
    }

    impl<S: BaseFloat> Shape<S> for Sphere<S> {
        fn intersect(&self, r: Ray3<S>) -> Option<(Point3<S>, Vector3<S>)> {
            // Due to floating point errors, advance ray up to avoid re-intersecting.
            // NOTE: this is insufficient for very oblique rays; see PBRT error-tracking for a better solution.
            let r = Ray3::new(r.origin + r.direction * S::from(0.000_001).unwrap(), r.direction);
            let r2 = self.radius * self.radius;
            let norm_dir = if self.radius > S::zero() { S::one() } else { -S::one() };
            let l = self.center - r.origin;
            let p = if l.magnitude2() >= r2 {
                // origin is outside or on the surface
                let tca = l.dot(r.direction);
                if tca <= S::zero() {
                    // heading away from the sphere
                    return None;
                }
                let d2 = l.magnitude2() - tca * tca;
                if d2 >= r2 {
                    return None;
                }
                let thc = (r2 - d2).sqrt();
                r.origin
                    + r.direction
                        * (if tca != thc {
                            tca - thc
                        } else {
                            // we're on the surface going through the sphere, so we need to return the point on the other side.
                            // technically tca=thc;
                            tca + thc
                        })
            } else {
                // origin is in the sphere
                let tca = l.dot(r.direction);
                // if tca > 0: heading somewhat towards the center
                // else, heading away from center
                // In either case, correction is tca + thc; in the else case it's (thc - (-tca))
                let d2 = l.magnitude2() - tca * tca;
                let thc = (r2 - d2).sqrt();
                r.origin + r.direction * (tca + thc)
            };
            Some((p, (p - self.center).normalize() * norm_dir))
        }
    }
}

mod material {
    use super::geom::*;
    use super::util;
    use cgmath::*;
    use rand;
    use std::ops::*;

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

    impl<S: BaseFloat + 'static> Material<S> for Lambertian<S>
    where
        rand::distributions::Standard: rand::distributions::Distribution<S>,
    {
        fn scatter(
            &self,
            _in_: Ray3<S>,
            point: Point3<S>,
            normal: Vector3<S>,
        ) -> Option<(Ray3<S>, Vector3<S>)> {
            // Note we could just as well only scatter with some probability p and have attenuation be albedo/p.
            let ray = Ray3 {
                origin: point,
                direction: (normal + util::random_in_unit_sphere()).normalize(),
            };
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

    impl<S: BaseFloat + 'static> Material<S> for Metal<S>
    where
        rand::distributions::Standard: rand::distributions::Distribution<S>,
    {
        fn scatter(
            &self,
            in_: Ray3<S>,
            point: Point3<S>,
            normal: Vector3<S>,
        ) -> Option<(Ray3<S>, Vector3<S>)> {
            let reflected = reflect(in_.direction, normal);
            let scattered = Ray3 {
                origin: point,
                direction: (reflected + util::random_in_unit_sphere() * self.fuzz).normalize(),
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

    impl<S: BaseFloat + 'static> Material<S> for Dielectric<S>
    where
        rand::distributions::Standard: rand::distributions::Distribution<S>,
    {
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
                Some(refracted) if rand::random() >= reflect_p => refracted,
                _ => reflected,
            };
            Some((Ray3::new(point, out), attenuation))
        }
    }
}

mod prims {
    use super::geom::*;
    use super::material::*;

    use cgmath::*;
    use std::marker::PhantomData;

    pub trait Primitive<S: BaseFloat> {
        fn intersect(&self, _: Ray3<S>) -> Option<SurfaceInteraction<'_, S>>;
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
        fn intersect(&self, r: Ray3<S>) -> Option<SurfaceInteraction<'_, S>> {
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
    fn intersect(&self, r: Ray3<S>) -> Option<SurfaceInteraction<'_, S>> {
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
    let width = 1920; // 960; //960;
    let height = 1200; // 600; // 600;
    let sample_sqrt = 16;

    let mut prims: Vec<Box<dyn Primitive<f64> + Sync>> = vec![
        Box::new(ShapePrimitive::new(
            Sphere { center: Point3::new(0.0, -1000.0, 0.0), radius: 1000.0 },
            Lambertian { albedo: Vector3::new(0.5, 0.5, 0.5) },
        )),
        Box::new(ShapePrimitive::new(
            Sphere { center: Point3::new(0.0, 1.0, 0.0), radius: 1.0 },
            Dielectric { ref_index: 1.5 },
        )),
        Box::new(ShapePrimitive::new(
            Sphere { center: Point3::new(-4.0, 1.0, 0.0), radius: 1.0 },
            Lambertian { albedo: Vector3::new(0.4, 0.2, 0.1) },
        )),
        Box::new(ShapePrimitive::new(
            Sphere { center: Point3::new(4.0, 1.0, 0.0), radius: 1.0 },
            Metal { albedo: Vector3::new(0.7, 0.6, 0.5), fuzz: 0.0 },
        )),
    ];
    for a in -11..11 {
        for b in -11..11 {
            let center = Point3::new(
                f64::from(a) + 0.9 * random::<f64>(),
                0.2,
                f64::from(b) + 0.9 * random::<f64>(),
            );
            if (center - Point3::new(4.0, 0.0, 2.0)).magnitude() <= 0.9 {
                continue;
            }
            let mat = random::<f64>();
            let prim = if mat < 0.8 {
                // diffuse
                Box::new(ShapePrimitive::new(
                    Sphere { center, radius: 0.2 },
                    Lambertian {
                        albedo: Vector3::new(
                            random::<f64>() * random::<f64>(),
                            random::<f64>() * random::<f64>(),
                            random::<f64>() * random::<f64>(),
                        ),
                    },
                )) as Box<dyn Primitive<f64> + Sync>
            } else if mat < 0.95 {
                //metal
                Box::new(ShapePrimitive::new(
                    Sphere { center, radius: 0.2 },
                    Metal {
                        albedo: Vector3::new(
                            (random::<f64>() + 1.0) / 2.0,
                            (random::<f64>() + 1.0) / 2.0,
                            (random::<f64>() + 1.0) / 2.0,
                        ),
                        fuzz: 0.5 * random::<f64>(),
                    },
                )) as Box<dyn Primitive<f64> + Sync>
            } else {
                // glass
                Box::new(ShapePrimitive::new(
                    Sphere { center, radius: 0.2 },
                    Dielectric { ref_index: 1.5 },
                )) as Box<dyn Primitive<f64> + Sync>
            };
            prims.push(prim)
        }
    }

    let world = Aggregate::<f64> { prims };

    let from = Point3::new(12.0, 3.0, 3.0);
    let to = Point3::new(0.0, 0.0, -1.0);
    let c = Camera::new(
        from,
        to,
        Vector3::unit_y(),
        55.0,
        f64::from(width) / f64::from(height),
        0.1,
        (to - from).magnitude(),
    );
    let mut imgbuf = image::RgbImage::new(width, height);
    imgbuf
        .enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut image::Rgb<u8>)>>()
        .par_iter_mut()
        // .iter_mut()
        .for_each(|(x, y, pixel)| {
            let mut col = Vector3::zero();
            for i in 0..(sample_sqrt * sample_sqrt) {
                let quad_size = 1.0 / f64::from(sample_sqrt);
                let x_quad = i % sample_sqrt;
                let y_quad = i / sample_sqrt;
                let u = (f64::from(*x) + quad_size * (f64::from(x_quad) + random::<f64>()))
                    / f64::from(width);
                let v = (f64::from(height - *y)
                    + quad_size * (f64::from(y_quad) + random::<f64>()))
                    / f64::from(height);
                let r = c.get_ray(u, v);
                col += color(r, &world);
            }
            col /= f64::from(sample_sqrt * sample_sqrt);
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

fn color<S: BaseFloat>(r: Ray3<S>, world: &dyn Primitive<S>) -> Vector3<S>
where
    distributions::Standard: distributions::Distribution<S>,
{
    let p = S::from(7.0 / 8.0).unwrap();
    let mut bounces = 0;
    let mut ray = r;
    let mut color = Vector3::from_value(S::zero());
    let mut throughput = Vector3::from_value(S::one());
    loop {
        match world.intersect(ray) {
            Some(ref hit) => {
                // Hit a thing!
                match hit.material.scatter(ray, hit.point, hit.normal) {
                    None => return Vector3::zero(),
                    Some((r, t)) => {
                        ray = r;
                        throughput.mul_assign_element_wise(t);
                        if bounces > 5 {
                            let p = S::max(throughput.x, S::max(throughput.y, throughput.z));
                            if random::<S>() > p {
                                return Vector3::zero();
                            }
                            throughput /= p;
                            bounces += 1;
                        }
                    }
                };
            }

            _ => {
                // No hit - eval against background radiation.
                break;
            }
        }
    }
    let t = (ray.direction.y + S::one()) / S::from(2).unwrap();
    color = Vector3::from_value(S::one()) * (S::one() - t)
        + Vector3::new(S::from(0.5).unwrap(), S::from(0.7).unwrap(), S::one()) * t;
    color.mul_element_wise(throughput)
}

struct Camera<S> {
    origin: Point3<S>,
    lower_left: Point3<S>,
    horizontal: Vector3<S>,
    vertical: Vector3<S>,
    lens_radius: S,
    u: Vector3<S>,
    v: Vector3<S>,
    w: Vector3<S>,
}
impl<S> Camera<S>
where
    S: BaseFloat,
    rand::distributions::Standard: rand::distributions::Distribution<S>,
{
    fn new(
        origin: Point3<S>,
        target: Point3<S>,
        vup: Vector3<S>,
        fov: S,
        aspect: S,
        aperture: S,
        focus_dist: S,
    ) -> Camera<S> {
        let theta = fov * S::from(std::f64::consts::PI / 180.0).unwrap();

        let half_width = (theta / S::from(2).unwrap()).tan();
        let half_height = half_width / aspect;

        let w = (origin - target).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        Camera {
            origin,
            lower_left: origin - (u * half_width + v * half_height + w) * focus_dist,
            horizontal: u * (half_width + half_width) * focus_dist,
            vertical: v * (half_height + half_height) * focus_dist,
            lens_radius: aperture / S::from(2).unwrap(),
            u,
            v,
            w,
        }
    }
    fn get_ray(&self, s: S, t: S) -> Ray3<S> {
        let rd = util::random_in_unit_disk() * self.lens_radius;
        let offset = self.u * rd.x + self.v * rd.y;

        Ray3 {
            origin: self.origin + offset,
            direction: ((self.lower_left + self.horizontal * s + self.vertical * t)
                - (self.origin + offset))
                .normalize(),
        }
    }
}
