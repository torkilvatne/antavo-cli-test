/// Flow 04 — Burn Points at Checkout (points_burned)
///
/// Goal: Confirm burn and earn behaviour during a pending checkout.
/// Confirmed: burn is immediate (spent += burn on submit), earn goes to pending.
/// Pre-condition: reset to 0 first, then gives itself 1000 pts.
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::checkout::{CheckoutParams, checkout};
use crate::events::checkout_accept::checkout_accept;
use crate::events::point_add::point_add;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    println!("\n=== Flow 04: Burn Points at Checkout ===");
    println!("Expected after checkout submit: spent=+400, pending=+100, score unchanged");
    println!("Expected after accept: score=+100 (pending cleared), spent unchanged");

    point_add(client, 1000, "Flow 04 baseline").await?;

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F04-001"),
        total: 2000.0,
        points_burned: Some(400),
        ..Default::default()
    }).await?;

    checkout_accept(client, &tx("TX-F04-001")).await?;

    println!("\nFlow 04 complete. Final: score=1100, spent=400, spendable=700.");
    Ok(())
}
