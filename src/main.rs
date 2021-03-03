mod test;

extern crate stft;
extern crate cpal;

use stft::{STFT, WindowType, log10_positive};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use std::sync::mpsc;
use num::complex::Complex;
use std::cmp::Ordering;

struct HyperParams {
    q: f64,
    p: f64,
    r: f64,
    rho: f64,
    num_harms: usize
}

struct TWMConfig {
    amplitude_threshhold: f32,
    max_error: f32,
    min_freq:f32,
    max_freq: f32,
    hyper_params: &'static HyperParams
}


const HYPER_PARAMS: &'static HyperParams = &HyperParams{
    p: 0.5,
    q: 1.4,
    r: 0.5,
    rho: 0.33,
    num_harms: 10
};

const TWM_CONFIG: &'static TWMConfig = &TWMConfig{
    amplitude_threshhold: -80.0,
    max_error: 5.0,
    min_freq: 100.0,
    max_freq: 3000.0,
    hyper_params: HYPER_PARAMS
};


fn to_db_mag_phase(x: Complex<f32>) -> (f32, f32) {
    let (mag, phase) = x.to_polar();
    let mag_in_db = 20.0 * log10_positive(mag);
    (mag_in_db, phase)
}


/// Takes a minimum threshold and a vector of (magnitudes, phases), and returns
/// a vector of indices of local maxima above the threshold.
pub fn detect_peaks(amplitude_threshold: f32, mag_phase_spectrum: Vec<(f32, f32)>) -> Vec<usize> {
    // Drop first and last elements, used in indexing.
    let len = mag_phase_spectrum.len();
    let indexed_spectrum = &mag_phase_spectrum.clone()
        .into_iter()
        .enumerate()
        .collect::<Vec<(usize, (f32, f32))>>()[1..len-1];
    let filtered = indexed_spectrum.into_iter()
        .map(|(_, (mag,_))| if mag >= &amplitude_threshold { mag } else { &0.0 });
    let next = indexed_spectrum.into_iter()
        .map(|(ind, (mag, _))| if mag > &mag_phase_spectrum[ind+1].0 { mag } else { &0.0 });
    let prev = indexed_spectrum.into_iter()
        .map(|(ind, (mag, _))| if mag > &mag_phase_spectrum[ind-1].0 { mag } else { &0.0 });
    filtered
        .zip(next)
        .zip(prev)
        .enumerate()
        .fold(Vec:: new(), |mut init, (i, ((f, n), p))| if f * n * p != 0.0 {
            init.push(i + 1);
            init
        } else { init })
}


/// linear interpolation determines, from two points (x0,y0) and (x1,y1), what the value of y is at a different point x3.
/// Linear interpolation assumes a linear slope between points 1 and 2 and based on that slope will determine what the value of
/// y would be at another given point x.
pub fn linear_interpoliation(p1: (f32, f32), p2: (f32, f32), x: f32) -> f32 {
    let slope = (p2.1 - p1.1) / (p2.0 - p1.0);
    p1.1 + (x - p1.0) * slope
}

fn get_adj_vals<T>(vec: &Vec<T>, i: usize) -> (&T, &T, &T) {
    (&vec[i -1], &vec[i], &vec[i + 1])
}

fn get_imaginary_peak_mag(p: f32, l: f32, m: f32, r: f32) -> f32 {
    p + 0.5 * (l - r)/(l - 2.0 * m + r)
}



pub fn interpolate_peaks(mag_phase_spectrum: Vec<(f32, f32)>, magnitude_peaks: Vec<usize>, sampling_rate: i32, window_len: usize) -> Vec<(f32, f32, f32)> {
    let ipm = |i: usize| {
        let (ml, mm, mr) = get_adj_vals(&mag_phase_spectrum, i);
        let ipeak = get_imaginary_peak_mag(i as f32, ml.0, mm.0, mr.0);
        let imag = mm.0 - 0.25 * (ml.0 - mr.0) * (ipeak - i as f32);
        let ind= ipeak.floor();
        let p1 = (ind, mag_phase_spectrum[ind as usize].1);
        let p2 = (ind + 1.0, mag_phase_spectrum[ind as usize + 1].1);
        let iphase = linear_interpoliation(p1, p2, ipeak);
        (indice_to_freq(ipeak, sampling_rate, window_len), imag, iphase)
    };
    magnitude_peaks
        .into_iter()
        .map(ipm)
        .collect::<Vec<(f32, f32, f32)>>()
}

/// Converts the first element in the tuple from an index to a frequency
fn indice_to_freq(ind: f32, sampling_rate: i32, window_len: usize) -> f32 {
    sampling_rate as f32 * (ind / window_len as f32)
}


