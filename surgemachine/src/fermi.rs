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

type Bag = FermiParamsBag;
pub type Fermi = PolySynth<FermiVoice>;

impl DevicePlugin for Fermi {
    fn get_parameter_name(&self, param: i32) -> String {
        format!("{:?}", FermiParams::from_index(param as _))
    }
    fn get_parameter_label(&self, param: i32) -> String {
        match FermiParams::from_index(param as _) {
            FermiParams::Osc1Level |
            FermiParams::MasterLevel => "dB".to_string(),
            _ => "".to_string()
        }
    }
    fn get_parameter_text(&self, param: i32) -> String {
        let value = self.get_parameter(param);
        match FermiParams::from_index(param as _) {
            FermiParams::Osc1Waveform |
            FermiParams::Osc2Waveform => format!("{:.1}", value),
            FermiParams::Osc1RatioCoarse |
            FermiParams::Osc2RatioCoarse => format!("{}", (value * 32.99).floor()),
            FermiParams::Osc1Level |
            FermiParams::MasterLevel => format!("{:.0}", helpers::control_to_db(value)),
            _ => format!("{:.3}", value),
        }
    }
}

#[derive(Debug, Clone, Copy, IndexedEnum)]
pub enum FermiParams {
    Osc1RatioCoarse,
    Osc1RatioFine,
    Osc1Feedback,
    Osc1Feedforward,

    Osc1Attack,
    Osc1Decay,
    Osc1Sustain,
    Osc1Release,

    Osc2RatioCoarse,
    Osc2RatioFine,
    Osc2Feedback,
    Osc1Level,

    Osc2Attack,
    Osc2Decay,
    Osc2Sustain,
    Osc2Release,

    Osc1Waveform,
    Osc2Waveform,
    MasterLevel
}

define_params_bag!(FermiParamsBag, FermiParams, [
    0.0, 0.0, 0.1, 0.1,
    0.1, 0.2, 0.5, 0.4,
    0.0, 0.0, 0.1, 0.5,
    0.1, 0.2, 0.5, 0.4,
    0.5, 0.2, 0.5,
]);

#[derive(Default)]
pub struct FermiVoice {
    env1: ADSREnvelope,
    env2: ADSREnvelope,
    osc1: Oscillator<SinSq>,
    osc2: Oscillator<SinSq>,
    current_note: Option<u8>,
    osc1_feedback: f32,
    osc2_feedback: f32,
    osc1_feedforward: f32,
    osc1_output: f32,
    osc2_output: f32,
    velocity: f32,
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

    fn setup_feeds(&mut self, params: &Bag) {
        self.osc1_feedback = helpers::log_control(params.get(FermiParams::Osc1Feedback));
        self.osc2_feedback = helpers::log_control(params.get(FermiParams::Osc2Feedback));
        self.osc1_feedforward = helpers::log_control(params.get(FermiParams::Osc1Feedforward))
    }

    fn setup_waves(&mut self, params: &Bag) {
        self.osc1.get_wave_mut().square_mix = params.get(FermiParams::Osc1Waveform);
        self.osc2.get_wave_mut().square_mix = params.get(FermiParams::Osc2Waveform);
    }
}

impl Voice for FermiVoice {
    type ParamsEnum = FermiParams;
    type Bag = Bag;
    type Depth = f32;
    type PostParam = f32;

    fn prepare_post(params: &Bag) -> f32 {
        helpers::log_control(params.get(FermiParams::MasterLevel))
    }

    fn process_post(master: &f32, frame: Frame) -> Frame {
        frame * *master
    }

    fn init (&mut self, params: &Bag, rate: f32) {
        self.setup_envelopes(params, rate);
    }

    fn current_note (&self) -> Option<u8> { self.current_note }

    fn is_finished (&self) -> bool {
        self.env1.is_finished() &&
        self.env2.is_finished()
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        self.current_note = Some(note);
        self.env1.trigger();
        self.env2.trigger();

        self.osc1.phase_reset();
        self.osc2.phase_reset();

        self.osc1_output = 0.0;
        self.osc2_output = 0.0;
        self.velocity = velocity as f32 / 127.0;
    }

    fn note_off(&mut self, _note: u8, _velocity: u8) {
        self.current_note = None;
        self.env1.release();
        self.env2.release();
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
            FermiParams::Osc1Feedback => self.setup_feeds(bag),
            FermiParams::Osc2Feedback => self.setup_feeds(bag),
            FermiParams::Osc1Feedforward => self.setup_feeds(bag),
            FermiParams::Osc1Waveform => self.setup_waves(bag),
            FermiParams::Osc2Waveform => self.setup_waves(bag),
            _ => (),
        };
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

                self.osc1.set_freq(freq1);
                self.osc2.set_freq(freq2);
                true
            }
            _ => !self.is_finished()
        }
    }

    #[inline]
    fn process_sample(&mut self, timestep: f32) -> Frame {
        const MAGIC: f32 = 13.25;

        let env1 = self.env1.get_value();
        let env2 = self.env2.get_value();

        let feedback1 = self.osc1_output * self.osc1_feedback;
        let feedback2 = self.osc2_output * self.osc2_feedback;

        self.osc1_output = self.osc1.get_offset_value(feedback1) * env1 * MAGIC;
        let feedforward = self.osc1_output * self.osc1_feedforward;
        self.osc2_output = self.osc2.get_offset_value(feedback2 * MAGIC + feedforward) * env2;

        self.env1.process();
        self.env2.process();
        self.osc1.step(timestep);
        self.osc2.step(timestep);

        Frame {
            l: self.osc2_output,
            r: self.osc2_output
        }
    }
}
