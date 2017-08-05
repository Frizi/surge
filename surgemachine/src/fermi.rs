use device::*;
use helpers;
use IndexedEnum;

#[derive(Debug, IndexedEnum)]
pub enum ParamIndices {
    Osc1Waveform = 0,
    Osc1RatioCoarse,
    Osc1RatioFine,
    Osc1Feedback,
    Osc1FeedForward,

    Osc1Attack,
    Osc1Decay,
    Osc1Sustain,
    Osc1Release,

    Osc2Waveform,
    Osc2RatioCoarse,
    Osc2RatioFine,
    Osc2Feedbsck,

    Osc2Attack,
    Osc2Decay,
    Osc2Sustain,
    Osc2Release,

    MasterLevel,

    VoicesUnisono,
    VoicesDetune,
    VoicesPan,

    VibratoFreq,
    VibratoAmount,
    Rise,
}

pub struct Fermi {
    time: f32,
    note_duration: f32,
    note: Option<u8>,
    params: [f32; ParamIndices::NUM_ITEMS as usize]
}

impl Default for Fermi {
    fn default() -> Fermi {
        Fermi {
            note_duration: 0.0,
            time: 0.0,
            note: None,
            params: [
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.5,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
           ]
        }
    }
}

impl Fermi {
    fn param (&self, index: ParamIndices) -> f32 {
        self.params[index as usize]
    }
}

impl Device for Fermi {
    fn get_num_parameters (&self) -> i32 { ParamIndices::LAST_PARAM as i32 }
    fn get_parameter (&self, index: i32) -> f32 { self.params[index as usize] }
    fn set_parameter (&mut self, index: i32, value: f32) { self.params[index as usize] = value }

    fn run<'a> (&mut self, sample_rate: f64, mut outputs : AudioBus<'a, f32>) {
        let samples = outputs
            .first()
            .unwrap()
            .len();

        let total_time : f64 = (samples as f64) / sample_rate;

        if let Some(current_note) = self.note {
            let freq = helpers::midi_note_to_hz(current_note);
            let attack = self.param(ParamIndices::Osc1Attack) as f64;

            for output_buffer in outputs.iter_mut() {
                let mut i : u32 = 0;

                for output_sample in output_buffer.iter_mut() {
                     let extra_time = total_time * (i as f64) / samples as f64;
                     let t : f64 = self.time + extra_time;
                     let nt = self.note_duration + extra_time;

                     let signal = (t * freq * TAU).sin();
                     // Apply a quick envelope to the attack of the signal to avoid popping.

                     let alpha = if nt < attack {
                         nt / attack
                     } else {
                         1.0
                     };

                     *output_sample = (signal * alpha) as f32;
                     i += 1;
                 }
            }

            self.note_duration += total_time;
            self.time += total_time;
        }
        else {
            for output_buffer in outputs.iter_mut() {
                for output_sample in output_buffer.iter_mut() {
                     *output_sample = 0.0;
                 }
            }
        }
    }

    fn note_on(&mut self, note: u8, _velocity: u8) {
        self.note_duration = 0.0;
        self.note = Some(note)
    }

    fn note_off(&mut self, note: u8, _velocity: u8) {
        if self.note == Some(note) {
            self.note = None
        }
    }
}

const PI : f64 = 3.14159265359;
pub const TAU : f64 = PI * 2.0;

// for (input_buffer, output_buffer) in inputs.iter().zip(outputs) {
//     let mut t = self.time;
//
//     // let osc1Feedback = fix_denormal(self.osc1Feedback * self.osc1Feedback / 2.0);
//     // let osc2Feedback = fix_denormal(self.osc1Feedback * self.osc1Feedback / 2.0);
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
