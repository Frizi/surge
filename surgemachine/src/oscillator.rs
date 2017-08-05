use waveform::*;

pub struct Oscillator {
    pub phase: f32,
    frequency: f32,
    wave: Dynamic
}

impl Default for Oscillator {
    fn default () -> Oscillator {
        Oscillator {
            phase: 0.0,
            frequency: 0.0,
            wave: Dynamic::Sine
        }
    }
}

impl Oscillator {
    #[inline]
    pub fn step (&mut self, timestep: f32) -> () {
        self.phase = (self.phase + timestep * self.frequency).fract();
    }

    pub fn set_freq(&mut self, freq: f32) -> () {
        self.frequency = freq;
    }

    #[inline]
    pub fn get_value (&self) -> f32 {
        self.wave.value_at_phase(self.phase)
    }

    #[inline]
    pub fn get_offset_value (&self, phase_offset: f32) -> f32 {
        self.wave.value_at_phase((self.phase + phase_offset).fract())
    }

    pub fn set_wave(&mut self, wave: Dynamic) -> () {
        self.wave = wave
    }

    pub fn phase_reset(&mut self) -> () {
        self.phase = 0.0;
    }
}
