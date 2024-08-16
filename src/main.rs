use rand::Rng;
use image::{ImageBuffer, Rgb};
use noise::{NoiseFn, Perlin};

const SIZE: u32 = 512;
const NUM_POINTS: usize = 240;
const BLUR_RADIUS: i32 = 3;

#[derive(Clone, Copy)]
struct Point { x: f32, y: f32 }

/// Calculate the toroidal distance between two points
///
/// This function ensures that the distance wraps around the edges of the texture,
/// creating a seamless, tileable result.
///
/// # Arguments
///
/// * `p1` - The first point
/// * `p2` - The second point
///
/// # Returns
///
/// The toroidal distance between the two points
///
/// # Example
///
/// ```rust
/// let p1 = Point { x: 0.1, y: 0.1 };
/// let p2 = Point { x: 0.9, y: 0.9 };
/// let distance = toroidal_distance(p1, p2);
/// assert!(distance < 0.3); // The wrapped distance should be small
/// ```
fn toroidal_distance(p1: Point, p2: Point) -> f32 {
    let dx = (p1.x - p2.x).abs();
    let dy = (p1.y - p2.y).abs();
    let dx = dx.min(1.0 - dx);
    let dy = dy.min(1.0 - dy);
    (dx * dx + dy * dy).sqrt()
}

/// Generate a tileable Voronoi diagram
///
/// This function creates a Voronoi diagram that can be tiled seamlessly.
/// The resulting image uses only the red channel, with brighter values
/// representing areas further from Voronoi cell centers.
///
/// # Algorithm
///
/// 1. Generate random points in a unit square
/// 2. For each pixel in the output image:
///    a. Calculate the toroidal distance to each Voronoi point
///    b. Find the minimum distance
/// 3. Normalize the minimum distances across the entire image
/// 4. Invert the normalized distances (so cell centers are dark and edges are bright)
/// 5. Map the inverted distances to grayscale values (0-255)
///
/// # Returns
///
/// An `ImageBuffer` containing the Voronoi diagram
///
/// # Performance
///
/// This function has O(SIZE^2 * NUM_POINTS) complexity. For large images or
/// many Voronoi points, consider parallelizing the pixel generation process.
///
/// # Example
///
/// ```rust
/// let voronoi_texture = generate_tileable_voronoi();
/// save_image(&voronoi_texture, "voronoi_texture.png").unwrap();
/// ```
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
                .map(|&p| toroidal_distance(current, p))
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap()
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    // Second pass: generate the image
    ImageBuffer::from_fn(SIZE, SIZE, |x, y| {
        let current_point = Point { 
            x: x as f32 / SIZE as f32, 
            y: y as f32 / SIZE as f32 
        };

        let min_distance = points.iter()
            .map(|&p| toroidal_distance(current_point, p))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        // Normalize the distance and invert it (distant = brighter)
        let normalized_distance = 1.0 - (min_distance / max_distance);
        
        // Map to 0-255 range for the red channel
        let red_value = 255 - (normalized_distance * 255.0) as u8;

        Rgb([red_value, 0, 0])  // Only red channel, others set to 0
    })
}

/// Apply directional blur to an image
///
/// This function applies a directional blur to the input image, using another image
/// as a direction map. The blur direction for each pixel is determined by the
/// corresponding pixel value in the direction map.
///
/// # Algorithm
///
/// 1. For each pixel in the input image:
///    a. Determine the blur direction from the direction map
///    b. Sample pixels along this direction within the blur radius
///    c. Calculate the average of the sampled pixels
///    d. Set the output pixel to this average value
/// 2. Wrap around image edges to ensure seamless tiling
///
/// # Arguments
///
/// * `img` - The input image to be blurred
/// * `direction_channel` - The image used as a direction map for the blur
/// * `blur_radius` - The radius of the blur effect
///
/// # Returns
///
/// An `ImageBuffer` containing the blurred image
///
/// # Performance
///
/// This function has O(width * height * blur_radius) complexity. For large images
/// or large blur radii, consider parallelizing the pixel processing.
///
/// # Example
///
/// ```rust
/// let input_image = generate_tileable_voronoi();
/// let direction_map = generate_perlin_noise();
/// let blurred_image = directional_blur(&input_image, &direction_map, 5);
/// save_image(&blurred_image, "blurred_image.png").unwrap();
/// ```
fn directional_blur(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    direction_channel: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    blur_radius: i32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut output = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let direction = direction_channel.get_pixel(x, y)[0] as f32 / 255.0 * 360.0;
            let mut sum_red = 0.0;
            let mut count = 0.0;

            for i in -blur_radius..=blur_radius {
                let angle = direction.to_radians();
                let delta_x = (i as f32 * angle.cos()).round() as i32;
                let delta_y = (i as f32 * angle.sin()).round() as i32;

                let sample_x = (x as i32 + delta_x).rem_euclid(width as i32) as u32;
                let sample_y = (y as i32 + delta_y).rem_euclid(height as i32) as u32;

                let pixel = img.get_pixel(sample_x, sample_y);
                sum_red += pixel[0] as f32;
                count += 1.0;
            }

            let blurred_pixel = Rgb([
                (sum_red / count).round() as u8,
                0,
                0,
            ]);
            output.put_pixel(x, y, blurred_pixel);
        }
    }

    output
}

