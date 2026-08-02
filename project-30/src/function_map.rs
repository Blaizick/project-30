pub mod function_map {
    use std::{any::{self, Any}, collections::{HashMap, VecDeque}, os::raw::c_void};
    
    pub struct FunctionMap {
        function_map: HashMap<String, *const c_void>,
    }

    impl FunctionMap {
        pub fn new() -> Self  {
            Self { 
                function_map: HashMap::new(), 
            }
        }
        pub fn register_function(&mut self, name: impl Into<String>, function_ptr: *const c_void, ) {
            self.function_map.insert(name.into(), function_ptr);
        }
        pub fn get_function(&mut self, name: &str, ) -> Option<*const c_void> {
            self.function_map.get(name).copied()
        }
    }
}
#[macro_export]
macro_rules! register_function {
    ($map:expr, $func:ident) => {
        $map.register_function(
            stringify!($func),
            $func as *const std::ffi::c_void,
        );
    };
}