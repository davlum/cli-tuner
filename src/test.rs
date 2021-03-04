#[cfg(test)]
mod tests {
    use std::f32::consts::PI;
    use crate::{ZeroCross, get_smallest_pow2, CONFIG, Bitstream};

    #[test]
    fn test_get_smallest_pow2() {
        assert_eq!(get_smallest_pow2(3), 4);
        assert_eq!(get_smallest_pow2(1200), 2048);
    }

    #[test]
    fn test_bitstream() {
        let mut bs = Bitstream::new();
        let i = 7;
        bs.set(i, true);
        assert_eq!(bs.get(i), true);
        assert_eq!(bs.get(6), false);
        assert_eq!(bs.get(8), false);
        bs.set(i, false);
        assert_eq!(bs.get(i), false);
        assert_eq!(bs.get(31), false);
    }

    #[test]
    fn test_detect_peaks() {
        let freq = 261.626;
        let period = CONFIG.samples_per_second as f32 / freq;
        let inp: Vec<f32> = (0..CONFIG.buff_size).map(|x| {
            let angle = x as f32 / period;
            let first_harmonic = 0.3 * (2.0 * PI * angle).sin();
            let second_harmonic = 0.4 * (4.0 * PI * angle).sin();
            let third_harmonic = 0.3 * (6.0 * PI * angle).sin();
            first_harmonic + second_harmonic + third_harmonic
        }).collect();

        let res = Bitstream::estimate_pitch(&inp);
        assert_eq!(format!("{:.3}", res.unwrap()), "261.626");
    }

    #[test]
    fn test_estimate_pitch() {
        let freq = 261.626;
        let period = CONFIG.samples_per_second as f32 / freq;
        let signal: Vec<f32> = (0..CONFIG.buff_size).map(|x| {
            let angle = x as f32 / period;
            let first_harmonic = 0.3 * (2.0 * PI * angle).sin();
            let second_harmonic = 0.4 * (4.0 * PI * angle).sin();
            let third_harmonic = 0.3 * (6.0 * PI * angle).sin();
            first_harmonic + second_harmonic + third_harmonic
        }).collect();

        let mut zc = ZeroCross::new();
        let mut bs = Bitstream::new();
        for i in 0..CONFIG.buff_size {
            bs.set(i, zc.run(signal[i]));
        }

        let (count, est_index, mut corr) = bs.autocorrelate(CONFIG.min_period);
        assert_eq!(count, 617);
        assert_eq!(est_index, 337);
        let est_index = Bitstream::handle_harmonics(count, est_index, &mut corr);
        assert_eq!(est_index, 168);
    }
}
