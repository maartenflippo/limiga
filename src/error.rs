use thiserror::Error;

#[derive(Debug, Error)]
pub enum LimigaError {
    #[error("error reading file")]
    Io(#[from] std::io::Error),

    #[error("failed to parse dimacs")]
    DimacsError(#[from] limiga_dimacs::DimacsParseError),
}
