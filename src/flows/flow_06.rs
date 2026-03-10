/// Flow 06 — Reserve Overflow + Burn
///
/// Goal: Test overdraft when burn > spendable after a reservation.
/// Part A: burn on DIFFERENT tx_id → reserved untouched, point_balance goes negative
/// Part B: burn on SAME tx_id → reservation consumed cleanly, no overdraft
///
/// Confirmed behaviours:
/// - Different tx_id: reserved stays, point_balance goes negative, Antavo DOES NOT block
/// - Same tx_id: reservation consumed (reserved→0), point_balance stays positive
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::checkout::{CheckoutParams, checkout};
use crate::events::checkout_accept::checkout_accept;
use crate::events::point_add::point_add;
use crate::events::reserve_points::reserve_points;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    println!("\n=== Flow 06 Part A: Reserve Overflow Burn (different tx_id) ===");
    println!("Expected: point_balance = -100 after checkout (overdraft allowed!)");

    point_add(client, 1000, "Flow 06 Part A baseline").await?;
    reserve_points(client, &tx("TX-F06-RESERVE"), 800).await?;

    // Burn 300 on a DIFFERENT tx — 100 over spendable (200)
    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F06-BURN-OVER"),
        total: 2000.0,
        points_burned: Some(300),
        ..Default::default()
    }).await?;

    checkout_accept(client, &tx("TX-F06-BURN-OVER")).await?;

    println!("\n--- Flow 06 Part B: Same tx_id consumes reservation cleanly ---");
    println!("Expected: reserved drops to 0, no overdraft");

    point_add(client, 1000, "Flow 06 Part B baseline").await?;
    reserve_points(client, &tx("TX-F06-LINKED"), 800).await?;

    // Burn 800 on the SAME tx as reservation — reservation consumed
    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F06-LINKED"),
        total: 2000.0,
        points_burned: Some(800),
        ..Default::default()
    }).await?;

    checkout_accept(client, &tx("TX-F06-LINKED")).await?;

    println!("\nFlow 06 complete.");
    Ok(())
}
