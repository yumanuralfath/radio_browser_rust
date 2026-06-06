use std::fmt::Debug;

pub fn debug(enabled: bool, label: &str) {
    if enabled {
        println!("{label}");
    }
}

pub fn debug_value<T: Debug>(enabled: bool, label: &str, value: &T) {
    if enabled {
        println!("{label}: {value:#?}");
    }
}
