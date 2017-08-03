use device::*;
use helpers;
use envelope::ADSREnvelope;
use envelope::Envelope;
use oscillator::Oscillator;
use waveform::*;
use IndexedEnum;

#[derive(Debug, Clone, Copy, IndexedEnum)]
pub enum PendulumParams {
    Osc1Waveform,
    Osc1RatioCoarse,
    Osc1RatioFine,
    Osc1PhaseOffset,

    Osc2Waveform,
    Osc2RatioCoarse,
    Osc2RatioFine,
    Osc2PhaseOffset,

    Osc3Waveform,
    Osc3RatioCoarse,
    Osc3RatioFine,
    Osc3PhaseOffset,

    Osc1Attack,
    Osc1Decay,
    Osc1Sustain,
    Osc1Release,

    Osc2Attack,
    Osc2Decay,
    Osc2Sustain,
    Osc2Release,

    Osc3Attack,
    Osc3Decay,
    Osc3Sustain,
    Osc3Release,

    Osc1Detune,
    Osc2Detune,
    Osc3Detune,
    Osc3AM,

    Osc2Level,
    Osc3Level,
    MasterLevel,
}

define_params_bag!(PendulumParamsBag, PendulumParams, [
    0.0, 0.0, 0.0, 0.0, // osc1
    0.5, 0.0, 0.0, 0.5, // osc2
    0.0, 0.0, 0.0, 0.0, // osc3
    0.2, 0.0, 1.0, 0.2, // osc1 envelope
    0.2, 0.0, 1.0, 0.2, // osc2 envelope
    0.2, 0.0, 1.0, 0.2, // osc3 envelope
    0.0, 0.0, 0.0, 0.0, // detunes + am
    1.0, 1.0, 0.5, // levels
]);

pub struct Pendulum {
    sample_rate: f64,
    voices: [PendulumVoice; 8],
    params: PendulumParamsBag
}


impl Default for Pendulum {
    fn default() -> Pendulum {
        Pendulum {
            sample_rate: 1.0,
            voices: Default::default(),
            params: Default::default(),
        }
    }
}

impl Pendulum {
    fn setup_envelope_1(&mut self) {
        let a = self.params.get(PendulumParams::Osc1Attack) as f64;
        let d = self.params.get(PendulumParams::Osc1Decay) as f64;
        let s = self.params.get(PendulumParams::Osc1Sustain) as f64;
        let r = self.params.get(PendulumParams::Osc1Release) as f64;
        for voice in self.voices.iter_mut() {
            let rate = self.sample_rate;
            voice.osc1.envelope.set_adsr(rate, a, d, s, r);
        }
    }

    fn setup_envelope_2(&mut self) {
        let a = self.params.get(PendulumParams::Osc2Attack) as f64;
        let d = self.params.get(PendulumParams::Osc2Decay) as f64;
        let s = self.params.get(PendulumParams::Osc2Sustain) as f64;
        let r = self.params.get(PendulumParams::Osc2Release) as f64;
        for voice in self.voices.iter_mut() {
            let rate = self.sample_rate;
            voice.osc2.envelope.set_adsr(rate, a, d, s, r);
        }
    }

    fn setup_envelope_3(&mut self) {
        let a = self.params.get(PendulumParams::Osc3Attack) as f64;
        let d = self.params.get(PendulumParams::Osc3Decay) as f64;
        let s = self.params.get(PendulumParams::Osc3Sustain) as f64;
        let r = self.params.get(PendulumParams::Osc3Release) as f64;
        for voice in self.voices.iter_mut() {
            let rate = self.sample_rate;
            voice.osc3.envelope.set_adsr(rate, a, d, s, r);
        }
    }

    fn setup_waves(&mut self) {
        let w1 = Dynamic::from_param(self.params.get(PendulumParams::Osc1Waveform));
        let w2 = Dynamic::from_param(self.params.get(PendulumParams::Osc2Waveform));
        let w3 = Dynamic::from_param(self.params.get(PendulumParams::Osc3Waveform));
        for voice in self.voices.iter_mut() {
            voice.osc1.osc_l.set_wave(w1);
            voice.osc1.osc_r.set_wave(w1);
            voice.osc2.osc_l.set_wave(w2);
            voice.osc2.osc_r.set_wave(w2);
            voice.osc3.osc_l.set_wave(w3);
            voice.osc3.osc_r.set_wave(w3);
        }
    }

