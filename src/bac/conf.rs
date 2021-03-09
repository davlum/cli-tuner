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

pub struct Config {
    pub(crate) amp_threshold: f32,
    pub(crate) tuning: f32,
    pub(crate) nbits: usize,
    pub samples_per_second: usize,
    pub min_period: usize,
    pub buff_size: usize,
    pub(crate) array_size: usize,
    pub(crate) mid_array: usize,
    pub(crate) mid_pos: usize
}

pub const CONFIG: Config = Config{
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

/// Calculate the smallest power of 2 greater than n.
/// Useful for getting the appropriate buffer size
pub const fn get_smallest_pow2(n: usize) -> usize {
    const fn smallest_pow2(n: usize, m: usize) -> usize {
        if m < n { smallest_pow2(n, m << 1) } else { m }
    }
    smallest_pow2(n, 1)
}
