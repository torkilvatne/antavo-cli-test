use anyhow::Result;
use serde_json::json;

use crate::antavo::{client::AntavoClient, state::CustomerState};
use crate::events::send_and_show;

pub async fn release_points(
    client: &AntavoClient,
    transaction_id: &str,
    points: Option<i64>,
) -> Result<CustomerState> {
    let mut data = json!({ "transaction_id": transaction_id });
    if let Some(pts) = points {
        data.as_object_mut()
            .unwrap()
            .insert("points".to_string(), serde_json::Value::from(pts));
    }

    let body = json!({
        "customer": client.customer_id,
        "action": "release_points",
        "data": data
    });
    send_and_show(
        client,
        &format!("release_points (tx: {})", transaction_id),
        body,
    )
    .await
}
