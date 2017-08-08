#[macro_use] extern crate vst2;
extern crate surgemachine_plugin_base as base;

use base::{SynthPlugin, SynthPluginData, DeviceType};

type PendulumPlugin = SynthPlugin<PendulumPluginData>;

struct PendulumPluginData;
impl SynthPluginData for PendulumPluginData {
    fn get_device_type () -> DeviceType { DeviceType::Pendulum }
}

plugin_main!(PendulumPlugin);
