use vst2::buffer::AudioBuffer;
use vst2::plugin::{Category, Plugin, Info, CanDo};
use vst2::event::Event;
use vst2::api::Supported;
use std::marker::PhantomData;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use surgemachine::create_device;
pub use surgemachine::DeviceType;
use surgemachine::device::DevicePlugin;

pub struct SynthPlugin<Data: SynthPluginData> {
    sample_rate: f32,
    device: Option<Box<DevicePlugin>>,
    phantom: PhantomData<Data>
}

pub trait SynthPluginData {
    fn get_name () -> String { format!("{:?}", Self::get_device_type()) }
    fn get_uid () -> i32 {
        let mut s = DefaultHasher::new();
        Self::get_name().hash(&mut s);
        s.finish() as i32
    }
    fn get_device_type () -> DeviceType;
}

impl<Data: SynthPluginData> SynthPlugin<Data> {

    /// http://www.midimountain.com/midi/midi_status.htm
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
        let mut device = create_device(Data::get_device_type());
        device.set_sample_rate(self.sample_rate);
        self.device = Some(device)
    }
}

impl<D: SynthPluginData> Default for SynthPlugin<D> {
    fn default() -> Self {
        let mut plugin = Self {
            sample_rate: 44100.0,
            device: None,
            phantom: Default::default()
        };
        plugin.init_device();
        plugin
    }
}

impl<Data: SynthPluginData> Plugin for SynthPlugin<Data> {
    fn get_info(&self) -> Info {
        Info {
            name: Data::get_name(),
            vendor: "Frizi".to_string(),
            unique_id: Data::get_uid(),
            category: Category::Synth,
            inputs: 0,
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
        self.device.as_ref()
            .map_or(format!("{}", param), |dev| dev.get_parameter_name(param))
    }

    fn get_parameter_label(&self, param: i32) -> String {
        self.device.as_ref()
            .map_or("".to_string(), |dev| dev.get_parameter_label(param))
    }

    fn get_parameter_text(&self, param: i32) -> String {
        let value = self.get_parameter(param);
        self.device.as_ref()
            .map_or(format!("{:.3}", value), |dev| dev.get_parameter_text(param))
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
        self.device.as_mut()
            .map(|dev| dev.set_sample_rate(rate));
    }

    fn process(&mut self, buffer: AudioBuffer<f32>) {
        let (_, mut outputs) = buffer.split();

        if let Some(dev) = self.device.as_mut() {
            if outputs.len() < 2 { panic!("Outputs should have at least length 2") }
            let right = outputs.remove(1);
            let left = outputs.remove(0);
            dev.run(None, Some([right, left]))
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe
        }
    }

}
