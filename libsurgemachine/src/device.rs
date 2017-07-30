
pub type AudioBus<'a> = [&'a mut [f32]; 2];

pub trait Device {
    fn run<'a> (&mut self, sample_rate: f64, outputs: AudioBus<'a>);
    fn note_on (&mut self, note: u8, velocity: u8);
    fn note_off (&mut self, note: u8, velocity: u8);

    fn get_parameter(&self, index: i32) -> f32;
    fn set_parameter(&mut self, index: i32, val: f32);
    fn get_num_parameters(&self) -> i32;
}
