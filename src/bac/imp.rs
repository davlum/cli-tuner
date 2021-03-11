extern crate cpal;

use crate::bac::conf::CONFIG;


pub struct Bitstream<'a> {
    bits: &'a mut [u32; CONFIG.array_size]
}

#[derive(Clone, Debug)]
pub struct ZeroCross {
    y: bool
}

impl ZeroCross {
    pub fn new() -> Self {
        ZeroCross { y: false }
    }

    pub fn run(&mut self, s: f32) -> bool {
        if s < -0.1 {
            self.y = false
        }
        if s > 0.0 {
            self.y = true
        }
        self.y
    }
}

impl<'a> Bitstream<'a> {

    pub fn new(bits: &'a mut [u32; CONFIG.array_size]) -> Self {
        Bitstream { bits }
    }

    // fn clear(&mut self) {
    //     self.bits.iter_mut().for_each(|x| *x = 0)
    // }

    pub fn get(&self, i: usize) -> bool {
        let mask = 1 << (i % CONFIG.nbits);
        (self.bits[i / CONFIG.nbits] & mask) != 0
    }

    pub fn set(&mut self, i: usize, val: bool) {
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

    pub fn autocorrelate(&self, start_pos: usize) -> (u32, usize, [u32; CONFIG.mid_pos]) {

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

    pub fn handle_harmonics(max_count: u32, est_index: usize, corr: &mut [u32; CONFIG.mid_pos]) -> usize {
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

    pub fn estimate_pitch(bits: &mut [u32; CONFIG.array_size], signal: &[f32]) -> Option<f32> {
        let mut zc = ZeroCross::new();
        let mut bs = Bitstream::new(bits);
        for i in 0..CONFIG.buff_size {
            bs.set(i, zc.run(signal[i]));
        }
        let (count, est_index, mut corr) = bs.autocorrelate(CONFIG.min_period);
        let est_index = Bitstream::handle_harmonics(count, est_index, &mut corr);
        Bitstream::estimate_pitch_with_index(signal, est_index)
    }
}
