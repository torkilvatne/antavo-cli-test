#[derive(Debug)]
pub enum EncodingState {
    Raw,
    Encoded,
}

#[derive(Debug)]
pub struct RequestMetadata {
    pub path_encoding: EncodingState,
    pub query_encoding: EncodingState,
}

#[derive(Debug)]
pub struct EscherRequest<'a> {
    method: &'a str,
    path: &'a str,
    host: &'a str,
    body: &'a [u8],
    query: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    meta: RequestMetadata,
}

impl<'a> EscherRequest<'a> {
    pub fn new(
        method: &'a str,
        path: &'a str,
        host: &'a str,
        body: &'a [u8],
        query: &'a str,
        headers: Vec<(&'a str, &'a str)>,
        meta: RequestMetadata,
    ) -> Self {
        Self {
            method,
            path,
            host,
            body,
            query,
            headers,
            meta,
        }
    }

    pub fn path(&self) -> &'a str {
        self.path
    }

    pub fn method(&self) -> &'a str {
        self.method
    }

    pub fn host(&self) -> &'a str {
        self.host
    }

    pub fn body(&self) -> &[u8] {
        self.body
    }

    pub fn query(&self) -> &'a str {
        self.query
    }

    pub fn headers(&self) -> &[(&'a str, &'a str)] {
        &self.headers
    }

    pub fn path_encoding_state(&self) -> &EncodingState {
        &self.meta.path_encoding
    }

    pub fn query_encoding_state(&self) -> &EncodingState {
        &self.meta.query_encoding
    }
}
