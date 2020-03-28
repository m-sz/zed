pub fn calculate_frame_from_angle(angle: f64, discrete_directions: usize) -> (usize, f64) {
    use std::f64::consts::{PI, FRAC_PI_2, FRAC_PI_6, FRAC_PI_8};
    let angle = angle + FRAC_PI_2;
    let angle = (angle % (2.0*PI) + 2.0*PI) % (2.0*PI);

    let rad_per_direction = FRAC_PI_2 / (discrete_directions as f64);
    let frame = (angle / rad_per_direction).round() % 4.0;
    let angle = ((angle + rad_per_direction/2.0) / FRAC_PI_2).trunc() * FRAC_PI_2 % (2.0*PI);

    (frame as usize , angle)
}

mod tests {
    use super::calculate_frame_from_angle;
    #[test]
    fn test_calculate_frame_from_angle() {
        use std::f64::consts::{PI, FRAC_PI_2, FRAC_PI_6, FRAC_PI_8};

        let angles_results = [
            (FRAC_PI_8 * 0.0, (0, FRAC_PI_2)), // 0 deg, case #1
            (FRAC_PI_8 * 1.0, (1, FRAC_PI_2)),
            (FRAC_PI_8 * 2.0, (2, FRAC_PI_2)),
            (FRAC_PI_8 * 3.0, (3, FRAC_PI_2)),

            (FRAC_PI_8 * 4.0, (0, FRAC_PI_2*2.0)), // 90 deg
            (FRAC_PI_8 * 5.0, (1, FRAC_PI_2*2.0)),
            (FRAC_PI_8 * 6.0, (2, FRAC_PI_2*2.0)),
            (FRAC_PI_8 * 7.0, (3, FRAC_PI_2*2.0)),

            (FRAC_PI_8 * 8.0, (0, FRAC_PI_2*3.0)), // 180 deg
            (FRAC_PI_8 * 9.0, (1, FRAC_PI_2 * 3.0)),
            (FRAC_PI_8 * 10.0, (2, FRAC_PI_2 * 3.0)),
            (FRAC_PI_8 * 11.0, (3, FRAC_PI_2 * 3.0)),

            (FRAC_PI_8 * 12.0, (0, FRAC_PI_2 * 0.0)), // 270 deg
            (FRAC_PI_8 * 13.0, (1, FRAC_PI_2 * 0.0)),
            (FRAC_PI_8 * 14.0, (2, FRAC_PI_2 * 0.0)),
            (FRAC_PI_8 * 15.0, (3, FRAC_PI_2 * 0.0)),

            // Edge cases
            (FRAC_PI_8 * 11.5, (0, 0.0)),
            (FRAC_PI_8 * 11.9, (0, 0.0)),
            (FRAC_PI_8 * 12.49, (0, 0.0)),
            (FRAC_PI_8 * 12.51, (1, 0.0)),

            (-FRAC_PI_8*5.0, (3, FRAC_PI_2*3.0)),
        ];

        for (i, &(angle, (frame, rotation))) in angles_results.iter().enumerate() {
            assert_eq!(calculate_frame_from_angle(angle, 4), (frame, rotation), "Mismatch for angle {} (rad) {} (deg) in case #{}", angle, angle * 180.0 / PI, i+1);
        }
    }
}
