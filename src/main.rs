use rand::Rng;
use image::{ImageBuffer, Rgb};

const SIZE: u32 = 256;
const NUM_POINTS: usize = 20;
const BLUR_RADIUS: i32 = 64;

#[derive(Clone, Copy)]
struct Point { x: f32, y: f32 }

fn distance(p1: Point, p2: Point) -> f32 {
    let dx = (p1.x - p2.x).abs();
    let dy = (p1.y - p2.y).abs();
    let dx = dx.min(1.0 - dx);
    let dy = dy.min(1.0 - dy);
    (dx * dx + dy * dy).sqrt()
}

fn generate_tileable_voronoi() -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut rng = rand::thread_rng();
    let points: Vec<Point> = (0..NUM_POINTS)
        .map(|_| Point { x: rng.gen(), y: rng.gen() })
        .collect();

    // First pass: find the maximum distance
    let max_distance = (0..SIZE).flat_map(|x| (0..SIZE).map(move |y| (x, y)))
        .map(|(x, y)| {
            let current = Point { 
                x: x as f32 / SIZE as f32, 
                y: y as f32 / SIZE as f32 
            };
            points.iter()
                .map(|&p| distance(current, p))
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap()
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    // Second pass: generate the image
    ImageBuffer::from_fn(SIZE, SIZE, |x, y| {
        let current = Point { 
            x: x as f32 / SIZE as f32, 
            y: y as f32 / SIZE as f32 
        };

        let min_distance = points.iter()
            .map(|&p| distance(current, p))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        // Normalize the distance and invert it (distant = brighter)
        let normalized_distance = 1.0 - (min_distance / max_distance);
        
        // Map to 0-255 range for the red channel
        let red_value = 255 - (normalized_distance * 255.0) as u8;

        Rgb([red_value, 0, 0])  // Only red channel, others set to 0
    })
}

fn directional_blur(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    data_channel: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    blur_radius: i32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let direction = data_channel.get_pixel(x, y)[0] as f32 / 255.0 * 360.0;
            let mut sum_r = 0.0;
            let mut count = 0.0;

            for i in -blur_radius..=blur_radius {
                let angle = direction.to_radians();
                let dx = (i as f32 * angle.cos()).round() as i32;
                let dy = (i as f32 * angle.sin()).round() as i32;

                let sample_x = (x as i32 + dx).rem_euclid(width as i32) as u32;
                let sample_y = (y as i32 + dy).rem_euclid(height as i32) as u32;

                let pixel = img.get_pixel(sample_x, sample_y);
                sum_r += pixel[0] as f32;
                count += 1.0;
            }

            let blurred_pixel = Rgb([
                (sum_r / count).round() as u8,
                0,
                0,
            ]);
            output.put_pixel(x, y, blurred_pixel);
        }
    }

    output
}

fn main() {
    // Generate the Voronoi texture
    let voronoi_texture = generate_tileable_voronoi();
    voronoi_texture.save("voronoi_texture_red.png").unwrap();

    // Apply directional blur using the Voronoi texture as both input and data channel
    let blurred_texture = directional_blur(&voronoi_texture, &voronoi_texture, BLUR_RADIUS);
    blurred_texture.save("blurred_voronoi_texture_red.png").unwrap();
}