use helpers;

pub struct Sine;
pub struct Saw;
pub struct Square;
pub struct SawExp(pub f64);

pub trait Waveform {
    fn value_at_phase (&self, phase: f64) -> f64;
}

impl Waveform for Sine {
    #[inline]
    fn value_at_phase(&self, phase: f64) -> f64 {
        (helpers::TAU * phase).sin()
    }
}

impl Waveform for Saw {
    #[inline]
    fn value_at_phase(&self, phase: f64) -> f64 {
        phase * -2.0 + 1.0
    }
}

impl Waveform for SawExp {
    #[inline]
    fn value_at_phase(&self, phase: f64) -> f64 {
        let SawExp(steepness) = *self;
        let saw = Saw.value_at_phase(phase);
        saw * saw.abs().powf(steepness)
    }
}

impl Waveform for Square {
    #[inline]
    fn value_at_phase(&self, phase: f64) -> f64 {
        (if phase.fract() < 0.5 { -1.0 } else { 1.0 })
    }
}
