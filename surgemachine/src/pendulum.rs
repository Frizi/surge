use helpers;
use envelope::ADSREnvelope;
use envelope::Envelope;
use oscillator::Oscillator;
use waveform::*;
use IndexedEnum;
use frame::Frame;
use params_bag::ParamsBag;
use voice::Voice;
use poly_synth::PolySynth;
use device::{Device, DevicePlugin};

type Bag = PendulumParamsBag;
pub type Pendulum = PolySynth<PendulumVoice>;

impl DevicePlugin for Pendulum {
    fn get_parameter_name(&self, param: i32) -> String {
        format!("{:?}", PendulumParams::from_index(param as _))
    }
    fn get_parameter_label(&self, param: i32) -> String {
        match PendulumParams::from_index(param as _) {
            PendulumParams::Osc2Level |
            PendulumParams::Osc3Level |
            PendulumParams::MasterLevel => "dB".to_string(),
            PendulumParams::Osc1Detune |
            PendulumParams::Osc2Detune |
            PendulumParams::Osc3Detune => "cents".to_string(),
            _ => "".to_string()
        }
    }
    fn get_parameter_text(&self, param: i32) -> String {
        let value = self.get_parameter(param);
        match PendulumParams::from_index(param as _) {
            PendulumParams::Osc1Waveform |
            PendulumParams::Osc2Waveform |
            PendulumParams::Osc3Waveform => format!("{:?}", Dynamic::from_param(value)),
            PendulumParams::Osc1RatioCoarse |
            PendulumParams::Osc2RatioCoarse |
            PendulumParams::Osc3RatioCoarse => format!("{}", (value * 32.99).floor()),
            PendulumParams::Osc1Detune |
            PendulumParams::Osc2Detune |
            PendulumParams::Osc3Detune => format!("{:.0}", (helpers::unit_to_cents(value) * 0.1).round()),
            PendulumParams::Osc2Level |
            PendulumParams::Osc3Level |
            PendulumParams::MasterLevel => format!("{:.0}", helpers::control_to_db(value)),
            PendulumParams::Osc3AM => format!("{:?}", value > 0.5),
            _ => format!("{:.3}", value),
        }
    }
}

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
    0.3, 0.0, 0.0, 1.0, // osc2
    1.0, 0.0, 0.0, 0.0, // osc3
    0.2, 0.1, 1.0, 0.2, // osc1 envelope
    0.1, 0.1, 1.0, 0.2, // osc2 envelope
    0.01, 0.1, 0.0, 0.1, // osc3 envelope
    0.5, 0.5, 0.5, 0.0, // detunes + am
    1.0, 1.0, 0.5, // levels
]);

#[derive(Default)]
pub struct PendulumVoice {
    osc1: PendulumOsc,
    osc2: PendulumOsc,
    osc3: PendulumOsc,
    current_note: Option<u8>,
    osc2_level: f32,
    osc3_level: f32,
    osc3_am: bool,
    velocity: f32,
}

impl PendulumVoice {
    fn setup_envelopes(&mut self, params: &Bag, rate: f32) {
        let a1 = params.get(PendulumParams::Osc1Attack);
        let d1 = params.get(PendulumParams::Osc1Decay);
        let s1 = params.get(PendulumParams::Osc1Sustain);
        let r1 = params.get(PendulumParams::Osc1Release);
        let a2 = params.get(PendulumParams::Osc2Attack);
        let d2 = params.get(PendulumParams::Osc2Decay);
        let s2 = params.get(PendulumParams::Osc2Sustain);
        let r2 = params.get(PendulumParams::Osc2Release);
        let a3 = params.get(PendulumParams::Osc3Attack);
        let d3 = params.get(PendulumParams::Osc3Decay);
        let s3 = params.get(PendulumParams::Osc3Sustain);
        let r3 = params.get(PendulumParams::Osc3Release);
        self.osc1.envelope.set_adsr(rate, a1, d1, s1, r1);
        self.osc2.envelope.set_adsr(rate, a2, d2, s2, r2);
        self.osc3.envelope.set_adsr(rate, a3, d3, s3, r3);
    }

