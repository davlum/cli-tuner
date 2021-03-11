use std::f32::consts::PI;
use clituner::bac::imp;
use clituner::bac::decl;
use clituner::bac::conf;

const FREQ: f32 = 261.626;
const PERIOD: f32 = conf::CONFIG.samples_per_second as f32 / FREQ;

fn generate_input() -> Vec<f32> {
    (0..conf::CONFIG.buff_size).map(|x| {
        let angle = x as f32 / PERIOD;
        let first_harmonic = 0.3 * (2.0 * PI * angle).sin();
        let second_harmonic = 0.4 * (4.0 * PI * angle).sin();
        let third_harmonic = 0.3 * (6.0 * PI * angle).sin();
        first_harmonic + second_harmonic + third_harmonic
    }).collect()
}

#[test]
fn test_get_smallest_pow2() {
    assert_eq!(conf::get_smallest_pow2(3), 4);
    assert_eq!(conf::get_smallest_pow2(1200), 2048);
}

#[test]
fn test_imperative_bitstream() {
    let mut array = [0; 64];
    let mut bs = imp::Bitstream::new(&mut array);
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
fn test_declarative_bitstream() {
    let mut bs = decl::Bitstream::new();
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
fn test_imperative_esimate_pitch() {
    let signal = generate_input();
    let res = imp::Bitstream::estimate_pitch(&signal);
    assert_eq!(format!("{:.3}", res.unwrap()), "261.626");
}

#[test]
fn test_declarative_esimate_pitch() {
    let signal = generate_input();
    let res = decl::Bitstream::estimate_pitch(&signal);
    assert_eq!(format!("{:.3}", res.unwrap()), "261.626");
}

#[test]
fn test_imperative_autocorrelate() {
    let mut zc = imp::ZeroCross::new();
    let mut array = [0; 64];
    let mut bs = imp::Bitstream::new(&mut array);
    let signal = generate_input();
    for i in 0..conf::CONFIG.buff_size {
        bs.set(i, zc.run(signal[i]));
    }
    let (count, est_index, mut corr) = bs.autocorrelate(conf::CONFIG.min_period);
    assert_eq!(count, 617);
    assert_eq!(est_index, 337);
    let est_index = imp::Bitstream::handle_harmonics(count, est_index, &mut corr);
    assert_eq!(est_index, 168);
}

#[test]
fn test_declarative_autocorrelate() {
    let mut zc = decl::ZeroCross::new();
    let mut bs = decl::Bitstream::new();
    let signal = generate_input();
    for i in 0..conf::CONFIG.buff_size {
        bs.set(i, zc.run(signal[i]));
    }
    let (count, est_index, mut corr) = bs.autocorrelate(conf::CONFIG.min_period);
    assert_eq!(count, 617);
    assert_eq!(est_index, 337);
    let est_index = decl::Bitstream::handle_harmonics(count, est_index, &mut corr);
    assert_eq!(est_index, 168);
}

#[test]
fn test_imperative_handle_harmonics() {
    let mut zc = imp::ZeroCross::new();
    let mut array = [0; 64];
    let mut bs = imp::Bitstream::new(&mut array);
    let signal = generate_input();
    for i in 0..conf::CONFIG.buff_size {
        bs.set(i, zc.run(signal[i]));
    }
    let (count, est_index, mut corr) = bs.autocorrelate(conf::CONFIG.min_period);
    let est_index = imp::Bitstream::handle_harmonics(count, est_index, &mut corr);
    assert_eq!(est_index, 168);
}

#[test]
fn test_declarative_handle_harmonics() {
    let mut zc = decl::ZeroCross::new();
    let mut bs = decl::Bitstream::new();
    let signal = generate_input();
    for i in 0..conf::CONFIG.buff_size {
        bs.set(i, zc.run(signal[i]));
    }
    let (count, est_index, mut corr) = bs.autocorrelate(conf::CONFIG.min_period);
    let est_index = decl::Bitstream::handle_harmonics(count, est_index, &mut corr);
    assert_eq!(est_index, 168);
}
