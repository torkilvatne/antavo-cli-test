use anyhow::Result;
use serde_json::json;

use crate::antavo::{client::AntavoClient, state::CustomerState};
use crate::events::send_and_show;

pub async fn point_fix(client: &AntavoClient, points: i64, reason: &str) -> Result<CustomerState> {
    let body = json!({
        "customer": client.customer_id,
        "action": "point_fix",
        "data": {
            "points": points,
            "reason": reason
        }
    });
    send_and_show(client, &format!("point_fix ({} pts)", points), body).await
}
