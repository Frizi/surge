use waveform;

pub struct Oscillator {
    phase: f64,
    frequency: f64,
    wave: Box<waveform::Waveform>
}

impl Default for Oscillator {
    fn default () -> Oscillator {
        Oscillator {
            phase: 0.0,
            frequency: 0.0,
            wave: Box::new(waveform::Sine)
        }
    }
}

impl Oscillator {
    pub fn step (&mut self, timestep: f64) -> () {
        self.phase = (self.phase + timestep * self.frequency).fract();
    }

    pub fn set_freq(&mut self, freq: f64) -> () {
        self.frequency = freq;
    }

    pub fn get_value (&self) -> f64 {
        self.wave.value_at_phase(self.phase)
    }

    pub fn set_wave(&mut self, wave: Box<waveform::Waveform>) -> () {
        self.wave = wave
    }
}
