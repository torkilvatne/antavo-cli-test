/// Flow 08 — Full Refund After Accepted Checkout
///
/// Goal: Verify that refund reverses both earned points AND burned points.
/// Note: Refund does NOT auto-revoke coupons or rewards.
/// Pre-condition: reset then give 1000 pts.
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::checkout::{CheckoutParams, checkout};
use crate::events::checkout_accept::checkout_accept;
use crate::events::point_add::point_add;
use crate::events::refund::refund;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    println!("\n=== Flow 08: Full Refund After Accepted Checkout ===");
    println!("Expected after accept: score=1100, spent=300, spendable=800");
    println!("Expected after refund: score=1000, spent=0, spendable=1000 (full reversal)");

    point_add(client, 1000, "Flow 08 baseline").await?;

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F08-001"),
        total: 2000.0,
        points_burned: Some(300),
        ..Default::default()
    }).await?;

    checkout_accept(client, &tx("TX-F08-001")).await?;

    refund(client, &tx("TX-F08-001")).await?;

    println!("\nFlow 08 complete. Look for: score=1000, spent=0, spendable=1000.");
    Ok(())
}
