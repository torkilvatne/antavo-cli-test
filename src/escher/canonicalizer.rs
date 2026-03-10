use crate::escher::{
    request::{EncodingState, EscherRequest},
    util,
};

pub struct CanonicalRequest {
    pub canonical_string: String,
    pub signed_headers_list: String,
}

pub struct Canonicalizer;

impl Canonicalizer {
    pub fn canonicalize(
        request: EscherRequest,
        additional_headers: &[(&str, &str)],
    ) -> CanonicalRequest {
        let complete_headers = Self::build_complete_headers(&request, additional_headers);

        let method = request.method().to_uppercase();
        let path = Self::canonicalize_path(request.path(), request.path_encoding_state());
        let query = Self::canonicalize_query(request.query(), request.query_encoding_state());
        let headers_str = Self::canonicalize_headers(&complete_headers);
        let signed_headers_list = Self::canonicalize_signed_headers(&complete_headers);
        let checksum = util::generate_checksum(request.body());

        let canonical_string = format!(
            "{}\n{}\n{}\n{}\n\n{}\n{}",
            method, path, query, headers_str, signed_headers_list, checksum
        );

        CanonicalRequest {
            canonical_string,
            signed_headers_list,
        }
    }

    fn build_complete_headers(
        request: &EscherRequest,
        additional_headers: &[(&str, &str)],
    ) -> Vec<(String, String)> {
        let mut headers: Vec<(String, String)> = request
            .headers()
            .iter()
            .map(|(k, v)| (k.to_lowercase(), v.to_string()))
            .collect();

        headers.extend(
            additional_headers
                .iter()
                .map(|(name, value)| (name.to_lowercase(), value.to_string())),
        );

        headers.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        headers
    }

    fn canonicalize_path(path: &str, enc_state: &EncodingState) -> String {
        if path.is_empty() {
            return "/".to_string();
        }

        match enc_state {
            EncodingState::Encoded => path.into(),
            EncodingState::Raw => {
                let mut result = String::new();
                for (i, segment) in path.split('/').enumerate() {
                    if i > 0 {
                        result.push('/');
                    }
                    result.push_str(&util::url_encode(segment));
                }
                result
            }
        }
    }

    fn canonicalize_query(query: &str, enc_state: &EncodingState) -> String {
        if query.is_empty() || query == "?" {
            return String::new();
        }

        let mut encoded_query: Vec<String> = match enc_state {
            EncodingState::Encoded => query.split('&').map(|q| q.to_string()).collect(),
            EncodingState::Raw => query.split('&').map(util::url_encode).collect(),
        };
        encoded_query.sort_unstable();
        encoded_query.join("&")
    }

    fn canonicalize_headers(headers: &[(String, String)]) -> String {
        let capacity = headers
            .iter()
            .map(|(name, value)| name.len() + value.len() + 2)
            .sum();

        let mut result = String::with_capacity(capacity);

        for (i, (key, value)) in headers.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(key);
            result.push(':');
            result.push_str(value);
        }
        result
    }

    fn canonicalize_signed_headers(headers: &[(String, String)]) -> String {
        let capacity = headers.iter().map(|(name, _)| name.len() + 1).sum();
        let mut result = String::with_capacity(capacity);
        for (i, (key, _)) in headers.iter().enumerate() {
            if i > 0 {
                result.push(';');
            }
            result.push_str(key);
        }
        result
    }
}
