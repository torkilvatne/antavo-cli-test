use anyhow::Result;
use serde_json::json;

use crate::antavo::{client::AntavoClient, state::CustomerState};
use crate::events::send_and_show;

pub async fn reserve_points(
    client: &AntavoClient,
    transaction_id: &str,
    points: i64,
) -> Result<CustomerState> {
    let body = json!({
        "customer": client.customer_id,
        "action": "reserve_points",
        "data": {
            "transaction_id": transaction_id,
            "points": points
        }
    });
    send_and_show(
        client,
        &format!("reserve_points ({} pts, tx: {})", points, transaction_id),
        body,
    )
    .await
}
