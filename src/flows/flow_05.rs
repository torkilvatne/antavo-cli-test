/// Flow 05 — Reserve and Release Points
///
/// Goal: Understand how reserve_points and release_points affect reserved and spendable.
/// Pre-condition: reset then give 1000 pts.
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::point_add::point_add;
use crate::events::release_points::release_points;
use crate::events::reserve_points::reserve_points;

pub async fn run(client: &AntavoClient) -> Result<()> {
    let sfx = crate::flows::tx_suffix();
    let tx = |s: &str| format!("{}-{}", s, sfx);

    println!("\n=== Flow 05: Reserve and Release Points ===");

    point_add(client, 1000, "Flow 05 baseline").await?;

    reserve_points(client, &tx("TX-F05-001"), 600).await?;
    reserve_points(client, &tx("TX-F05-002"), 200).await?;

    release_points(client, &tx("TX-F05-001"), Some(600)).await?;
    release_points(client, &tx("TX-F05-002"), Some(200)).await?;

    println!("\nFlow 05 complete. Final: reserved=0, spendable=1000.");
    Ok(())
}
