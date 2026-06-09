pub struct FallOffGenerator;

impl FallOffGenerator {
    pub fn generate_fall_off_map(size: usize) -> Vec<Vec<f32>> {
        let mut map = vec![vec![0.0; size]; size];
        let half_size = size as f32 / 2.0;

        for y in 0..size {
            for x in 0..size {
                let normalized_x = (x as f32 - half_size) / half_size;
                let normalized_y = (y as f32 - half_size) / half_size;
                let value = normalized_x.abs().max(normalized_y.abs());
                map[y][x] = Self::evaluate(value);
            }
        }

        map
    }

    fn evaluate(value: f32) -> f32 {
        // This function creates a smooth fall-off curve.
        // You can adjust the parameters to change the shape of the curve.
        let a = 3.0;
        let b = 2.2;
        value.powf(a) / (value.powf(a) + (b - b * value).powf(a))
    }
}