/// Generate Perlin noise texture
///
/// This function creates a texture using Perlin noise with multiple octaves,
/// resulting in a fractal-like pattern. The noise is normalized to use only
/// the red channel of the image.
///
/// # Algorithm
///
/// 1. Initialize a Perlin noise generator
/// 2. For each pixel in the output image:
///    a. Generate fractal Brownian motion (fBm) noise:
///       - Sum multiple octaves of Perlin noise
///       - For each octave, increase frequency and decrease amplitude
///    b. Normalize the resulting noise value to the range [0, 1]
///    c. Map the normalized value to a grayscale intensity (0-255)
/// 3. Set the red channel of each pixel to the calculated intensity
///
/// # Returns
///
/// An `ImageBuffer` containing the Perlin noise texture
///
/// # Performance
///
/// The complexity is O(SIZE^2 * octaves). Consider parallelizing the pixel
/// generation process for large images or many octaves.
///
/// # Example
///
/// ```rust
/// let perlin_texture = generate_perlin_noise();
/// save_image(&perlin_texture, "perlin_texture.png").unwrap();
/// ```
fn generate_perlin_noise() -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let perlin = Perlin::new(0);
    let octaves = 6;
    let persistence = 0.5;
    let lacunarity = 2.0;

    ImageBuffer::from_fn(SIZE, SIZE, |x, y| {
        let mut noise_value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            let normalized_x = x as f64 / SIZE as f64 * frequency;
            let normalized_y = y as f64 / SIZE as f64 * frequency;

            noise_value += perlin.get([normalized_x, normalized_y]) * amplitude;
            
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        // Normalize the noise value
        noise_value = (noise_value / max_value + 1.0) / 2.0;
        let intensity = (noise_value * 255.0) as u8;

        Rgb([intensity, 0, 0])
    })
}
/// Normalize an image to use the full 0-255 range
///
/// This function adjusts the pixel values of the input image to span the full
/// 0-255 range, improving contrast. It operates only on the red channel.
///
/// # Algorithm
///
/// 1. Find the minimum and maximum pixel values in the input image
/// 2. For each pixel:
///    a. Apply the formula: new_value = (old_value - min) / (max - min) * 255
///    b. Round the result and set it as the new pixel value
///
/// # Arguments
///
/// * `img` - The input image to be normalized
///
/// # Returns
///
/// An `ImageBuffer` containing the normalized image
///
/// # Performance
///
/// This function has O(width * height) complexity. For very large images,
/// consider processing the image in chunks to reduce memory usage.
///
/// # Example
///
/// ```rust
/// let input_image = generate_perlin_noise();
/// let normalized_image = normalize_image(&input_image);
/// save_image(&normalized_image, "normalized_perlin.png").unwrap();
/// ```
fn normalize_image(img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = img.dimensions();
    let mut min_value = 255;
    let mut max_value = 0;

    // Find min and max values
    for pixel in img.pixels() {
        let value = pixel[0];
        min_value = min_value.min(value);
        max_value = max_value.max(value);
    }

    // Normalize the image
    ImageBuffer::from_fn(width, height, |x, y| {
        let pixel = img.get_pixel(x, y);
        let normalized_value = if max_value > min_value {
            (pixel[0] as f32 - min_value as f32) / (max_value as f32 - min_value as f32) * 255.0
        } else {
            pixel[0] as f32
        };
        Rgb([normalized_value.round() as u8, 0, 0])
    })
}

/// Main function to generate and process textures
///
/// This function orchestrates the texture generation process:
/// 1. Generates a Voronoi texture
/// 2. Generates a Perlin noise texture
/// 3. Applies directional blur to the Voronoi texture
/// 4. Saves the resulting textures as PNG images
///

fn main() {
    // Generate the Voronoi texture
    let voronoi_texture = generate_tileable_voronoi();
    voronoi_texture.save("voronoi_texture_red.png").unwrap();

    // Generate and save the Perlin noise texture
    let perlin_texture = generate_perlin_noise();
    perlin_texture.save("perlin_noise_texture.png").unwrap();
    
    // Apply directional blur using the Voronoi texture as both input and data channel
    let mut blurred_texture = voronoi_texture.clone();
    
    for i in 0..4 {
        blurred_texture = directional_blur(&blurred_texture, &voronoi_texture, BLUR_RADIUS * 2i32.pow(i));
        blurred_texture = normalize_image(&blurred_texture);
        
        // Save intermediate results (optional)
        //blurred_texture.save(format!("blurred_voronoi_texture_red_step_{}.png", i+1)).unwrap();
    }

    // Save the final result
    blurred_texture.save("blurred_voronoi_texture_red.png").unwrap();
}