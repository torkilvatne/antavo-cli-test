/// Flow 03 — Direct Point Operations
///
/// Goal: Understand how each point event affects score, spent, and spendable.
/// Tests: point_add, point_spend, point_unspend, point_fix, point_sub
/// Net result from 0: score +900, spent +200, spendable +700
use anyhow::Result;

use crate::antavo::client::AntavoClient;
use crate::events::point_add::point_add;
use crate::events::point_spend::point_spend;
use crate::events::point_unspend::point_unspend;
use crate::events::point_fix::point_fix;
use crate::events::point_sub::point_sub;

pub async fn run(client: &AntavoClient) -> Result<()> {
    println!("\n=== Flow 03: Direct Point Operations ===");

    point_add(client, 1000, "Flow 03 baseline").await?;
    point_spend(client, 300, "Flow 03 spend test").await?;
    point_unspend(client, 100, "Flow 03 unspend test").await?;
    point_fix(client, 200, "Flow 03 fix positive").await?;
    point_fix(client, -100, "Flow 03 fix negative").await?;
    point_sub(client, 200, "Flow 03 sub test").await?;

    println!("\nFlow 03 complete. Net from baseline: score +900, spent +200, spendable +700.");
    Ok(())
}
