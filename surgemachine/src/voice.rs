use frame::Frame;
use params_bag::ParamsBag;

pub trait Voice {
    type ParamsEnum: ::IndexedEnum;
    type Bag: ParamsBag<Self::ParamsEnum>;
    type PostParam;
    type Depth;

    fn init(&mut self, bag: &Self::Bag, sample_rate: f32) {
        bag.for_each(&mut |param, _| {
            self.update_param(bag, param, sample_rate);
        });
    }

    fn current_note (&self) -> Option<u8>;
    fn note_on(&mut self, note: u8, _velocity: u8);
    fn note_off(&mut self, note: u8, _velocity: u8);
    fn init_process(&mut self, &Self::Bag) -> bool { true }
    fn process_sample(&mut self, timestep: f32) -> Frame<Self::Depth>;
    fn is_finished (&self) -> bool;
    fn prepare_post (&Self::Bag) -> Self::PostParam;
    fn process_post (&Self::PostParam, f: Frame<Self::Depth>) -> Frame<Self::Depth> { f }
    fn update_param (&mut self, &Self::Bag, Self::ParamsEnum, f32) {}
}
