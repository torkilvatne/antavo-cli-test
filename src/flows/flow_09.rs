/// Flow 09 — Partial Refund
///
/// Goal: Verify partial_refund reverses points proportionally (50% refund = 50% points reversed).
/// Two partial refunds that together cover 100% of the transaction.
/// Pre-condition: reset then give 500 pts.
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::checkout::{CheckoutParams, checkout};
use crate::events::checkout_accept::checkout_accept;
use crate::events::partial_refund::partial_refund;
use crate::events::point_add::point_add;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    println!("\n=== Flow 09: Partial Refund ===");
    println!("Checkout total=2000, earn=100 pts. Two partial refunds of 1000 each.");
    println!("Expected after 1st partial (50%): score -= 50  → 550");
    println!("Expected after 2nd partial (50%): score -= 50  → 500 (back to baseline)");

    point_add(client, 500, "Flow 09 baseline").await?;

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F09-001"),
        total: 2000.0,
        ..Default::default()
    }).await?;

    checkout_accept(client, &tx("TX-F09-001")).await?;

    partial_refund(client, &tx("TX-F09-001"), 1000.0).await?;
    partial_refund(client, &tx("TX-F09-001"), 1000.0).await?;

    println!("\nFlow 09 complete. Look for: score=500, spendable=500.");
    Ok(())
}
