#![allow(dead_code)]
use device::AudioBus;
use std::f32::consts::E;

pub fn fix_denormal (value: f32) -> f32 {
    return value
}

pub fn ratio_scalar (coarse: f32, fine: f32) -> f32 {
    return 1.0 + (coarse * 32.99).floor() + (fine).powf(2.0)
}

pub fn log_control (linear: f32) -> f32 {
    (linear.exp() - 1.0) / (E - 1.0)
}

pub fn unit_to_db (unit: f32) -> f32 {
    10.0 * unit.log(10.0)
}

pub fn unit_to_cents (unit: f32) -> f32 {
    (unit - 0.5) * 2400.0
}

pub fn cents_to_ratio (cents: f32) -> f32 {
    (cents / 1200.0).exp2()
}

pub fn control_to_db(linear: f32) -> f32 {
    unit_to_db(log_control(linear))
}

pub fn param_detune(param: f32) -> f32 {
    cents_to_ratio(unit_to_cents(param) * 0.1)
}

pub fn midi_note_to_hz(note: u8) -> f32 {
    const A4: f32 = 440.0;
    (A4 / 32.0) * ((note as f32 - 9.0) / 12.0).exp2()
}

pub fn time_per_sample (sample_rate: f32) -> f32 {
    sample_rate.recip()
}

pub fn frame_iter<'a, 'b, T>(mut channels: &'a mut AudioBus<'b, T>) -> impl Iterator<Item=(&'a mut T, &'a mut T)> {
    let mut out_it = channels.iter_mut();
    let left = out_it.next().unwrap();
    let right = out_it.next().unwrap();
    left.iter_mut().zip(right.iter_mut())
}
