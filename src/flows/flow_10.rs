/// Flow 10 — Checkout Reject: Burn + Earn Reversal With Interleaved Events
///
/// Goal: Confirm checkout_reject reverses both points_burned AND voids pending earn,
/// even when other operations (point_add, point_spend) happen between submit and reject.
/// Those other operations must NOT be affected by the reject.
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::checkout::{CheckoutParams, checkout};
use crate::events::checkout_reject::checkout_reject;
use crate::events::point_add::point_add;
use crate::events::point_spend::point_spend;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    println!("\n=== Flow 10: Checkout Reject With Interleaved Events ===");
    println!("Pre-checkout: 1000 pts");
    println!("After checkout submit: spent=400, pending=100");
    println!("After point_add(200): score=1200");
    println!("After point_spend(100): spent=500");
    println!("After checkout_reject: spent=100 (only the point_spend remains), pending=0");
    println!("Key: reject reverses the 400 burn but NOT the interleaved point_spend");

    point_add(client, 1000, "Flow 10 baseline").await?;

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F10-001"),
        total: 2000.0,
        points_burned: Some(400),
        ..Default::default()
    }).await?;

    // Interleaved operations between submit and reject
    point_add(client, 200, "Flow 10 interleaved add").await?;
    point_spend(client, 100, "Flow 10 interleaved spend").await?;

    checkout_reject(client, &tx("TX-F10-001")).await?;

    println!("\nFlow 10 complete.");
    println!("Look for: score=1200, spent=100, spendable=1100, pending=0.");
    Ok(())
}
