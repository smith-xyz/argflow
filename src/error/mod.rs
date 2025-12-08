mod classifier;
mod io;
mod parser;
mod query;

pub use classifier::ClassifierError;
pub use io::IoError;
pub use parser::ParserError;
pub use query::QueryError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Parser(#[from] ParserError),

    #[error(transparent)]
    Classifier(#[from] ClassifierError),

    #[error(transparent)]
    Query(#[from] QueryError),
}

pub type Result<T> = std::result::Result<T, Error>;
