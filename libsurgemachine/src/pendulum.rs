use device::*;
use helpers;
use envelope::ADSREnvelope;
use envelope::Envelope;
use oscillator::Oscillator;
use waveform;

#[derive(Debug)]
pub enum ParamIndices {
    Osc1Waveform = 0,
    Osc1RatioCoarse,
    Osc1RatioFine,

    Osc1Attack,
    Osc1Decay,
    Osc1Sustain,
    Osc1Release,

    Osc2Waveform,
    Osc2RatioCoarse,
    Osc2RatioFine,

    Osc2Attack,
    Osc2Decay,
    Osc2Sustain,
    Osc2Release,

    MasterLevel,

    #[allow(non_camel_case_types)]
    LAST_PARAM
}

impl ParamIndices {
    pub fn from_i32 (num: i32) -> ParamIndices {
        match num {
            0 => ParamIndices::Osc1Waveform,
            1 => ParamIndices::Osc1RatioCoarse,
            2 => ParamIndices::Osc1RatioFine,

            3 => ParamIndices::Osc1Attack,
            4 => ParamIndices::Osc1Decay,
            5 => ParamIndices::Osc1Sustain,
            6 => ParamIndices::Osc1Release,

            7 => ParamIndices::Osc2Waveform,
            8 => ParamIndices::Osc2RatioCoarse,
            9 => ParamIndices::Osc2RatioFine,

            10 => ParamIndices::Osc2Attack,
            11 => ParamIndices::Osc2Decay,
            12 => ParamIndices::Osc2Sustain,
            13 => ParamIndices::Osc2Release,

            14 => ParamIndices::MasterLevel,
            _ => panic!("Invalid param index {}", num)
        }
    }
}

pub struct Pendulum {
    sample_rate: f64,
    note: Option<u8>,
    params: [f32; ParamIndices::LAST_PARAM as usize],
    envelope1: ADSREnvelope,
    envelope2: ADSREnvelope,
    osc1: Oscillator,
    osc2: Oscillator,
}

impl Default for Pendulum {
    fn default() -> Pendulum {
        Pendulum {
            sample_rate: 1.0,
            note: None,
            params: [
                0.0, 0.0, 0.0,
                0.5, 0.0, 1.0, 0.5,
                0.0, 0.0, 1.0,
                0.2, 0.0, 1.0, 0.2,
                0.5,
           ],
           envelope1: Default::default(),
           envelope2: Default::default(),
           osc1: Default::default(),
           osc2: Default::default(),
        }
    }
}

impl Pendulum {
    fn param (&self, index: ParamIndices) -> f32 {
        self.params[index as usize]
    }

    fn setup_envelope_1(&mut self) {
        let a = self.param(ParamIndices::Osc1Attack) as f64;
        let d = self.param(ParamIndices::Osc1Decay) as f64;
        let s = self.param(ParamIndices::Osc1Sustain) as f64;
        let r = self.param(ParamIndices::Osc1Release) as f64;
        let rate = self.sample_rate;
        self.envelope1.set_adsr(rate, a, d, s, r);
    }

    fn setup_envelope_2(&mut self) {
        let a = self.param(ParamIndices::Osc2Attack) as f64;
        let d = self.param(ParamIndices::Osc2Decay) as f64;
        let s = self.param(ParamIndices::Osc2Sustain) as f64;
        let r = self.param(ParamIndices::Osc2Release) as f64;
        let rate = self.sample_rate;
        self.envelope2.set_adsr(rate, a, d, s, r);
    }

    fn select_waveform (value: f32) -> Box<waveform::Waveform> {
        let sel = (value * 5.99).floor() as u32;
        match sel {
            0 => Box::new(waveform::Sine),
            1 => Box::new(waveform::Saw),
            2 => Box::new(waveform::Square),
            3 => Box::new(waveform::SawExp(0.5)),
            4 => Box::new(waveform::SawExp(1.2)),
            5 => Box::new(waveform::SawExp(2.0)),
            _ => panic!("Wrong waveform type {}", sel)
        }
    }

