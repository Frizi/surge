use frame::Frame;
use params_bag::ParamsBag;

pub trait Voice<E: ::IndexedEnum, T:ParamsBag<E>, D=f32> {
    fn init(&mut self, bag: &T, sample_rate : f32) {
        bag.for_each(&mut |param, _| {
            self.update_param(bag, param, sample_rate);
        });
    }

    fn current_note (&self) -> Option<u8>;
    fn note_on(&mut self, note: u8, _velocity: u8);
    fn note_off(&mut self, note: u8, _velocity: u8);
    fn init_process(&mut self, &T) -> bool { true }
    fn process_sample(&mut self, timestep: f32) -> Frame<D>;
    fn is_finished (&self) -> bool;
    fn process_post (&T, f: Frame<D>) -> Frame<D> { f }
    fn update_param (&mut self, &T, E, f32) {}
}