    fn setup_waves(&mut self, params: &Bag) {
        let w1 = Dynamic::from_param(params.get(PendulumParams::Osc1Waveform));
        let w2 = Dynamic::from_param(params.get(PendulumParams::Osc2Waveform));
        let w3 = Dynamic::from_param(params.get(PendulumParams::Osc3Waveform));
        self.osc1.set_wave(w1);
        self.osc2.set_wave(w2);
        self.osc3.set_wave(w3);
    }
}

impl Voice for PendulumVoice {
    type ParamsEnum = PendulumParams;
    type Bag = Bag;
    type PostParam = f32;
    type Depth = f32;

    fn init (&mut self, params: &Bag, rate: f32) {
        self.setup_envelopes(params, rate);
        self.setup_waves(params);
    }

    fn current_note (&self) -> Option<u8> { self.current_note }
    fn is_finished (&self) -> bool {
        self.osc1.is_finished() &&
        self.osc2.is_finished() &&
        self.osc3.is_finished()
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        self.current_note = Some(note);
        self.velocity = (velocity as f32 / 127.0).min(1.0);
        self.osc1.trigger();
        self.osc2.trigger();
        self.osc3.trigger();
    }

    fn note_off(&mut self, _note: u8, _velocity: u8) {
        self.current_note = None;
        self.osc1.release();
        self.osc2.release();
        self.osc3.release();
    }
    fn prepare_post(params: &Bag) -> f32 {
        helpers::log_control(params.get(PendulumParams::MasterLevel))
    }

    fn process_post(data: &f32, frame: Frame) -> Frame {
        frame * *data
    }

    fn init_process(&mut self, params: &Bag) -> bool {
        match self.current_note {
            Some(note) => {
                let note_freq: f32 = helpers::midi_note_to_hz(note);

                let freq1 = note_freq * helpers::ratio_scalar(
                    params.get(PendulumParams::Osc1RatioCoarse),
                    params.get(PendulumParams::Osc1RatioFine),
                );
                let freq2 = note_freq * helpers::ratio_scalar(
                    params.get(PendulumParams::Osc2RatioCoarse),
                    params.get(PendulumParams::Osc2RatioFine)
                );
                let freq3 = note_freq * helpers::ratio_scalar(
                    params.get(PendulumParams::Osc3RatioCoarse),
                    params.get(PendulumParams::Osc3RatioFine)
                );

                let detune1 = helpers::param_detune(params.get(PendulumParams::Osc1Detune));
                let detune2 = helpers::param_detune(params.get(PendulumParams::Osc2Detune));
                let detune3 = helpers::param_detune(params.get(PendulumParams::Osc3Detune));

                let phase_offset1 = params.get(PendulumParams::Osc1PhaseOffset);
                let phase_offset2 = params.get(PendulumParams::Osc2PhaseOffset);
                let phase_offset3 = params.get(PendulumParams::Osc3PhaseOffset);

                self.osc2_level = helpers::log_control(params.get(PendulumParams::Osc2Level));
                self.osc3_level = helpers::log_control(params.get(PendulumParams::Osc3Level));
                self.osc3_am = params.get(PendulumParams::Osc3AM) > 0.5;

                self.osc1.setup(freq1, detune1, phase_offset1);
                self.osc2.setup(freq2, detune2, phase_offset2);
                self.osc3.setup(freq3, detune3, phase_offset3);

                true
            }
            _ => !self.is_finished()
        }
    }

