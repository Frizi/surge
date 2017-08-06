use IndexedEnum;
use rand;

pub struct Sine;
pub struct Saw;
pub struct Square;
pub struct SawExp2;
pub struct Random;

#[derive(Debug, Copy, Clone, IndexedEnum)]
pub enum Dynamic {
    Sine,
    Saw,
    Square,
    SawExp2,
    Random,
}

pub trait Waveform {
    fn value_at_phase (&self, phase: f32) -> f32;
}

impl Waveform for Dynamic {
    #[inline]
    fn value_at_phase(&self, phase: f32) -> f32 {
        match *self {
            Dynamic::Sine => Sine.value_at_phase(phase),
            Dynamic::Saw => Saw.value_at_phase(phase),
            Dynamic::Square => Square.value_at_phase(phase),
            Dynamic::SawExp2 => SawExp2.value_at_phase(phase),
            Dynamic::Random => Random.value_at_phase(phase),
        }
    }
}

use std::f32::consts::PI;
const TAU: f32 = 2.0 * PI;

impl Waveform for Sine {
    #[inline]
    fn value_at_phase(&self, phase: f32) -> f32 {
        (TAU * phase).sin()
    }
}

impl Waveform for Saw {
    #[inline]
    fn value_at_phase(&self, phase: f32) -> f32 {
        (phase * -2.0 + 1.0) as _
    }
}

impl Waveform for SawExp2 {
    #[inline]
    fn value_at_phase(&self, phase: f32) -> f32 {
        saw_exp(phase, 2.0)
    }
}

impl Waveform for Square {
    #[inline]
    fn value_at_phase(&self, phase: f32) -> f32 {
        (if phase.fract() < 0.5 { -1.0 } else { 1.0 })
    }
}


impl Waveform for Random {
    #[inline]
    fn value_at_phase(&self, _phase: f32) -> f32 {
        rand::random::<f32>() * 2.0 - 1.0
    }
}

#[inline]
fn saw_exp(phase: f32, steepness: f32) -> f32 {
    let saw = Saw.value_at_phase(phase);
    saw * saw.abs().powf(steepness)
}
