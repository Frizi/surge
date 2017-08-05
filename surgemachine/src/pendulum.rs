use device::*;
use helpers;
use envelope::ADSREnvelope;
use envelope::Envelope;
use oscillator::Oscillator;
use waveform::*;
use IndexedEnum;
use frame::Frame;
use smallvec::SmallVec;

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

pub struct Pendulum {
    sample_rate: f32,
    voices: [PendulumVoice; 8],
    params: PendulumParamsBag,
    voice_cycle: u8
}


impl Default for Pendulum {
    fn default() -> Pendulum {
        Pendulum {
            sample_rate: 1.0,
            voice_cycle: 0,
            voices: Default::default(),
            params: Default::default(),
        }
    }
}

impl Pendulum {
    fn setup_envelope_1(&mut self) {
        let a = self.params.get(PendulumParams::Osc1Attack);
        let d = self.params.get(PendulumParams::Osc1Decay);
        let s = self.params.get(PendulumParams::Osc1Sustain);
        let r = self.params.get(PendulumParams::Osc1Release);
        for voice in &mut self.voices {
            let rate = self.sample_rate;
            voice.osc1.envelope.set_adsr(rate, a, d, s, r);
        }
    }

    fn setup_envelope_2(&mut self) {
        let a = self.params.get(PendulumParams::Osc2Attack);
        let d = self.params.get(PendulumParams::Osc2Decay);
        let s = self.params.get(PendulumParams::Osc2Sustain);
        let r = self.params.get(PendulumParams::Osc2Release);
        for voice in &mut self.voices {
            let rate = self.sample_rate;
            voice.osc2.envelope.set_adsr(rate, a, d, s, r);
        }
    }

    fn setup_envelope_3(&mut self) {
        let a = self.params.get(PendulumParams::Osc3Attack);
        let d = self.params.get(PendulumParams::Osc3Decay);
        let s = self.params.get(PendulumParams::Osc3Sustain);
        let r = self.params.get(PendulumParams::Osc3Release);
        for voice in &mut self.voices {
            let rate = self.sample_rate;
            voice.osc3.envelope.set_adsr(rate, a, d, s, r);
        }
    }

    fn setup_waves(&mut self) {
        let w1 = Dynamic::from_param(self.params.get(PendulumParams::Osc1Waveform));
        let w2 = Dynamic::from_param(self.params.get(PendulumParams::Osc2Waveform));
        let w3 = Dynamic::from_param(self.params.get(PendulumParams::Osc3Waveform));
        for voice in &mut self.voices {
            voice.osc1.set_wave(w1);
            voice.osc2.set_wave(w2);
            voice.osc3.set_wave(w3);
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
    fn process_sample(&mut self, timestep: f32) -> Frame;
    fn is_finished (&self) -> bool;
}

impl Voice<PendulumParamsBag> for PendulumVoice {
    fn current_note (&self) -> Option<u8> { self.current_note }
    fn is_finished (&self) -> bool {
        self.osc1.is_finished() &&
        self.osc2.is_finished() &&
        self.osc3.is_finished()
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        self.current_note = Some(note);
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

    fn init_process(&mut self, params: &PendulumParamsBag) -> bool {
        match self.current_note {
            Some(note) => {
                let note_freq : f32 = helpers::midi_note_to_hz(note);

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

                let detune1 = (params.get(PendulumParams::Osc1Detune) - 0.5) * 0.02 + 1.0;
                let detune2 = (params.get(PendulumParams::Osc2Detune) - 0.5) * 0.02 + 1.0;
                let detune3 = (params.get(PendulumParams::Osc3Detune) - 0.5) * 0.02 + 1.0;

                let phase_offset1 = params.get(PendulumParams::Osc1PhaseOffset);
                let phase_offset2 = params.get(PendulumParams::Osc2PhaseOffset);
                let phase_offset3 = params.get(PendulumParams::Osc3PhaseOffset);

                self.osc2_level = params.get(PendulumParams::Osc2Level);
                self.osc3_level = params.get(PendulumParams::Osc3Level);
                self.osc3_am = params.get(PendulumParams::Osc3AM) > 0.5;

                self.osc1.setup(freq1, detune1, phase_offset1);
                self.osc2.setup(freq2, detune2, phase_offset2);
                self.osc3.setup(freq3, detune3, phase_offset3);

                true
            }
            _ => !self.is_finished()
        }
    }

    #[inline]
    fn process_sample(&mut self, timestep: f32) -> Frame {
        let s1 = self.osc1.process_sample(timestep);
        let s2 = self.osc2.process_sample(timestep) * self.osc2_level;
        let s3 = self.osc3.process_sample(timestep) * self.osc3_level;

        if self.osc3_am {
            (s1 + s2) * s3
        } else {
            s1 + s2 + s3
        }
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

    fn setup(&mut self, freq : f32, detune : f32, phase_offset : f32) {
        self.phase_offset = phase_offset;
        let detune_off =  (1.0 - detune.abs()) < 0.001;

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

impl Device for Pendulum {
    fn get_num_parameters (&self) -> i32 { PendulumParams::NUM_ITEMS as i32 }

    fn get_parameter (&self, index: i32) -> f32 {
        let param = PendulumParams::from_index(index as u32);
        let val = self.params.get(param);

        match param {
            PendulumParams::Osc1RatioCoarse => (val * 32.99).floor() / 32.0,
            PendulumParams::Osc2RatioCoarse => (val * 32.99).floor() / 32.0,
            _ => val
        }
    }

    fn set_parameter (&mut self, index: i32, value: f32) {
        let param = PendulumParams::from_index(index as u32);
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

    fn set_sample_rate (&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.setup_envelope_1();
        self.setup_envelope_2();
        self.setup_envelope_3();
    }

    fn run (&mut self, mut outputs : AudioBus<f32>) {
        let timestep = helpers::time_per_sample(self.sample_rate);

        if !self.is_finished() {
            let master = self.params.get(PendulumParams::MasterLevel);
            let params = &self.params;


            let mut active_voices : SmallVec<[&mut PendulumVoice; 8]> = Default::default();

            for voice in self.voices.iter_mut() {
                if voice.init_process(&params) {
                    active_voices.push(voice);
                }
            }

            for (left_sample, right_sample) in helpers::frame_iter(&mut outputs) {
                let signal = active_voices.iter_mut()
                    .map(|voice| voice.process_sample(timestep))
                    .sum::<Frame>() * master;

                *left_sample = signal.l;
                *right_sample = signal.r;
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
        match self.voices.iter_mut().nth(self.voice_cycle as _) {
            Some(voice) => {
                if voice.current_note() == None {
                    voice.note_on(note, velocity);
                    self.voice_cycle = (self.voice_cycle + 1) % 8;
                    return;
                }
            },
            _ => ()
        };


        for voice in self.voices.iter_mut() {
            if voice.current_note() == None {
                voice.note_on(note, velocity);
                return;
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
