pub mod bac;

extern crate cpal;

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use std::io::{self, Write};
use std::{thread, time};
use colored::{ColoredString, Colorize};
use crate::bac::conf::CONFIG;
use crate::bac::imp::Bitstream;


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

pub fn main() {
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

