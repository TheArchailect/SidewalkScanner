use crate::constants::COORDINATE_TRANSFORM;

/// Apply coordinate transformation matrix to ensure consistency.
/// Transforms input coordinates using predefined transformation matrix.
pub fn transform_coordinates(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let input = [x, y, z];
    let mut output = [0.0; 3];

    for i in 0..3 {
        for j in 0..3 {
            output[i] += COORDINATE_TRANSFORM[i][j] * input[j];
        }
    }

    (output[0], output[1], output[2])
}
