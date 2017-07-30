use device::*;
use helpers;

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

            5 => ParamIndices::Osc1Attack,
            6 => ParamIndices::Osc1Decay,
            7 => ParamIndices::Osc1Sustain,
            8 => ParamIndices::Osc1Release,

            9 => ParamIndices::Osc2Waveform,
            10 => ParamIndices::Osc2RatioCoarse,
            11 => ParamIndices::Osc2RatioFine,

            13 => ParamIndices::Osc2Attack,
            14 => ParamIndices::Osc2Decay,
            15 => ParamIndices::Osc2Sustain,
            16 => ParamIndices::Osc2Release,

            17 => ParamIndices::MasterLevel,
            _ => panic!("Invalid param index {}", num)
        }
    }
}

pub struct Pendulum {
    time: f64,
    note_duration: f64,
    note: Option<u8>,
    params: [f32; ParamIndices::LAST_PARAM as usize]
}

impl Default for Pendulum {
    fn default() -> Pendulum {
        Pendulum {
            note_duration: 0.0,
            time: 0.0,
            note: None,
            params: [
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
           ]
        }
    }
}

impl Pendulum {
    fn param (&self, index: ParamIndices) -> f32 {
        self.params[index as usize]
    }
}


impl Device for Pendulum {
    fn get_num_parameters (&self) -> i32 { ParamIndices::LAST_PARAM as i32 }
    fn get_parameter (&self, index: i32) -> f32 { self.params[index as usize] }
    fn set_parameter (&mut self, index: i32, value: f32) { self.params[index as usize] = value }

    fn run<'a> (&mut self, sample_rate: f64, outputs : AudioBus<'a, f32>) {
        let samples = outputs
            .first()
            .unwrap()
            .len();

        let total_time : f64 = (samples as f64) / sample_rate;

        if let Some(current_note) = self.note {
            let freq = helpers::midi_note_to_hz(current_note);
            let attack = self.param(ParamIndices::Osc1Attack) as f64;
            let mut i : u64 = 0;

            helpers::each_frame(outputs, &mut|left_sample, right_sample| {
                let extra_time = total_time * (i as f64) / samples as f64;
                let t : f64 = self.time + extra_time;
                let nt = self.note_duration + extra_time;

                let signal = (t * freq * helpers::TAU).sin();
                // Apply a quick envelope to the attack of the signal to avoid popping.

                let alpha = if nt < attack {
                 nt / attack
                } else {
                 1.0
                };

                *left_sample =(signal * alpha) as f32;
                *right_sample = (signal * alpha) as f32;
                i += 1;
            });

            self.note_duration += total_time;
            self.time += total_time;
        }
        else {
            helpers::each_frame(outputs, &mut|left_sample, right_sample| {
                *left_sample = 0.0;
                *right_sample = 0.0;
            });
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
