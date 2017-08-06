pub trait ParamsBag<T: ::IndexedEnum>
    where Self: Sized
{
    fn get (&self, param: T) -> f32;
    fn set (&mut self, param: T, value: f32);
    fn for_each (&self, &mut FnMut(T, f32) -> ());
}

macro_rules! define_params_bag {
    ($struct:ident, $params_type:ident, $default:expr) => {
        pub struct $struct {
            params: [f32; $params_type::NUM_ITEMS as usize]
        }

        impl $crate::params_bag::ParamsBag<$params_type> for $struct {
            fn get (&self, param: $params_type) -> f32 {
                self.params[param.to_index() as usize]
            }

            fn set (&mut self, param: $params_type, value: f32) {
                self.params[param.to_index() as usize] = value
            }

            fn for_each (&self, func: &mut FnMut($params_type, f32) -> ()) {
                for x in 0..$params_type::NUM_ITEMS {
                    let param = $params_type::from_index(x);
                    let value = self.get(param);
                    func(param, value);
                };
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