    fn is_finished (&self) -> bool {
        self.voices.iter().fold(true, |acc, voice| {
            acc && voice.is_finished()
        })
    }
}

#[derive(Default)]
struct PendulumVoice {
    osc1: PendulumOsc,
    osc2: PendulumOsc,
    osc3: PendulumOsc,
    current_note: Option<u8>,
    osc2_level: f32,
    osc3_level: f32,
    osc3_am: bool,
}

trait Voice<T> {
    fn current_note (&self) -> Option<u8>;
    fn note_on(&mut self, note: u8, _velocity: u8);
    fn note_off(&mut self, note: u8, _velocity: u8);
    fn init_process(&mut self, &T) -> bool { true }
    fn process_sample(&mut self, timestep: f64) -> (f32, f32);
    fn is_finished (&self) -> bool;
}

impl Voice<PendulumParamsBag> for PendulumVoice {
    fn current_note (&self) -> Option<u8> { self.current_note }
    fn is_finished (&self) -> bool {
        self.osc1.envelope.is_finished() &&
        self.osc2.envelope.is_finished() &&
        self.osc3.envelope.is_finished()
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        self.current_note = Some(note);
        self.osc1.envelope.trigger();
        self.osc2.envelope.trigger();
        self.osc3.envelope.trigger();
    }

    fn note_off(&mut self, _note: u8, _velocity: u8) {
        self.current_note = None;
        self.osc1.envelope.release();
        self.osc2.envelope.release();
        self.osc3.envelope.release();
    }

    fn init_process(&mut self, params: &PendulumParamsBag) -> bool {
        match self.current_note {
            Some(note) => {
                let note_freq = helpers::midi_note_to_hz(note);

                let osc1freq = note_freq * helpers::ratio_scalar(
                    params.get(PendulumParams::Osc1RatioCoarse) as f64,
                    params.get(PendulumParams::Osc1RatioFine) as f64
                );
                let osc2freq = note_freq * helpers::ratio_scalar(
                    params.get(PendulumParams::Osc2RatioCoarse) as f64,
                    params.get(PendulumParams::Osc2RatioFine) as f64
                );
                let osc3freq = note_freq * helpers::ratio_scalar(
                    params.get(PendulumParams::Osc3RatioCoarse) as f64,
                    params.get(PendulumParams::Osc3RatioFine) as f64
                );

                let detune1 = (params.get(PendulumParams::Osc1Detune) as f64) * 0.1 - 0.05 + 1.0;
                let detune2 = (params.get(PendulumParams::Osc2Detune) as f64) * 0.1 - 0.05 + 1.0;
                let detune3 = (params.get(PendulumParams::Osc3Detune) as f64) * 0.1 - 0.05 + 1.0;

                self.osc1.phase_offset = (params.get(PendulumParams::Osc1PhaseOffset) as f64) * 0.5;
                self.osc2.phase_offset = (params.get(PendulumParams::Osc2PhaseOffset) as f64) * 0.5;
                self.osc3.phase_offset = (params.get(PendulumParams::Osc3PhaseOffset) as f64) * 0.5;

                self.osc2_level = params.get(PendulumParams::Osc2Level);
                self.osc3_level = params.get(PendulumParams::Osc3Level);
                self.osc3_am = params.get(PendulumParams::Osc3AM) > 0.5;

                self.osc1.osc_l.set_freq(osc1freq);
                self.osc1.osc_r.set_freq(osc1freq * detune1);
                self.osc2.osc_l.set_freq(osc2freq);
                self.osc2.osc_r.set_freq(osc2freq * detune2);
                self.osc3.osc_l.set_freq(osc3freq);
                self.osc3.osc_r.set_freq(osc3freq * detune3);

                true
            }
            _ => !self.is_finished()
        }
    }

