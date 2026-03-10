use crate::escher::{
    canonicalizer::CanonicalRequest, context::SigningContext, params::SigningParams,
};

#[derive(Debug, Clone)]
pub struct SignedHeader {
    pub name: String,
    pub value: String,
}

impl From<(&str, &str)> for SignedHeader {
    fn from((name, value): (&str, &str)) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignedHeaders {
    pub auth: SignedHeader,
    pub date: SignedHeader,
    pub host: SignedHeader,
}

impl SignedHeaders {
    pub(crate) fn from_parts(
        params: &SigningParams,
        context: &SigningContext,
        canonical: &CanonicalRequest,
        signature: String,
    ) -> Self {
        let credential_scope_dated =
            format!("{}/{}", context.short_date(), params.credential_scope());

        let auth_value = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            params.hashing_algorithm(),
            params.api_key(),
            credential_scope_dated,
            canonical.signed_headers_list,
            signature
        );

        let [date, host] = context.canonical_headers();

        Self {
            auth: (params.auth_header_name(), auth_value.as_str()).into(),
            date: date.into(),
            host: host.into(),
        }
    }
}

impl IntoIterator for SignedHeaders {
    type Item = (String, String);
    type IntoIter = std::array::IntoIter<Self::Item, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (self.auth.name, self.auth.value),
            (self.date.name, self.date.value),
            (self.host.name, self.host.value),
        ]
        .into_iter()
    }
}
