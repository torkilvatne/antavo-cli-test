use crate::escher::SigningParams;
use crate::escher::hmac_signer::HmacSigner;
use crate::escher::{
    canonicalizer::Canonicalizer, context::SigningContext, error::EscherError,
    headers::SignedHeaders, request::EscherRequest,
};

pub fn sign_request(
    request: EscherRequest,
    params: &SigningParams,
) -> Result<SignedHeaders, EscherError> {
    let context = SigningContext::new(&request, params, jiff::Timestamp::now())?;
    let canonical = Canonicalizer::canonicalize(request, &context.canonical_headers());
    let signer = HmacSigner::new(params);
    let signature = signer.compute_signature(&context, &canonical)?;
    Ok(SignedHeaders::from_parts(params, &context, &canonical, signature))
}
