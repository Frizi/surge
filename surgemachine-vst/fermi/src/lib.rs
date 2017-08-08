#[macro_use] extern crate vst2;
extern crate surgemachine_plugin_base as base;

use base::{SynthPlugin, SynthPluginData, DeviceType};

type FermiPlugin = SynthPlugin<FermiPluginData>;

struct FermiPluginData;
impl SynthPluginData for FermiPluginData {
    fn get_device_type () -> DeviceType { DeviceType::Fermi }
}

plugin_main!(FermiPlugin);