    fn update_param(&mut self, bag: &Bag, param: PendulumParams, rate: f32) {
        match param {
            PendulumParams::Osc1Attack => self.setup_envelopes(bag, rate),
            PendulumParams::Osc1Decay => self.setup_envelopes(bag, rate),
            PendulumParams::Osc1Sustain => self.setup_envelopes(bag, rate),
            PendulumParams::Osc1Release => self.setup_envelopes(bag, rate),
            PendulumParams::Osc2Attack => self.setup_envelopes(bag, rate),
            PendulumParams::Osc2Decay => self.setup_envelopes(bag, rate),
            PendulumParams::Osc2Sustain => self.setup_envelopes(bag, rate),
            PendulumParams::Osc2Release => self.setup_envelopes(bag, rate),
            PendulumParams::Osc3Attack => self.setup_envelopes(bag, rate),
            PendulumParams::Osc3Decay => self.setup_envelopes(bag, rate),
            PendulumParams::Osc3Sustain => self.setup_envelopes(bag, rate),
            PendulumParams::Osc3Release => self.setup_envelopes(bag, rate),
            PendulumParams::Osc1Waveform => self.setup_waves(bag),
            PendulumParams::Osc2Waveform => self.setup_waves(bag),
            PendulumParams::Osc3Waveform => self.setup_waves(bag),
            _ => (),
        };
    }

    #[inline]
    fn process_sample(&mut self, timestep: f32) -> Frame {
        let s1 = self.osc1.process_sample(timestep);
        let s2 = self.osc2.process_sample(timestep) * self.osc2_level;
        let s3 = self.osc3.process_sample(timestep) * self.osc3_level;

        (if self.osc3_am {
            (s1 + s2) * (s3 * 0.5 + 0.5)
        } else {
            s1 + s2 + s3
        }) * self.velocity
    }
}

enum PendulumOscMode {
    Mono,
    MonoOffset,
    Stereo
}

impl Default for PendulumOscMode {
    fn default () -> Self { PendulumOscMode::Mono }
}

#[derive(Default)]
struct PendulumOsc {
    envelope: ADSREnvelope,
    osc_l: Oscillator,
    osc_r: Oscillator,
    phase_offset: f32,
    osc_mode: PendulumOscMode
}

impl PendulumOsc {
    #[inline]
    fn process_sample(&mut self, timestep: f32) -> Frame {
        let env = self.envelope.get_value();
        self.envelope.process();
        match self.osc_mode {
            PendulumOscMode::Mono => {
                let val = self.osc_l.get_value();
                let f = Frame {
                    l: val,
                    r: val,
                } * env;
                self.osc_l.step(timestep);
                f
            },
            PendulumOscMode::MonoOffset => {
                let f = Frame {
                    l: self.osc_l.get_value(),
                    r: self.osc_l.get_offset_value(self.phase_offset),
                } * env;
                self.osc_l.step(timestep);
                f
            }

            PendulumOscMode::Stereo => {
                let f = Frame {
                    l: self.osc_l.get_value(),
                    r: self.osc_r.get_offset_value(self.phase_offset),
                } * env;
                self.osc_l.step(timestep);
                self.osc_r.step(timestep);
                f
            }
        }
    }

    fn set_wave(&mut self, wave: Dynamic) {
        self.osc_l.set_wave(wave);
        self.osc_r.set_wave(wave);
    }

    fn trigger(&mut self) {
        self.envelope.trigger();
        self.osc_l.phase_reset();
        self.osc_r.phase_reset();
    }

    fn setup(&mut self, freq: f32, detune: f32, phase_offset: f32) {
        self.phase_offset = phase_offset;
        let detune_off =  (1.0 - detune).abs() < 0.001;

        if detune_off {
            self.osc_l.set_freq(freq);
            self.osc_r.set_freq(freq);
            self.osc_mode = if phase_offset == 0.0 {
                PendulumOscMode::Mono
            } else {
                PendulumOscMode::MonoOffset
            };
        } else {
            self.osc_l.set_freq(freq / detune);
            self.osc_r.set_freq(freq * detune);
            self.osc_mode = PendulumOscMode::Stereo;
        }
    }

    fn release(&mut self) {
        self.envelope.release();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }
}
