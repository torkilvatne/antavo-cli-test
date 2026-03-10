use hmac::digest::InvalidLength;

use crate::escher::builder;

pub enum EscherError {
    ConflictingHeader(String),
    HashDigestError(InvalidLength),
    #[allow(dead_code)]
    ConfigurationError(String),
    BuilderError(builder::EscherBuilderError),
}

impl std::fmt::Debug for EscherError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EscherError::ConflictingHeader(header) => {
                f.debug_tuple("ConflictingHeader").field(header).finish()
            }
            EscherError::HashDigestError(source) => {
                f.debug_tuple("HashDigestError").field(source).finish()
            }
            EscherError::ConfigurationError(msg) => {
                f.debug_tuple("ConfigurationError").field(msg).finish()
            }
            EscherError::BuilderError(source) => {
                f.debug_tuple("BuilderError").field(source).finish()
            }
        }
    }
}

impl std::fmt::Display for EscherError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EscherError::ConflictingHeader(header) => {
                write!(
                    f,
                    "Header '{}' is reserved for Escher signing protocol.",
                    header
                )
            }
            EscherError::HashDigestError(source) => {
                write!(f, "Failed to generate hash: {}", source)
            }
            EscherError::ConfigurationError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
            EscherError::BuilderError(_) => {
                write!(f, "Failed to build request")
            }
        }
    }
}

impl std::error::Error for EscherError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EscherError::ConflictingHeader(_) => None,
            EscherError::HashDigestError(source) => Some(source),
            EscherError::ConfigurationError(_) => None,
            EscherError::BuilderError(source) => Some(source),
        }
    }
}

impl From<InvalidLength> for EscherError {
    fn from(source: InvalidLength) -> Self {
        EscherError::HashDigestError(source)
    }
}

impl From<builder::EscherBuilderError> for EscherError {
    fn from(source: builder::EscherBuilderError) -> Self {
        EscherError::BuilderError(source)
    }
}
