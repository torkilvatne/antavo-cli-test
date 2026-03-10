use crate::escher::request::{EncodingState, EscherRequest, RequestMetadata};

pub enum EscherBuilderError {
    MissingField(String),
}

impl std::fmt::Debug for EscherBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EscherBuilderError::MissingField(field) => {
                f.debug_tuple("MissingField").field(field).finish()
            }
        }
    }
}

impl std::fmt::Display for EscherBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EscherBuilderError::MissingField(field) => {
                write!(f, "Missing field: {}", field)
            }
        }
    }
}

impl std::error::Error for EscherBuilderError {}

impl EscherBuilderError {
    fn missing_field(field: impl Into<String>) -> EscherBuilderError {
        EscherBuilderError::MissingField(field.into())
    }
}

pub struct EscherRequestBuilder<'a> {
    body: Option<&'a [u8]>,
    method: Option<&'a str>,
    path: Option<&'a str>,
    host: Option<&'a str>,
    query: Option<&'a str>,
    headers: Vec<(&'a str, &'a str)>,
    path_encoding_state: EncodingState,
    query_encoding_state: EncodingState,
}

impl<'a> Default for EscherRequestBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> EscherRequestBuilder<'a> {
    pub fn new() -> Self {
        Self {
            body: None,
            method: None,
            path: None,
            host: None,
            query: None,
            headers: vec![],
            path_encoding_state: EncodingState::Encoded,
            query_encoding_state: EncodingState::Encoded,
        }
    }

    pub fn with_method(mut self, method: &'a str) -> Self {
        self.method = Some(method);
        self
    }

    pub fn with_path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_host(mut self, host: &'a str) -> Self {
        self.host = Some(host);
        self
    }

    pub fn with_body(mut self, body: &'a [u8]) -> Self {
        self.body = Some(body);
        self
    }

    #[allow(dead_code)]
    pub fn with_headers(mut self, headers: Vec<(&'a str, &'a str)>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }

    pub fn add_header(mut self, name: &'a str, value: &'a str) -> Self {
        self.headers.push((name, value));
        self
    }

    #[allow(dead_code)]
    pub fn encode_path(mut self) -> Self {
        self.path_encoding_state = EncodingState::Raw;
        self
    }

    #[allow(dead_code)]
    pub fn encode_query(mut self) -> Self {
        self.query_encoding_state = EncodingState::Raw;
        self
    }

    pub fn build(self) -> Result<EscherRequest<'a>, EscherBuilderError> {
        Ok(EscherRequest::new(
            self.method
                .ok_or(EscherBuilderError::missing_field("method"))?,
            self.path.ok_or(EscherBuilderError::missing_field("path"))?,
            self.host.ok_or(EscherBuilderError::missing_field("host"))?,
            self.body.unwrap_or(&[]),
            self.query.unwrap_or(""),
            self.headers,
            RequestMetadata {
                path_encoding: self.path_encoding_state,
                query_encoding: self.query_encoding_state,
            },
        ))
    }
}
