use rand::Rng;
use image::{ImageBuffer, Rgb};

const SIZE: u32 = 256;
const NUM_POINTS: usize = 20;

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

        Rgb([red_value, 0, 0])
    })
}

fn main() {
    let texture = generate_tileable_voronoi();
    texture.save("normalized_distance_voronoi.png").unwrap();
}