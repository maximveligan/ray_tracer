extern crate image;
extern crate cgmath;

use std::fs::File;
use std::io::Write;

use cgmath::Vector3;
use cgmath::Point3;
use cgmath::dot;
use cgmath::InnerSpace;

use image::Pixel;
use image::ImageFormat;
use image::Rgb;
use image::Rgba;
use image::DynamicImage;
use image::GenericImage;

trait Intersectable {
    fn intersection(&self, ray: &Ray) -> Option<f64>;
    fn normal_vec(&self, hit_point: &Point3<f64>) -> Vector3<f64>;
}

pub struct Sphere {
    radius: f64,
    albedo: f64,
    pub color: Rgb<f64>,
    pub center: Point3<f64>,
}

struct Plane {
    point: Point3<f64>,
    albedo: f64,
    normal: Vector3<f64>,
    color: Rgb<f64>,
}

pub struct Scene {
    width: u32,
    height: u32,
    fov: f64,
    objects: Vec<Object>,
    light: Light,
}

struct DirLight {
    direction: Vector3<f64>,
    intensity: f64,
    color: Rgb<f64>,
}

enum Object {
    Plane(Plane),
    Sphere(Sphere),
}

impl Object {
    fn get_color(&self) -> Rgb<f64>{
        match *self {
            Object::Sphere(ref s) => s.color,
            Object::Plane(ref p) => p.color,
        }
    }
    fn get_albedo(&self) -> f64 {
        match *self {
            Object::Sphere(ref s) => s.albedo,
            Object::Plane(ref p) => p.albedo,
        }
    }
}

enum Light {
    DirLight(DirLight),
}

impl Intersectable for Sphere {
    fn intersection(&self, ray: &Ray) -> Option<f64> {
        let center_l = self.center - ray.origin;
        let adj = center_l.dot(ray.direction);
        let r_squared = self.radius * self.radius;
        let d_squared = center_l.dot(center_l) - (adj * adj);
        if r_squared > d_squared {
            Some(adj - (r_squared - d_squared).sqrt())
        }
        else {
            None
        }
    }
    fn normal_vec(&self, hit_p: &Point3<f64>) -> Vector3<f64> {
        (*hit_p - self.center).normalize()
    }
}

fn color_mul_const(color: &Rgb<f64>, constant: f64) -> Rgb<f64> {
    Rgb { data: [((color.data[0] * constant)/255.0).min(255.0).max(0.0), ((color.data[1] * constant)/255.0).min(255.0).max(0.0), ((color.data[2] * constant)/255.0).min(255.0).max(0.0)] }
}

fn color_mul_color(color: &Rgb<f64>, color2: &Rgb<f64>) -> Rgb<f64> {
    Rgb { data: [((color.data[0] * color2.data[0])/255.0).min(255.0).max(0.0), ((color.data[1] * color2.data[1])/255.0).min(255.0).max(0.0), ((color.data[2] * color2.data[1])/255.0).min(255.0).max(0.0)] }
}

impl Intersectable for Object {
    fn intersection(&self, ray: &Ray) -> Option<f64> {
        match *self {
            Object::Plane(ref p) => p.intersection(ray),
            Object::Sphere(ref s) => s.intersection(ray),
        }
    }
    fn normal_vec(&self, hit_point: &Point3<f64>) -> Vector3<f64> {
        match *self {
            Object::Plane(ref p) => p.normal_vec(hit_point),
            Object::Sphere(ref s) => s.normal_vec(hit_point),
        }
    }
}

impl Intersectable for Plane {
    fn intersection(&self, ray: &Ray) -> Option<f64> {
        let denom = self.normal.dot(ray.direction);
        if denom > 1e-6 {
            let v = self.point - ray.origin;
            let distance = v.dot(self.normal) / denom;
            if distance >= 0.0 {
                return Some(distance);
            }
        }
        None
    }
    fn normal_vec(&self, _: &Point3<f64>) -> Vector3<f64> {
        -self.normal
    }
}

#[derive(Debug, Clone)]
struct Ray {
    origin: Point3<f64>,
    direction: Vector3<f64>,
}

const HALF_PIXEL: f64 = 0.5;
const BLACK: Rgba<u8> = Rgba {
    data: [70,70,70,1],
};
const ALBEDO_DEFAULT: f64 = 0.99;

impl Ray {
    fn create_prime(x: u32, y: u32, scene: &Scene) -> Ray {
        let aspect_ratio = scene.width as f64 / scene.height as f64;
        let canvas_x = (((x as f64 + HALF_PIXEL) / (scene.width as f64 /2.0)) - 1.0) * aspect_ratio;
        let canvas_y = 1.0 - ((y as f64 + HALF_PIXEL) / (scene.height as f64 /2.0));

        Ray {
            origin: Point3::new(0.0, 0.0, 0.0),
            direction: Vector3::new(canvas_x, canvas_y, -1.0).normalize(),
        }
    }
}

impl Scene {
    fn render(&self) -> DynamicImage {
        let mut image = DynamicImage::new_rgb8(self.width, self.height);

        for x in 0..self.width {
            for y in 0..self.height {

                let ray = Ray::create_prime(x, y, self);

                for object in self.objects.iter() {

                    if let Some(d) = object.intersection(&ray) {
                        
                        match self.light {
                            Light::DirLight(ref d_l) => { 
                                let light_p = (object.get_albedo() / std::f64::consts::PI) * d_l.intensity * object.normal_vec(&(ray.origin + (ray.direction * d))).dot(-d_l.direction.normalize()).max(0.0);
                                println!("got here");
                                image.put_pixel(x, y, rgbf64_to_rgb8(&color_mul_const(&color_mul_color(&object.get_color(), &d_l.color), light_p)).to_rgba());
                            }
                            _ => panic!("not done yet"),
                        }
                    }

                    else {
                        image.put_pixel(x, y, BLACK);
                    }

                }
            }
        }

    image

    }
}

fn rgbf64_to_rgb8(color: &Rgb<f64>) -> Rgb<u8> {
    Rgb { data: [color.data[0].round() as u8, color.data[1].round() as u8, color.data[2].round() as u8]}
}

#[test]
#[should_panic]
fn test_scene() {
    let scene = Scene {
        light: Light::DirLight(DirLight { 
            direction: Vector3 {
                x: -8.0,
                y: -10.0,
                z: -9.0,
            },
            intensity: 1000.0,
            color: Rgb {
                data: [230.0, 230.0, 230.0]
            },
        }),
        width: 800,
        height: 600,
        fov: 90.0,
        objects: vec![Object::Sphere(Sphere {
            center: Point3 {
                x: 0.0,
                y: 0.0,
                z: -7.0,
            },
            color: Rgb {
                data: [255.0, 255.0, 0.0]
            },
            albedo: ALBEDO_DEFAULT,       
            radius: 2.0,
        })],
    };
    let img = scene.render();
    let mut png: File = File::create("test.png").unwrap();
//  println!("{:?}", img.raw_pixels()[3]);
    img.save(&mut png, ImageFormat::JPEG);
}
