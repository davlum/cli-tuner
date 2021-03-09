use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

fn criterion_benchmark(c: &mut Criterion) {
    let signal = generate_input();
    c.bench_function("estimate_pitch_imperative", |b| b.iter(|| imp::Bitstream::estimate_pitch(black_box(&signal))));
    c.bench_function("estimate_pitch_declarative", |b| b.iter(|| decl::Bitstream::estimate_pitch(black_box(&signal))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
