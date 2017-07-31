#![allow(dead_code)]
use device::AudioBus;

pub fn fix_denormal (value: f64) -> f64 {
    return value
}

pub fn ratio_scalar (coarse: f64, fine: f64) -> f64 {
    return 1.0 + (coarse * 32.99).floor() + (fine).powf(2.0)
}

pub fn midi_note_to_hz(note: u8) -> f64 {
    const A4: f64 = 440.0;
    (A4 / 32.0) * ((note as f64 - 9.0) / 12.0).exp2()
}

pub fn time_per_sample (sample_rate: f64) -> f64 {
    1.0 / sample_rate
}

pub const PI : f64 = 3.14159265359;
pub const TAU : f64 = PI * 2.0;

pub fn frame_iter<'a, 'b, T>(mut channels: &'a mut AudioBus<'b, T>) -> impl Iterator<Item=(&'a mut T, &'a mut T)> {
    let mut out_it = channels.iter_mut();
    let left = out_it.next().unwrap();
    let right = out_it.next().unwrap();
    left.iter_mut().zip(right.iter_mut())
}
