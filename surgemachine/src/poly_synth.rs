use device::*;
use smallvec::*;
use voice::Voice;
use params_bag::ParamsBag;
use helpers;
use frame::Frame;
use IndexedEnum;

pub struct PolySynth<V: Voice> {
    sample_rate: f32,
    voices: [V; 8],
    params: V::Bag,
    voice_cycle: u8,
}

impl<V> Default for PolySynth<V>
    where
        V::Bag: Default,
        V: Voice + Default
{
    fn default() -> Self {
        let bag: V::Bag = Default::default();
        let mut voices: [V; 8] = Default::default();
        for voice in voices.iter_mut() {
            voice.init(&bag, 1.0);
        }
        Self {
            sample_rate: 1.0,
            voices: voices,
            params: bag,
            voice_cycle: 0,
        }
    }
}

impl<V: Voice> PolySynth<V> {
    fn init_process (&mut self) -> (&V::Bag, SmallVec<[&mut V; 8]>) {
        let params = &self.params;
        let mut active_voices: SmallVec<[&mut V; 8]> = Default::default();
        for voice in self.voices.iter_mut() {
            if voice.init_process(params) {
                active_voices.push(voice);
            }
        }
        (params, active_voices)
    }

    fn is_finished (&self) -> bool {
        self.voices.iter().fold(true, |acc, voice| {
            acc && voice.is_finished()
        })
    }
}

impl<V:Voice<Depth=f32>> Device for PolySynth<V>
    where V::ParamsEnum: Copy
{
    fn set_parameter (&mut self, index: i32, value: f32) {
        let param = V::ParamsEnum::from_index(index as u32);
        self.params.set(param, value);
        for voice in self.voices.iter_mut() {
            voice.update_param(&self.params, param, self.sample_rate)
        }
    }

    fn run (&mut self, _inputs: Option<AudioBus<f32>>, outputs: Option<AudioBus<f32>>) {
        let timestep = helpers::time_per_sample(self.sample_rate);

        if !self.is_finished() {
            let (params, mut active_voices) = self.init_process();
            let postproc_data = V::prepare_post(params);

            if let Some(mut outs) = outputs {
                for (left_sample, right_sample) in helpers::frame_iter(&mut outs) {
                    let signal = active_voices.iter_mut()
                        .map(|voice| voice.process_sample(timestep))
                        .sum::<Frame>();

                    let signal = V::process_post(&postproc_data, signal);

                    *left_sample = signal.l;
                    *right_sample = signal.r;
                }
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

    fn set_sample_rate (&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        let bag = &self.params;
        for voice in self.voices.iter_mut() {
            voice.init(bag, self.sample_rate);
        }
    }

    fn get_num_parameters (&self) -> i32
    {
        V::ParamsEnum::NUM_ITEMS as i32
    }

    fn get_parameter (&self, index: i32) -> f32 {
        let param = V::ParamsEnum::from_index(index as u32);
        self.params.get(param)
    }
}
