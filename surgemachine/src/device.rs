
pub type AudioBus<'a, T> = [&'a mut [T]; 2];

pub trait Device {
    fn run<'a> (&mut self, inputs: Option<AudioBus<'a, f32>>, outputs: Option<AudioBus<'a, f32>>);

    fn note_on (&mut self, note: u8, velocity: u8);
    fn note_off (&mut self, note: u8, velocity: u8);

    fn set_sample_rate(&mut self, sample_rate: f32);
    fn get_parameter(&self, index: i32) -> f32;
    fn set_parameter(&mut self, index: i32, val: f32);
    fn get_num_parameters(&self) -> i32;
}

// plugin specific
pub trait DevicePlugin: Device {
    fn get_parameter_name(&self, index: i32) -> String { format!("{}", index) }
    fn get_parameter_label(&self, _index: i32) -> String { "".to_string() }
    fn get_parameter_text(&self, index: i32) -> String {
        let value = self.get_parameter(index);
        format!("{:.3}", value)
    }
}
