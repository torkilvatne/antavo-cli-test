/// Flow 01 — Earn Points: Checkout Pending → Accept
///
/// Goal: Verify that earned points land in `pending` on checkout submit and
/// move to `score`/`spendable` only on `checkout_accept`.
/// Hotel earn rate: 5% of total NOK, CEIL rounding (1000 NOK = 50 pts).
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::checkout::{CheckoutParams, checkout};
use crate::events::checkout_accept::checkout_accept;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    println!("\n=== Flow 01: Earn Points — Pending → Accept ===");
    println!("Expected: pending +50 after checkout, score +50 after accept");

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F01-001"),
        total: 1000.0,
        ..Default::default()
    }).await?;

    checkout_accept(client, &tx("TX-F01-001")).await?;

    println!("\nFlow 01 complete. Look for: pending cleared, score/spendable +50.");
    Ok(())
}
