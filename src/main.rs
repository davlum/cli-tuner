mod test;

extern crate cpal;

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use std::io::{self, Write};
use std::{thread, time};
use colored::{ColoredString, Colorize};

const MIN_FREQ: usize = 50;
const MAX_FREQ: usize = 500;
const NBITS: usize = core::mem::size_of::<u32>() * 8;
const SAMPLES_PER_SECOND: usize = 44100;
const MIN_PERIOD: usize = SAMPLES_PER_SECOND / MAX_FREQ;
const MAX_PERIOD: usize = SAMPLES_PER_SECOND / MIN_FREQ;
const BUFF_SIZE: usize = get_smallest_pow2(MAX_PERIOD) * 2;
const ARRAY_SIZE: usize = BUFF_SIZE / NBITS;
const MID_ARRAY: usize = ((ARRAY_SIZE / 2) - 1) as usize ;
const MID_POS: usize = (BUFF_SIZE / 2) as usize;

struct Config {
    amp_threshold: f32,
    tuning: f32,
    nbits: usize,
    samples_per_second: usize,
    min_period: usize,
    buff_size: usize,
    array_size: usize,
    mid_array: usize,
    mid_pos: usize
}

const CONFIG: Config = Config{
    amp_threshold: -50.0,
    tuning: 444.0,
    nbits: NBITS,
    samples_per_second: SAMPLES_PER_SECOND,
    min_period: MIN_PERIOD,
    buff_size: BUFF_SIZE,
    array_size: ARRAY_SIZE,
    mid_array: MID_ARRAY,
    mid_pos: MID_POS
};

const GREETING: &str = r#"
 *******  **           **     **    **   ********** **      ** ********   ********** **      ** ** ****     **   ********
/**////**/**          ****   //**  **   /////**/// /**     /**/**/////   /////**/// /**     /**/**/**/**   /**  **//////**
/**   /**/**         **//**   //****        /**    /**     /**/**            /**    /**     /**/**/**//**  /** **      //
/******* /**        **  //**   //**         /**    /**********/*******       /**    /**********/**/** //** /**/**
/**////  /**       **********   /**         /**    /**//////**/**////        /**    /**//////**/**/**  //**/**/**    *****
/**      /**      /**//////**   /**         /**    /**     /**/**            /**    /**     /**/**/**   //****//**  ////**
/**      /********/**     /**   /**         /**    /**     /**/********      /**    /**     /**/**/**    //*** //********
//       //////// //      //    //          //     //      // ////////       //     //      // // //      ///   ////////
"#;

const C: &str = r#"
   ******
  **////**
 **    //
/**
/**
//**    **
 //******
  //////
"#;

const C_SHARP: &str = r#"
   ******
  **////**   **    **
 **    //  ************
/**       ///**////**/
/**         /**   /**
//**    ** ************
 //****** ///**////**/
  //////    //    //
"#;

const D: &str = r#"
 *******
/**////**
/**    /**
/**    /**
/**    /**
/**    **
/*******
///////
"#;

const D_SHARP: &str = r#"
 *******
/**////**    **    **
/**    /** ************
/**    /**///**////**/
/**    /**  /**   /**
/**    **  ************
/*******  ///**////**/
///////     //    //
"#;

const E: &str = r#"
 ********
/**/////
/**
/*******
/**////
/**
/********
////////
"#;

const F: &str = r#"
 ********
/**/////
/**
/*******
/**////
/**
/**
//
"#;

const F_SHARP: &str = r#"
 ********
/**/////    **    **
/**       ************
/******* ///**////**/
/**////    /**   /**
/**       ************
/**      ///**////**/
//         //    //
"#;

const G: &str = r#"
   ********
  **//////**
 **      //
/**
/**    *****
//**  ////**
 //********
  ////////
"#;

const G_SHARP: &str = r#"
   ********
  **//////**   **    **
 **      //  ************
/**         ///**////**/
/**    *****  /**   /**
//**  ////** ************
 //******** ///**////**/
  ////////    //    //
"#;

const A: &str = r#"
     **
    ****
   **//**
  **  //**
 **********
/**//////**
/**     /**
//      //
"#;

const A_SHARP: &str = r#"
     **
    ****      **    **
   **//**   ************
  **  //** ///**////**/
 **********  /**   /**
/**//////** ************
/**     /**///**////**/
//      //   //    //
"#;

