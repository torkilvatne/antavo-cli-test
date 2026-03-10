/// Flow 02 — Earn Points: Checkout Pending → Reject
///
/// Goal: Verify that rejecting a pending checkout voids the pending points.
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::checkout::{CheckoutParams, checkout};
use crate::events::checkout_reject::checkout_reject;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    println!("\n=== Flow 02: Earn Points — Pending → Reject ===");
    println!("Expected: pending +50 after checkout, pending back to 0 after reject (no score change)");

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F02-001"),
        total: 1000.0,
        ..Default::default()
    }).await?;

    checkout_reject(client, &tx("TX-F02-001")).await?;

    println!("\nFlow 02 complete. Look for: pending cleared, score/spendable unchanged.");
    Ok(())
}
