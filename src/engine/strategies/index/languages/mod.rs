pub mod c;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

pub use c::get_object_index as c_get_object_index;
pub use go::get_object_index as go_get_object_index;
pub use java::get_object_index as java_get_object_index;
pub use javascript::get_object_index as js_get_object_index;
pub use python::get_object_index as python_get_object_index;
pub use rust::get_object_index as rust_get_object_index;
