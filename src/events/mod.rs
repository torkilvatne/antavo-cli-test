pub mod checkout;
pub mod checkout_accept;
pub mod opt_in;
pub mod checkout_reject;
pub mod checkout_update;
pub mod partial_refund;
pub mod point_add;
pub mod point_fix;
pub mod point_spend;
pub mod point_sub;
pub mod point_unspend;
pub mod refund;
pub mod release_points;
pub mod reserve_points;

use anyhow::Result;
use crate::antavo::{client::AntavoClient, state::CustomerState};
use serde_json::Value;

/// Send an event and show the before/after state diff.
/// Returns the after-state.
pub async fn send_and_show(
    client: &AntavoClient,
    label: &str,
    body: Value,
) -> Result<CustomerState> {
    let before = client.get_customer_state().await?;

    println!("\n[{}]", label);
    println!("  Request: {}", serde_json::to_string(&body)?);

    let response = client.post_event(body).await?;
    println!("  Response: {}", serde_json::to_string(&response)?);

    let after = client.get_customer_state().await?;

    println!("  State diff:");
    CustomerState::print_diff(&before, &after);

    Ok(after)
}
