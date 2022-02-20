pub mod errors;
pub mod meshutil;
pub mod people;
pub mod util;
pub mod visuals;

#[test]
fn test_font() {
    use font_kit::canvas::{Canvas, Format, RasterizationOptions};
    use font_kit::family_name::FamilyName;
    use font_kit::hinting::HintingOptions;
    use font_kit::properties::Properties;
    use font_kit::source::SystemSource;
    use pathfinder_geometry::transform2d::Transform2F;
    use pathfinder_geometry::vector::{Vector2F, Vector2I};

    let font = SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .unwrap()
        .load()
        .unwrap();
    let glyph_id = font.glyph_for_char('A').unwrap();
    let mut canvas = Canvas::new(Vector2I::splat(32), Format::A8);
    font.rasterize_glyph(
        &mut canvas,
        glyph_id,
        32.0,
        Transform2F::from_translation(Vector2F::new(0.0, 32.0)),
        HintingOptions::None,
        RasterizationOptions::GrayscaleAa,
    )
    .unwrap();
    //println!("{:?}", canvas.pixels);
}

#[test]
fn test_tesselation() {
    use nalgebra as na;

    struct UnitSphere {
        bbox: tessellation::BoundingBox<f64>,
    }

    impl UnitSphere {
        fn new() -> UnitSphere {
            UnitSphere {
                bbox: tessellation::BoundingBox::new(
                    &na::Point3::new(-1., -1., -1.),
                    &na::Point3::new(1., 1., 1.),
                ),
            }
        }
    }

    impl tessellation::ImplicitFunction<f64> for UnitSphere {
        fn bbox(&self) -> &tessellation::BoundingBox<f64> {
            &self.bbox
        }
        fn value(&self, p: &na::Point3<f64>) -> f64 {
            return na::Vector3::new(p.x, p.y, p.z).norm() - 1.0;
        }
        fn normal(&self, p: &na::Point3<f64>) -> na::Vector3<f64> {
            return na::Vector3::new(p.x, p.y, p.z).normalize();
        }
    }

    let sphere = UnitSphere::new();
    let mut mdc = tessellation::ManifoldDualContouring::new(&sphere, 0.2, 0.1);
    let triangles = mdc.tessellate().unwrap();

    //println!("Triangles {:?}", triangles);
}

#[test]
fn test_tesselate_glyph() {
    use font_kit::canvas::{Canvas, Format, RasterizationOptions};
    use font_kit::family_name::FamilyName;
    use font_kit::hinting::HintingOptions;
    use font_kit::properties::Properties;
    use font_kit::source::SystemSource;
    use pathfinder_geometry::transform2d::Transform2F;
    use pathfinder_geometry::vector::{Vector2F, Vector2I};

    let font = SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .unwrap()
        .load()
        .unwrap();
    let glyph_id = font.glyph_for_char('A').unwrap();
    let mut canvas = Canvas::new(Vector2I::splat(32), Format::A8);
    font.rasterize_glyph(
        &mut canvas,
        glyph_id,
        32.0,
        Transform2F::from_translation(Vector2F::new(0.0, 32.0)),
        HintingOptions::None,
        RasterizationOptions::GrayscaleAa,
    )
    .unwrap();

    use nalgebra as na;

    struct TextBox {
        bbox: tessellation::BoundingBox<f64>,
        canvas: Canvas
    }

    impl tessellation::ImplicitFunction<f64> for TextBox {
        fn bbox(&self) -> &tessellation::BoundingBox<f64> {
            &self.bbox
        }
        fn value(&self, p: &na::Point3<f64>) -> f64 {
            // let n = na::Vector3::new(p.x, p.y, p.z).norm();
            // return n - 1.0;
            if p.z.abs() > 0.1 {
                return 1.0;
            } 
            self.canvas.pixels.get(
                p.y as usize * self.canvas.size.x() as usize
                + 
                p.x as usize
            ).copied()
            .map(|x| 16.5-(x as f64))
            .unwrap_or(1.0)
            //16 is an arbitrary threshold
            //return na::Vector3::new(p.x, p.y, p.z).norm() - 1.0;
        }
        fn normal(&self, p: &na::Point3<f64>) -> na::Vector3<f64> {
            return na::Vector3::new(0., 0., p.z.signum());
        }
    }

    

    let sphere = TextBox {
        bbox: tessellation::BoundingBox::new(
            &na::Point3::new(0., 0., -1.),
            &na::Point3::new(32., 32., 1.),
        ),
        canvas
    };
    let mut mdc = tessellation::ManifoldDualContouring::new(&sphere, 1.0, 0.1);
    let triangles = mdc.tessellate().unwrap();
    

    println!("Triangles: {:?}", triangles);
}