    fn process_sample(&mut self, timestep: f64) -> (f32, f32) {
        let s1 = self.osc1.process_sample(timestep);
        let s2 = self.osc2.process_sample(timestep);
        let s3 = self.osc3.process_sample(timestep);
        let l = s1.0 + self.osc2_level * s2.0;
        let r = s1.1 + self.osc2_level * s2.1;

        if self.osc3_am {
            let l = l * self.osc3_level * s3.0;
            let r = r * self.osc3_level * s3.1;
            (l, r)
        } else {
            let l = l + self.osc3_level * s3.0;
            let r = r + self.osc3_level * s3.1;
            (l, r)
        }
    }
}

#[derive(Default)]
struct PendulumOsc {
    envelope: ADSREnvelope,
    osc_l: Oscillator,
    osc_r: Oscillator,
    phase_offset: f64,
}

impl PendulumOsc {
    fn process_sample(&mut self, timestep: f64) -> (f32, f32) {
        let env = self.envelope.get_value() as f32;
        let signal_l = env * self.osc_l.get_value();
        let signal_r = env * self.osc_r.get_offset_value(self.phase_offset);

        self.envelope.process();
        self.osc_l.step(timestep);
        self.osc_r.step(timestep);

        (signal_l, signal_r)
    }
}

impl Device for Pendulum {
    fn get_num_parameters (&self) -> i32 { PendulumParams::NUM_ITEMS as i32 }

    fn get_parameter (&self, index: i32) -> f32 {
        let param = PendulumParams::from_index(index as u64);
        let val = self.params.get(param);

        match param {
            PendulumParams::Osc1RatioCoarse => (val * 32.99).floor() / 32.0,
            PendulumParams::Osc2RatioCoarse => (val * 32.99).floor() / 32.0,
            _ => val
        }
    }

    fn set_parameter (&mut self, index: i32, value: f32) {
        let param = PendulumParams::from_index(index as u64);
        self.params.set(param, value);
        match param {
            PendulumParams::Osc1Attack => self.setup_envelope_1(),
            PendulumParams::Osc1Decay => self.setup_envelope_1(),
            PendulumParams::Osc1Sustain => self.setup_envelope_1(),
            PendulumParams::Osc1Release => self.setup_envelope_1(),

            PendulumParams::Osc2Attack => self.setup_envelope_2(),
            PendulumParams::Osc2Decay => self.setup_envelope_2(),
            PendulumParams::Osc2Sustain => self.setup_envelope_2(),
            PendulumParams::Osc2Release => self.setup_envelope_2(),

            PendulumParams::Osc3Attack => self.setup_envelope_3(),
            PendulumParams::Osc3Decay => self.setup_envelope_3(),
            PendulumParams::Osc3Sustain => self.setup_envelope_3(),
            PendulumParams::Osc3Release => self.setup_envelope_3(),

            PendulumParams::Osc1Waveform => self.setup_waves(),
            PendulumParams::Osc2Waveform => self.setup_waves(),
            PendulumParams::Osc3Waveform => self.setup_waves(),
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
            let master = self.params.get(PendulumParams::MasterLevel);
            //
            // let offset1 = (self.param(PendulumParams::Osc1PhaseOffset) * 0.5) as f64;
            // let offset2 = (self.param(PendulumParams::Osc2PhaseOffset) * 0.5) as f64;

            let params = &self.params;

            let mut active_voices = vec!();
            for voice in self.voices.iter_mut() {
                if voice.init_process(&params) {
                    active_voices.push(voice)
                }
            }

            for (left_sample, right_sample) in helpers::frame_iter(&mut outputs) {
                let (signal_l, signal_r) = active_voices.iter_mut().fold((0.0, 0.0), |(l, r), voice| {
                    let (vl, vr) = voice.process_sample(timestep);
                    (l + vl, r + vr)
                });

                let signal_l = signal_l * master;
                let signal_r = signal_r * master;

                *left_sample = signal_l;
                *right_sample = signal_r;
            }
        }
        else {
            for (left_sample, right_sample) in helpers::frame_iter(&mut outputs) {
                *left_sample = 0.0;
                *right_sample = 0.0;
            }
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        for voice in self.voices.iter_mut() {
            if voice.current_note() == None {
                voice.note_on(note, velocity);
                break;
            }
        }
    }

    fn note_off(&mut self, note: u8, velocity: u8) {
        for voice in self.voices.iter_mut() {
            if voice.current_note() == Some(note) {
                voice.note_off(note, velocity);
                break;
            }
        }
    }
}
