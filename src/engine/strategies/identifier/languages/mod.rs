pub mod c;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

pub use go::{
    find_declaration as go_find_declaration, find_file_level_const as go_find_file_level_const,
};
