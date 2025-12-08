pub mod c;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

pub use c::extract_return as c_extract_return;
pub use go::extract_return as go_extract_return;
pub use java::extract_return as java_extract_return;
pub use javascript::extract_return as js_extract_return;
pub use python::extract_return as python_extract_return;
pub use rust::extract_return as rust_extract_return;
