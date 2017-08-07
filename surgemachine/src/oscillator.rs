use waveform::*;

pub struct Oscillator<W:Waveform=Dynamic> {
    phase: f32,
    frequency: f32,
    wave: W
}

impl<W: Waveform> Default for Oscillator<W>
where W: Default {
    fn default () -> Self {
        Self {
            phase: 0.0,
            frequency: 0.0,
            wave: W::default()
        }
    }
}

impl<W: Waveform> Oscillator<W> {
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

    pub fn set_wave(&mut self, wave: W) -> () {
        self.wave = wave
    }

    pub fn get_wave_mut(&mut self) -> &mut W {
        &mut self.wave
    }

    pub fn phase_reset(&mut self) -> () {
        self.phase = 0.0;
    }
}
