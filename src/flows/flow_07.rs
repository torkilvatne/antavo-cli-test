/// Flow 07 — Earn + Burn in Same Pending Checkout
///
/// Goal: Single checkout that both burns and earns.
/// Burn is immediate, earn goes to pending until accept.
/// Pre-condition: reset then give 1000 pts.
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::checkout::{CheckoutParams, checkout};
use crate::events::checkout_accept::checkout_accept;
use crate::events::point_add::point_add;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    println!("\n=== Flow 07: Earn + Burn in Same Checkout ===");
    println!("Expected after submit: spent=200, pending=100, score=1000");
    println!("Expected after accept: score=1100, spent=200, spendable=900");

    point_add(client, 1000, "Flow 07 baseline").await?;

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F07-001"),
        total: 2000.0,
        points_burned: Some(200),
        ..Default::default()
    }).await?;

    checkout_accept(client, &tx("TX-F07-001")).await?;

    println!("\nFlow 07 complete. Final: score=1100, spent=200, spendable=900.");
    Ok(())
}
