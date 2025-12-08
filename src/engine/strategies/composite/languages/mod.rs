pub mod c;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

pub use c::{resolve_array as c_resolve_array, resolve_initializer as c_resolve_initializer};
pub use go::{resolve_array as go_resolve_array, resolve_struct as go_resolve_struct};
pub use java::{resolve_array as java_resolve_array, resolve_object as java_resolve_object};
pub use javascript::{resolve_array as js_resolve_array, resolve_object as js_resolve_object};
pub use python::{resolve_array as python_resolve_array, resolve_dict as python_resolve_dict};
pub use rust::{resolve_array as rust_resolve_array, resolve_struct as rust_resolve_struct};
