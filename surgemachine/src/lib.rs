#![feature(conservative_impl_trait)]
// #![feature(alloc_system)]
// extern crate alloc_system;

#[macro_use] extern crate surgemachine_macros;
#[macro_use] mod params_macros;

extern crate rand;

pub mod device;
pub mod waveform;

mod envelope;
mod helpers;
mod oscillator;
mod pendulum;

use device::*;

pub use pendulum::PendulumParams;

pub trait IndexedEnum {
    const NUM_ITEMS: u64;
    fn to_index(&self) -> u64;
    fn from_index(index: u64) -> Self;
    fn from_param(val: f32) -> Self where Self: Sized {
        let num = Self::NUM_ITEMS;
        let index = (val * (num as f32)).floor() as u64;
        let bound_index = std::cmp::max(0, std::cmp::min(num - 1, index));
        Self::from_index(bound_index)
    }
}

pub enum DeviceType {
    Pendulum,
    // Fermi,
}

pub fn create_device (device_type: DeviceType) -> Box<Device> {
    match device_type {
        DeviceType::Pendulum => Box::new(pendulum::Pendulum::default()),
        // DeviceType::Fermi => Box::new(fermi::Fermi::default()),
    }
}
