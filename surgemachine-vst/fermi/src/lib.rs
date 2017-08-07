#[macro_use] extern crate vst2;
extern crate surgemachine;

use vst2::buffer::AudioBuffer;
use vst2::plugin::{Category, Plugin, Info, CanDo};
use vst2::event::Event;
use vst2::api::Supported;

use surgemachine::{create_device, DeviceType, IndexedEnum, FermiParams};
use surgemachine::device::{Device};
use surgemachine::helpers;

struct FermiPlugin {
    sample_rate: f32,
    device: Option<Box<Device>>,
}

impl FermiPlugin {
    /// Process an incoming midi event.
    ///
    /// The midi data is split up like so:
    ///
    /// `data[0]`: Contains the status and the channel. Source: [source]
    /// `data[1]`: Contains the supplemental data for the message - so, if this was a NoteOn then
    ///            this would contain the note.
    /// `data[2]`: Further supplemental data. Would be velocity in the case of a NoteOn message.
    ///
    /// [source]: http://www.midimountain.com/midi/midi_status.htm
    fn process_midi_event(&mut self, data: [u8; 3]) {
        self.device.as_mut()
            .map(|d| {
                match data[0] {
                    128 => d.note_off(data[1], data[2]),
                    144 => d.note_on(data[1], data[2]),
                    _ => ()
                };
            });
    }

    fn init_device (&mut self) {
        let mut device = create_device(DeviceType::Fermi);
        device.set_sample_rate(self.sample_rate);
        self.device = Some(device)
    }
}

impl Default for FermiPlugin {
    fn default() -> FermiPlugin {
        let mut plugin = FermiPlugin {
            sample_rate: 44100.0,
            device: None
        };
        plugin.init_device();
        plugin
    }
}

impl Plugin for FermiPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "Fermi".to_string(),
            vendor: "Frizi".to_string(),
            unique_id: 18563110,
            category: Category::Synth,
            inputs: 2,
            outputs: 2,
            parameters: self.device.as_ref().map_or(0, |d| d.get_num_parameters()),
            initial_delay: 0,
            ..Info::default()
        }
    }

    fn process_events(&mut self, events: Vec<Event>) {
        for event in events {
            match event {
                Event::Midi { data, ..  } => self.process_midi_event(data),
                // More events can be handled here.
                _ => {}
            }
        }
    }

    fn set_parameter(&mut self, param: i32, value: f32) {
        self.device.as_mut()
            .map(|dev| dev.set_parameter(param, value));
    }

    fn get_parameter(&self, param: i32) -> f32 {
        self.device.as_ref()
            .map_or(0.0, |dev| dev.get_parameter(param))
    }

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

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }

    fn process(&mut self, buffer: AudioBuffer<f32>) {
        let (_, mut outputs) = buffer.split();

        if outputs.len() < 2 {
            panic!("Outputs should have at least length 2")
        }

        let right = outputs.remove(1);
        let left = outputs.remove(0);

        self.device.as_mut()
            .map(|dev| dev.run([ left, right ]));
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe
        }
    }

}

plugin_main!(FermiPlugin);
