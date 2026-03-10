/// Flow 11 — checkout_update: Burn Adjustment Scenarios (FR/NFR 2.2)
///
/// Tests the checkout_update event for adjusting points_burned mid-flight.
/// Key rule from NFR 2.2: send the TOTAL burned amount each time (not a delta).
/// No auto_accept configured — explicit checkout_accept is required.
/// No `items` field in payloads (agreed with Antavo).
///
/// Scenario A: Increase burn via checkout_update (100 → 200)
/// Scenario B: Decrease burn via checkout_update (400 → 150)
/// Scenario C: Reduce burn to 0 then checkout_reject
/// Scenario D: checkout_update AFTER checkout_accept (invalid path — document actual response)
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::checkout::{CheckoutParams, checkout};
use crate::events::checkout_accept::checkout_accept;
use crate::events::checkout_reject::checkout_reject;
use crate::events::checkout_update::checkout_update;
use crate::events::point_add::point_add;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    // -------------------------------------------------------------------------
    println!("\n=== Flow 11 Scenario A: Burn increase via checkout_update ===");
    println!("Step 1: checkout(points_burned=100) → expect spent +100");
    println!("Step 2: checkout_update(points_burned=200) → expect spent to become 200 total");
    println!("Step 3: checkout_accept → transaction final");

    point_add(client, 1000, "Flow 11A baseline").await?;

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F11-A"),
        total: 2000.0,
        points_burned: Some(100),
        ..Default::default()
    }).await?;

    checkout_update(client, &tx("TX-F11-A"), 2000.0, Some(200), "hotel", "NOK").await?;

    checkout_accept(client, &tx("TX-F11-A")).await?;

    println!("\nScenario A complete. Expected final: spent increased by 200 total.");

    // -------------------------------------------------------------------------
    println!("\n=== Flow 11 Scenario B: Burn decrease via checkout_update ===");
    println!("Step 1: checkout(points_burned=400) → expect spent +400");
    println!("Step 2: checkout_update(points_burned=150) → expect spent to drop to 150 (250 returned)");
    println!("Step 3: checkout_accept → transaction final");

    point_add(client, 1000, "Flow 11B baseline").await?;

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F11-B"),
        total: 2000.0,
        points_burned: Some(400),
        ..Default::default()
    }).await?;

    checkout_update(client, &tx("TX-F11-B"), 2000.0, Some(150), "hotel", "NOK").await?;

    checkout_accept(client, &tx("TX-F11-B")).await?;

    println!("\nScenario B complete. Expected final: spent increased by 150 total (not 400).");

    // -------------------------------------------------------------------------
    println!("\n=== Flow 11 Scenario C: Reduce burn to 0, then checkout_reject ===");
    println!("Step 1: checkout(points_burned=300) → expect spent +300");
    println!("Step 2: checkout_update(points_burned=0) → expect spent back to 0");
    println!("Step 3: checkout_reject → full cancellation, no pending earn");

    point_add(client, 500, "Flow 11C baseline").await?;

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F11-C"),
        total: 1000.0,
        points_burned: Some(300),
        ..Default::default()
    }).await?;

    checkout_update(client, &tx("TX-F11-C"), 1000.0, Some(0), "hotel", "NOK").await?;

    checkout_reject(client, &tx("TX-F11-C")).await?;

    println!("\nScenario C complete. Expected: spent unchanged from before scenario (burn fully returned before reject).");

    // -------------------------------------------------------------------------
    println!("\n=== Flow 11 Scenario D: checkout_update AFTER checkout_accept (invalid path) ===");
    println!("Step 1: checkout(points_burned=100)");
    println!("Step 2: checkout_accept → finalizes transaction");
    println!("Step 3: checkout_update(points_burned=200) → SHOULD FAIL per spec");
    println!("Documenting actual Antavo response...");

    point_add(client, 500, "Flow 11D baseline").await?;

    checkout(client, CheckoutParams {
        transaction_id: tx("TX-F11-D"),
        total: 1000.0,
        points_burned: Some(100),
        ..Default::default()
    }).await?;

    checkout_accept(client, &tx("TX-F11-D")).await?;

    println!("\n[checkout_update after checkout_accept]");
    match checkout_update(client, &tx("TX-F11-D"), 1000.0, Some(200), "hotel", "NOK").await {
        Ok(state) => {
            println!("  UNEXPECTED SUCCESS: checkout_update succeeded after checkout_accept!");
            println!("  Post-update state: score={}, spent={}, spendable={}",
                state.score, state.spent, state.spendable);
        }
        Err(e) => {
            println!("  Expected error (document this): {}", e);
        }
    }

    println!("\nFlow 11 complete.");
    println!("Write results to .claude/knowledge/flows/flow-11-burn-rules.md");
    Ok(())
}
