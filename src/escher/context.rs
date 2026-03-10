use crate::escher::{error::EscherError, params::SigningParams, request::EscherRequest};

const HOST_HEADER_NAME: &str = "host";

#[derive(Debug)]
pub struct SigningContext {
    date_header: (String, String),
    host_header: (String, String),
    long_date: String,
    short_date: String,
}

impl SigningContext {
    pub fn new(
        request: &EscherRequest,
        params: &SigningParams,
        timestamp: jiff::Timestamp,
    ) -> Result<Self, EscherError> {
        Self::validate_no_conflicting_headers(request, params)?;
        let long_date = timestamp.strftime("%Y%m%dT%H%M%SZ").to_string();
        let short_date = timestamp.strftime("%Y%m%d").to_string();
        Ok(Self {
            short_date,
            long_date: long_date.clone(),
            date_header: (params.date_header_name().to_string(), long_date),
            host_header: (HOST_HEADER_NAME.to_string(), request.host().to_string()),
        })
    }

    pub fn canonical_headers(&self) -> [(&str, &str); 2] {
        [
            (self.date_header.0.as_str(), self.date_header.1.as_str()),
            (self.host_header.0.as_str(), self.host_header.1.as_str()),
        ]
    }

    pub fn long_date(&self) -> &str {
        &self.long_date
    }

    pub fn short_date(&self) -> &str {
        &self.short_date
    }

    fn validate_no_conflicting_headers(
        request: &EscherRequest,
        params: &SigningParams,
    ) -> Result<(), EscherError> {
        for (key, _) in request.headers() {
            if key.eq_ignore_ascii_case(HOST_HEADER_NAME)
                || key.eq_ignore_ascii_case(params.date_header_name())
            {
                return Err(EscherError::ConflictingHeader(key.to_string()));
            }
        }

        Ok(())
    }
}
