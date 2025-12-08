pub mod c;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

pub use c::get_selector as c_get_selector;
pub use go::get_selector as go_get_selector;
pub use java::get_selector as java_get_selector;
pub use javascript::get_selector as js_get_selector;
pub use python::get_selector as python_get_selector;
pub use rust::get_selector as rust_get_selector;
