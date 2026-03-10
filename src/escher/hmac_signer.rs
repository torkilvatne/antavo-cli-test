use hmac::{Hmac, Mac, digest::Output};
use sha2::Sha256;

use crate::escher::{
    SigningParams, canonicalizer::CanonicalRequest, context::SigningContext, error::EscherError,
    util,
};

pub struct HmacSigner<'a> {
    params: &'a SigningParams,
}

impl<'a> HmacSigner<'a> {
    pub fn new(params: &'a SigningParams) -> Self {
        Self { params }
    }

    pub fn compute_signature(
        &self,
        context: &SigningContext,
        canonical: &CanonicalRequest,
    ) -> Result<String, EscherError> {
        let credential_scope_dated = format!(
            "{}/{}",
            context.short_date(),
            self.params.credential_scope()
        );

        let checksum = util::generate_checksum(canonical.canonical_string.as_bytes());
        let signing_string = format!(
            "{}\n{}\n{}\n{}",
            self.params.hashing_algorithm(),
            context.long_date(),
            credential_scope_dated,
            checksum
        );

        let signing_key = self.create_signing_key(context.short_date())?;
        let signature_bytes = self.hmac_digest(&signing_key, signing_string.as_bytes())?;
        Ok(hex::encode(signature_bytes))
    }

    fn create_signing_key(&self, short_date: &str) -> Result<Output<Hmac<Sha256>>, EscherError> {
        let key = self.hmac_digest(
            self.params.signing_key_seed().as_bytes(),
            short_date.as_bytes(),
        )?;

        self.params
            .credential_scope()
            .split('/')
            .try_fold(key, |k, v| self.hmac_digest(&k, v.as_bytes()))
    }

    fn hmac_digest(&self, key: &[u8], message: &[u8]) -> Result<Output<Hmac<Sha256>>, EscherError> {
        Ok(Hmac::<Sha256>::new_from_slice(key)
            .map_err(EscherError::HashDigestError)?
            .chain_update(message)
            .finalize()
            .into_bytes())
    }
}
