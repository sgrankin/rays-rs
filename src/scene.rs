use crate::aggregate::*;
use crate::material::*;
use crate::prims::*;
use crate::shape::*;
use crate::types::*;
use crate::util;

pub fn new_cover_scene<'a>() -> Aggregate {
    let mut random = util::new_random(0);

    let mut prims: Vec<Box<dyn Primitive>> = vec![
        Box::new(ShapePrimitive::new(
            Sphere { center: Point3f::new(0.0, -1000.0, 0.0), radius: 1000.0 },
            Lambertian { albedo: Vector3f::new(0.5, 0.5, 0.5) },
        )),
        Box::new(ShapePrimitive::new(
            Sphere { center: Point3f::new(0.0, 1.0, 0.0), radius: 1.0 },
            Dielectric { ref_index: 1.5 },
        )),
        Box::new(ShapePrimitive::new(
            Sphere { center: Point3f::new(-4.0, 1.0, 0.0), radius: 1.0 },
            Lambertian { albedo: Vector3f::new(0.4, 0.2, 0.1) },
        )),
        Box::new(ShapePrimitive::new(
            Sphere { center: Point3f::new(4.0, 1.0, 0.0), radius: 1.0 },
            Metal { albedo: Vector3f::new(0.7, 0.6, 0.5), fuzz: 0.0 },
        )),
    ];
    for a in -11i16..11i16 {
        for b in -11i16..11i16 {
            let center =
                Point3f::new(Float::from(a) + 0.9 * random(), 0.2, Float::from(b) + 0.9 * random());
            if (center - Point3f::new(4.0, 0.0, 2.0)).magnitude() <= 0.9 {
                continue;
            }
            let mat = random();
            let prim = if mat < 0.8 {
                // diffuse
                Box::new(ShapePrimitive::new(
                    Sphere { center, radius: 0.2 },
                    Lambertian {
                        albedo: Vector3f::new(
                            random() * random(),
                            random() * random(),
                            random() * random(),
                        ),
                    },
                )) as Box<dyn Primitive + Sync>
            } else if mat < 0.95 {
                //metal
                Box::new(ShapePrimitive::new(
                    Sphere { center, radius: 0.2 },
                    Metal {
                        albedo: Vector3f::new(
                            (random() + 1.0) / 2.0,
                            (random() + 1.0) / 2.0,
                            (random() + 1.0) / 2.0,
                        ),
                        fuzz: 0.5 * random(),
                    },
                )) as Box<dyn Primitive + Sync>
            } else {
                // glass
                Box::new(ShapePrimitive::new(
                    Sphere { center, radius: 0.2 },
                    Dielectric { ref_index: 1.5 },
                )) as Box<dyn Primitive + Sync>
            };
            prims.push(prim)
        }
    }
    Aggregate::new(prims)
}
