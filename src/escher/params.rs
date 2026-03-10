#[derive(Debug, Clone)]
pub struct SigningParams {
    api_key: String,
    credential_scope: String,
    auth_header_name: String,
    date_header_name: String,
    signing_key_seed: String,
    hashing_algorithm: String,
}

impl SigningParams {
    pub fn new(
        api_key: String,
        api_secret: String,
        credential_scope: String,
        algo_prefix: String,
        hash_algo: String,
        auth_header_name: String,
        date_header_name: String,
    ) -> Self {
        let signing_key_seed = format!("{}{}", algo_prefix, api_secret);
        let hashing_algorithm = format!("{}-HMAC-{}", algo_prefix, hash_algo);

        Self {
            api_key,
            credential_scope,
            auth_header_name,
            date_header_name,
            signing_key_seed,
            hashing_algorithm,
        }
    }
}

impl SigningParams {
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
    pub fn credential_scope(&self) -> &str {
        &self.credential_scope
    }
    pub fn signing_key_seed(&self) -> &str {
        &self.signing_key_seed
    }
    pub fn hashing_algorithm(&self) -> &str {
        &self.hashing_algorithm
    }
    pub fn auth_header_name(&self) -> &str {
        &self.auth_header_name
    }
    pub fn date_header_name(&self) -> &str {
        &self.date_header_name
    }
}