const B: &str = r#"
 ******
/*////**
/*   /**
/******
/*//// **
/*    /**
/*******
///////
"#;

const FLAT: &str = r#"
     **
   **/ **
 **   // **
//      //
"#;

const SHARP: &str = r#"
/**   /**
//** /**
 //****
  //**
"#;

const NOTES: [&str; 12] = [C, C_SHARP, D, D_SHARP, E, F, F_SHARP, G, G_SHARP, A, A_SHARP, B];

struct Bitstream {
    bits: [u32; ARRAY_SIZE]
}

#[derive(Clone, Debug)]
struct ZeroCross {
    y: bool
}

impl ZeroCross {
    fn new() -> Self {
        ZeroCross { y: false }
    }

    fn run(&mut self, s: f32) -> bool {
        if s < -0.1 {
            self.y = false
        }
        if s > 0.0 {
            self.y = true
        }
        self.y
    }
}

/// Calculate the smallest power of 2 greater than n.
/// Useful for getting the appropriate buffer size
const fn get_smallest_pow2(n: usize) -> usize {
    const fn smallest_pow2(n: usize, m: usize) -> usize {
        if m < n { smallest_pow2(n, m << 1) } else { m }
    }
    smallest_pow2(n, 1)
}

impl Bitstream {

    fn new() -> Self {
        Bitstream { bits: [0; CONFIG.array_size] }
    }

    // fn clear(&mut self) {
    //     self.bits.iter_mut().for_each(|x| *x = 0)
    // }

    fn get(&self, i: usize) -> bool {
        let mask = 1 << (i % CONFIG.nbits);
        (self.bits[i / CONFIG.nbits] & mask) != 0
    }

    fn set(&mut self, i: usize, val: bool) {
        // Gets the section of 32 bits
        // where i resides
        let bs = &mut self.bits[i / CONFIG.nbits];

        // Creates a bitmask the 1 is at
        // the location of interest in the 32 bits
        let mask = 1 << (i % CONFIG.nbits);

        // will be either all zeros or all ones.
        // All zeros is identity element with XOR
        let id= if val { u32::MAX } else { 0 };
        *bs ^= (id ^ *bs) & mask;
    }

    fn autocorrelate(&self, start_pos: usize) -> (u32, usize, [u32; CONFIG.mid_pos]) {

        let mut corr = [0; CONFIG.mid_pos];
        let mut max_count = 0;
        let mut min_count = u32::MAX;
        let mut est_index = 0;
        let mut index = start_pos / CONFIG.nbits;
        let mut shift = start_pos % CONFIG.nbits;

        for pos in start_pos..CONFIG.mid_pos {
            let mut p1 = 0;
            let mut p2 = index;
            let mut count = 0;
            if shift == 0 {
                for _ in 0..CONFIG.mid_array {
                    count += (self.bits[p1] ^ self.bits[p2]).count_ones();
                    p1 += 1;
                    p2 += 1;
                }
            } else {
                let shift2 = CONFIG.nbits - shift;
                for _ in 0..CONFIG.mid_array {
                    let mut v = self.bits[p2] >> shift;
                    p2 += 1;
                    v |= self.bits[p2] << shift2;
                    count += (self.bits[p1] ^ v).count_ones();
                    p1 += 1;

                }
            }
            shift += 1;
            if shift == CONFIG.nbits {
                shift = 0;
                index += 1;
            }

            corr[pos] = count;
            max_count = max_count.max(count);
            if count < min_count {
                min_count = count;
                est_index = pos;
            }
        }
        (max_count, est_index, corr)
    }

    fn handle_harmonics(max_count: u32, est_index: usize, corr: &mut [u32; CONFIG.mid_pos]) -> usize {
        let sub_threshold = 0.15 * max_count as f32;
        let max_div = est_index / CONFIG.min_period;
        let mut est_index = est_index as f32;
        for div in (0..max_div).rev() {
            let mut all_strong = true;
            let mul = 1.0 / div as f32;
            for k in 1..div {
                let sub_period = k + (est_index * mul) as usize;
                if corr[sub_period] > sub_threshold as u32 {
                    all_strong = false;
                    break;
                }
            }
            if all_strong {
                est_index = est_index * mul;
                break;
            }
        }
        return est_index as usize
    }

    fn estimate_pitch_with_index(signal: &[f32], est_index: usize) -> Option<f32> {
        if est_index >= CONFIG.buff_size {
            return None
        }
        let mut prev: f32 = 0.0;
        let mut start_edge_index = 0;
        let mut start_edge = signal[start_edge_index];
        while start_edge <= 0.0 {
            prev = start_edge;
            start_edge_index += 1;
            if start_edge_index >= CONFIG.buff_size {
                return None
            }
            start_edge = signal[start_edge_index]
        }

        let dy1 = start_edge - prev;
        let dx1 = -prev / dy1;

        let mut next_edge_index = est_index - 1;
        let mut next_edge = signal[next_edge_index];
        while next_edge <= 0.0 {
            prev = next_edge;
            next_edge_index += 1;
            if next_edge_index >= CONFIG.buff_size {
                return None
            }
            next_edge = signal[next_edge_index]
        }
        let dy2 = next_edge - prev;
        let dx2 = -prev / dy2;

        let n_samples = (next_edge_index - start_edge_index) as f32 + (dx2 - dx1);
        Some(CONFIG.samples_per_second as f32 / n_samples)
    }

    fn estimate_pitch(signal: &[f32]) -> Option<f32> {
        let mut zc = ZeroCross::new();
        let mut bs = Bitstream::new();
        for i in 0..CONFIG.buff_size {
            bs.set(i, zc.run(signal[i]));
        }
        let (count, est_index, mut corr) = bs.autocorrelate(CONFIG.min_period);
        let est_index = Bitstream::handle_harmonics(count, est_index, &mut corr);
        Bitstream::estimate_pitch_with_index(signal, est_index)
    }
}

fn linear_to_db(freq: f32) -> f32 {
    20.0 * freq.abs().log10()
}

fn freq_to_note<'a>(freq: f32) -> (&'a str, i32) {
    let note_with_cents = 12.0 * (freq / CONFIG.tuning).log2() + 69.0;
    let midi_note = note_with_cents.round();
    let target_freq = 2.0f32.powf((midi_note - 69.0) / 12.0) * CONFIG.tuning;
    let note = NOTES[midi_note as usize % 12];
    let cents = (1200.0 * (freq / target_freq).log2()).round() as i32;
    (note, cents)
}

fn cents_to_color(note: &str, cents: i32) -> ColoredString {
    match cents {
        i32::MIN..=-31 => note.red(),
        -30..=-9       => note.yellow(),
        -10..=10       => note.green(),
        11..=30        => note.yellow(),
        31..=i32::MAX  => note.red()
    }
}

fn print_message(note: &str, cents: i32) {
    if cents < 0 {
        print!("{}", cents_to_color(FLAT, cents));
        print!("{}", cents_to_color(note, cents));
    } else {
        print!("\n\n\n\n\n");
        print!("{}", cents_to_color(note, cents));
        print!("{}", cents_to_color(SHARP, cents));
    }
}

fn process_signal(signal: &mut Vec<f32>, data: &[f32]) {
    for d in data.iter() {
        signal.push(*d);
    }
    if signal.len() >= CONFIG.buff_size {
        let slice = &signal[0..CONFIG.buff_size];
        let avg: f32 = slice.iter().fold(0.0, |x, y| x + y) / CONFIG.buff_size as f32;
        if linear_to_db(avg) > CONFIG.amp_threshold {
            let est_freq = Bitstream::estimate_pitch(slice);
            est_freq.map(|f| {
                let (note, cents) = freq_to_note(f);
                print!("\x1B[2J\x1B[1;1H"); // clear terminal
                print_message(note, cents);
                io::stdout().flush().unwrap();
                thread::sleep(time::Duration::from_millis(100));
            });
        }
        signal.clear();
    }
}

fn main() {
    // lowest frequency determines buf_size. We need twice the period worth of samples
    // https://www.cycfi.com/2018/04/fast-and-efficient-pitch-detection-bliss/

    // let (sender, receiver) = mpsc::channel();
    let host = cpal::default_host();
    let device = host.default_input_device().expect("no input device available");
    let config = device
        .default_input_config()
        .expect("no default config")
        .config();

    let mut signal = Vec::with_capacity(CONFIG.buff_size);
    print!("\x1B[2J\x1B[1;1H");
    print!("{}", GREETING.red());

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| process_signal(&mut signal, data),
        move |err| { panic!(err); },
    ).unwrap();

    stream.play().unwrap();
    loop {}
}