    fn is_finished (&self) -> bool {
        self.envelope1.is_finished() &&
        self.envelope2.is_finished()
    }
}

impl Device for Pendulum {
    fn get_num_parameters (&self) -> i32 { ParamIndices::LAST_PARAM as i32 }
    fn get_parameter (&self, index: i32) -> f32 {
        let val = self.params[index as usize];
        match ParamIndices::from_i32(index) {
            ParamIndices::Osc1RatioCoarse => (val * 32.99).floor() / 32.0,
            ParamIndices::Osc2RatioCoarse => (val * 32.99).floor() / 32.0,
            _ => val
        }
    }

    fn set_parameter (&mut self, index: i32, value: f32) {
        self.params[index as usize] = value;
        match ParamIndices::from_i32(index) {
            ParamIndices::Osc1Attack => self.setup_envelope_1(),
            ParamIndices::Osc1Decay => self.setup_envelope_1(),
            ParamIndices::Osc1Sustain => self.setup_envelope_1(),
            ParamIndices::Osc1Release => self.setup_envelope_1(),
            ParamIndices::Osc2Attack => self.setup_envelope_2(),
            ParamIndices::Osc2Decay => self.setup_envelope_2(),
            ParamIndices::Osc2Sustain => self.setup_envelope_2(),
            ParamIndices::Osc2Release => self.setup_envelope_2(),
            ParamIndices::Osc1Waveform => self.osc1.set_wave(Self::select_waveform(value)),
            ParamIndices::Osc2Waveform => self.osc2.set_wave(Self::select_waveform(value)),
            _ => (),
        };
    }

    fn set_sample_rate (&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.setup_envelope_1();
        self.setup_envelope_2();
    }

    fn run<'a> (&mut self, mut outputs : AudioBus<'a, f32>) {
        let timestep = helpers::time_per_sample(self.sample_rate);

        if !self.is_finished() {
            if let Some(current_note) = self.note {
                let freq = helpers::midi_note_to_hz(current_note);
                let freq1 = {
                    let coarse = self.param(ParamIndices::Osc1RatioCoarse) as f64;
                    let fine = self.param(ParamIndices::Osc1RatioFine) as f64;
                    freq * helpers::ratio_scalar(coarse, fine)
                };
                let freq2 = {
                    let coarse = self.param(ParamIndices::Osc2RatioCoarse) as f64;
                    let fine = self.param(ParamIndices::Osc2RatioFine) as f64;
                    freq * helpers::ratio_scalar(coarse, fine)
                };
                let master = self.param(ParamIndices::MasterLevel) as f64;

                self.osc1.set_freq(freq1);
                self.osc2.set_freq(freq2);

                for (left_sample, right_sample) in helpers::frame_iter(&mut outputs) {
                    self.envelope1.process();
                    self.envelope2.process();

                    let osc1val = self.osc1.get_value();
                    let osc2val = self.osc2.get_value();

                    self.osc1.step(timestep);
                    self.osc2.step(timestep);

                    let alpha1 = self.envelope1.get_value();
                    let alpha2 = self.envelope2.get_value();

                    let signal = (
                        osc1val * alpha1 +
                        osc2val * alpha2
                    ) * master;

                    *left_sample = signal as f32;
                    *right_sample = signal as f32;
                }
            }
        }
        else {
            for (left_sample, right_sample) in helpers::frame_iter(&mut outputs) {
                *left_sample = 0.0;
                *right_sample = 0.0;
            }
        }
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        self.note = Some(note);
        self.envelope1.trigger();
        self.envelope2.trigger();
    }

    fn note_off(&mut self, note: u8, _velocity: u8) {
        if self.note == Some(note) {
            self.envelope1.release();
            self.envelope2.release();
        }
    }
}
