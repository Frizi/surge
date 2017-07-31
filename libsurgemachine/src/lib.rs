#![feature(conservative_impl_trait)]

pub mod device;
pub mod pendulum;
// pub mod fermi;
mod envelope;
mod helpers;
mod oscillator;
mod waveform;

use device::*;

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
