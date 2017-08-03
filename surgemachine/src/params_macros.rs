macro_rules! define_params_bag {
    ($struct:ident, $params_type:ident, $default:expr) => {
        struct $struct {
            params: [f32; $params_type::NUM_ITEMS as usize]
        }

        impl $struct {
            fn get (&self, param: $params_type) -> f32 {
                self.params[param.to_index() as usize]
            }

            fn set (&mut self, param: $params_type, value: f32) {
                self.params[param.to_index() as usize] = value
            }
        }

        impl Default for $struct {
            fn default () -> $struct {
                $struct {
                    params: $default
                }
            }
        }
    }
}
