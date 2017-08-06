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

type Bag = FermiParamsBag;
pub type Fermi = PolySynth<FermiParams, Bag, FermiVoice>;

#[derive(Debug, Clone, Copy, IndexedEnum)]
pub enum FermiParams {
    Osc1Waveform,
    Osc1RatioCoarse,
    Osc1RatioFine,
    Osc1PhaseOffset,

    Osc2Waveform,
    Osc2RatioCoarse,
    Osc2RatioFine,
    Osc2PhaseOffset,

    Osc1Attack,
    Osc1Decay,
    Osc1Sustain,
    Osc1Release,

    Osc2Attack,
    Osc2Decay,
    Osc2Sustain,
    Osc2Release,

    Osc1Level,
    Osc2Level,
    MasterLevel
}

define_params_bag!(FermiParamsBag, FermiParams, Default::default());

#[derive(Default)]
pub struct FermiVoice {
    env1: ADSREnvelope,
    env2: ADSREnvelope,
    osc1: Oscillator,
    osc2: Oscillator,
    current_note: Option<u8>,
}

impl FermiVoice {
    fn setup_envelopes(&mut self, params: &Bag, rate: f32) {
        let a1 = params.get(FermiParams::Osc1Attack);
        let d1 = params.get(FermiParams::Osc1Decay);
        let s1 = params.get(FermiParams::Osc1Sustain);
        let r1 = params.get(FermiParams::Osc1Release);
        let a2 = params.get(FermiParams::Osc2Attack);
        let d2 = params.get(FermiParams::Osc2Decay);
        let s2 = params.get(FermiParams::Osc2Sustain);
        let r2 = params.get(FermiParams::Osc2Release);
        self.env1.set_adsr(rate, a1, d1, s1, r1);
        self.env2.set_adsr(rate, a2, d2, s2, r2);
    }

    fn setup_waves(&mut self, params: &Bag) {
        let w1 = Dynamic::from_param(params.get(FermiParams::Osc1Waveform));
        let w2 = Dynamic::from_param(params.get(FermiParams::Osc2Waveform));
        self.osc1.set_wave(w1);
        self.osc2.set_wave(w2);
    }
}

impl Voice<FermiParams,Bag> for FermiVoice {
    fn init (&mut self, params: &Bag, rate: f32) {
        self.setup_envelopes(params, rate);
        self.setup_waves(params);
    }

    fn current_note (&self) -> Option<u8> { self.current_note }
    fn is_finished (&self) -> bool {
        self.env1.is_finished() &&
        self.env2.is_finished()
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        self.current_note = Some(note);
        self.env1.trigger();
        self.env1.trigger();
    }

    fn note_off(&mut self, _note: u8, _velocity: u8) {
        self.current_note = None;
        self.env1.release();
        self.env1.release();
    }

    fn process_post(params: &Bag, frame: Frame) -> Frame {
        frame * params.get(FermiParams::MasterLevel)
    }

    fn init_process(&mut self, params: &Bag) -> bool {
        match self.current_note {
            Some(note) => {
                let note_freq: f32 = helpers::midi_note_to_hz(note);

                let freq1 = note_freq * helpers::ratio_scalar(
                    params.get(FermiParams::Osc1RatioCoarse),
                    params.get(FermiParams::Osc1RatioFine),
                );
                let freq2 = note_freq * helpers::ratio_scalar(
                    params.get(FermiParams::Osc2RatioCoarse),
                    params.get(FermiParams::Osc2RatioFine)
                );


                self.osc2_level = params.get(FermiParams::Osc2Level);
                self.osc3_level = params.get(FermiParams::Osc3Level);
                self.osc3_am = params.get(FermiParams::Osc3AM) > 0.5;

                self.osc1.setup(freq1, detune1, phase_offset1);
                self.osc2.setup(freq2, detune2, phase_offset2);
                self.osc3.setup(freq3, detune3, phase_offset3);

                true
            }
            _ => !self.is_finished()
        }
    }

    fn update_param(&mut self, bag: &Bag, param: FermiParams, rate: f32) {
        match param {
            FermiParams::Osc1Attack => self.setup_envelopes(bag, rate),
            FermiParams::Osc1Decay => self.setup_envelopes(bag, rate),
            FermiParams::Osc1Sustain => self.setup_envelopes(bag, rate),
            FermiParams::Osc1Release => self.setup_envelopes(bag, rate),
            FermiParams::Osc2Attack => self.setup_envelopes(bag, rate),
            FermiParams::Osc2Decay => self.setup_envelopes(bag, rate),
            FermiParams::Osc2Sustain => self.setup_envelopes(bag, rate),
            FermiParams::Osc2Release => self.setup_envelopes(bag, rate),
            FermiParams::Osc3Attack => self.setup_envelopes(bag, rate),
            FermiParams::Osc3Decay => self.setup_envelopes(bag, rate),
            FermiParams::Osc3Sustain => self.setup_envelopes(bag, rate),
            FermiParams::Osc3Release => self.setup_envelopes(bag, rate),
            FermiParams::Osc1Waveform => self.setup_waves(bag),
            FermiParams::Osc2Waveform => self.setup_waves(bag),
            FermiParams::Osc3Waveform => self.setup_waves(bag),
            _ => (),
        };
    }

    #[inline]
    fn process_sample(&mut self, timestep: f32) -> Frame {
        let s1 = self.osc1.process_sample(timestep);
        let s2 = self.osc2.process_sample(timestep) * self.osc2_level;
        let s3 = self.osc3.process_sample(timestep) * self.osc3_level;

        // for (input_buffer, output_buffer) in inputs.iter().zip(outputs) {
        //     let mut t = self.time;
        //
        //     // let osc1Feedback = fix_denormal(self.osc1Feedback * self.osc1Feedback / 2.0);
        //     // let osc2Feedback = fix_denormal(self.osc2Feedback * self.osc2Feedback / 2.0);
        //
        //     for (_, output_sample) in input_buffer.iter().zip(output_buffer) {
        //         if let Some(current_note) = self.note {
        //             let signal = (t * midi_note_to_hz(current_note) * TAU).sin();
        //
        //             // let osc1Input = osc1Phase / currentSampleRate * TAU + fix_denormal(osc1Output * osc1Feedback);
        //             // osc1Output = fix_denormal((sin(osc1Input) + square35(osc1Input) * osc1Waveform)) * osc1Env * 13.25;
        //             //
        //             // osc2Input = osc2Phase / currentSampleRate * TAU + fix_denormal(osc1Output * osc1Feedback * 13.25) + osc1Input * osc1FeedForwardScalar;
        //             // osc2Output = fix_denormal((sin(osc2Input) + square35(osc2Input) * osc2Waveform)) * osc2Env;
        //
        //
        //             // Apply a quick envelope to the attack of the signal to avoid popping.
        //             let attack = 0.5;
        //             let alpha = if self.note_duration < attack {
        //  self.note_duration / attack
        //             } else {
        //  1.0
        //             };
        //
        //             *output_sample = (signal * alpha) as f32;
        //
        //             t += per_sample;
        //         } else {
        //             *output_sample = 0.0;
        //         }
        //     }
        // }
        //
        // self.time += samples as f64 * per_sample;
        // self.note_duration += samples as f64 * per_sample;


        if self.osc3_am {
            (s1 + s2) * s3
        } else {
            s1 + s2 + s3
        }
    }
}