fn process_candidate(f: f32, indexed_peakfreq_mag_phase: Vec<(usize, (f32,f32,f32))>) -> Vec<(usize, (f32,f32,f32))> {
    let mut short_list = indexed_peakfreq_mag_phase.into_iter()
        .filter(|(_, (freqs, _, _))| (freqs - f).abs() < f / 2.0)
        .collect::<Vec<(usize, (f32, f32, f32))>>();
    let max_mag = indexed_peakfreq_mag_phase.iter()
        .max_by(|(_, (_, mag1, _)), (_, (_, mag2, _))| mag1.partial_cmp(&mag2).unwrap())
        .unwrap();
    let max_freq = max_mag.1.2 % f;

    if max_freq > f / 2.0 {
        if !indexed_peakfreq_mag_phase.contains(max_mag) && (f - max_freq) > (f / 4.0) {
            short_list.push(*max_mag);
            short_list
        } else { short_list }
    } else {
        if !indexed_peakfreq_mag_phase.contains(max_mag) && max_freq > (f / 4.0) {
            short_list.push(*max_mag);
            short_list
        } else { short_list }
    }
}



fn f0twm(twm_conf: TWMConfig, peakfreq_mag_phase: Vec<(f32, f32, f32)>, last_candidate: Option<f32>) -> Option<f32> {
    if peakfreq_mag_phase.len() < 3 && last_candidate.is_none() {
        return None
    }
    let indexed_within_range = peakfreq_mag_phase.into_iter()
        .enumerate()
        .filter(|(_, (peakfreq, _, _))| peakfreq > &twm_conf.min_freq && peakfreq < &twm_conf.max_freq)
        .collect::<Vec<(usize, (f32,f32,f32))>>();

    if indexed_within_range.is_empty() == 0 {
        return None
    }
    let f0cf = match last_candidate {
        None => indexed_within_range,
        Some(f) => process_candidate(f, indexed_within_range)
    };
    if f0cf.is_empty() {
        return None
    };

    let (cand, err) = twm(twm_conf.hyper_params, peakfreq_mag_phase, f0cf);
    if cand == 0 {
        return None
    };
    if err > twm_conf.max_error {
        return None
    };
    return Some(cand)
}

fn twm(hyper_params: &HyperParams, peakfreq_mag_phase: Vec<(f32, f32, f32)>, previous_candidates: Vec<(usize, (f32,f32,f32))>) -> (f32, f32) {
    let errors: Vec<f32> = Vec::with_capacity(previous_candidates.len());
    let max_npm = hyper_params.num_harms.min(peakfreq_mag_phase.len());

}

fn predicted_to_measured(max_npm: usize, f0_candidates: Vec<(usize, (f32,f32,f32))>, errors: Vec<f32>) -> Vec<f32> {
    for i in 0..max_npm {
        let dif_matrix = f0_candidates.into_iter().map()
    }
}

fn main() {
    // let's initialize our short-time fourier transform
    let window_type: WindowType = WindowType::Hanning;
    let window_size: usize = 1024;
    let step_size: usize = 512;
    let positive_spectrum_idx: usize = window_size / 2 + 1;

    let mut stft = STFT::<f32>::new(window_type, window_size, step_size);

    let (sender, receiver) = mpsc::channel();

    let host = cpal::default_host();
    let device = host.default_input_device().expect("no input device available");
    let config = device
        .default_input_config()
        .expect("no default config")
        .config();

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // react to stream events and read or write stream data here.
            // println!("{:?}", data);
            stft.append_samples(data);

            // as long as there remain window_size samples in the internal
            // ringbuffer of the stft
            while stft.contains_enough_to_compute() {
                // compute one column of the stft by
                // taking the first window_size samples of the internal ringbuffer,
                // multiplying them with the window,
                // computing the fast fourier transform,
                // taking half of the symetric complex outputs,
                // computing the norm of the complex outputs and
                // taking the log10
                stft.compute_into_complex_output();


                // here's where you would do something with the
                // spectrogram_column...
                match sender.send(stft.complex_buffer.clone()) {
                    Ok(_) => (),
                    Err(e) => panic!(e)
                };
                // drop step_size samples from the internal ringbuffer of the stft
                // making a step of size step_size
                stft.move_to_next_column();
            }
        },
        move |err| {
            panic!(err);
            // react to errors here.
        },
    ).unwrap();

    stream.play().unwrap();
    loop {
        match receiver.recv() {
            Ok(t) => print!("{:?}", t),
            Err(e) => panic!(e)
        }
    }
}

