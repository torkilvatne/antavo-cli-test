use anyhow::Result;
use serde_json::json;

use crate::antavo::{client::AntavoClient, state::CustomerState};
use crate::events::send_and_show;

pub async fn checkout_update(
    client: &AntavoClient,
    transaction_id: &str,
    total: f64,
    points_burned: Option<i64>,
    channel: &str,
    currency: &str,
) -> Result<CustomerState> {
    let mut data = json!({
        "transaction_id": transaction_id,
        "total": total,
        "channel": channel,
        "currency": currency,
    });
    if let Some(burn) = points_burned {
        data.as_object_mut()
            .unwrap()
            .insert("points_burned".to_string(), serde_json::Value::from(burn));
    }

    let body = json!({
        "customer": client.customer_id_required()?,
        "action": "checkout_update",
        "data": data
    });
    send_and_show(
        client,
        &format!("checkout_update (total={}, points_burned={:?}, tx: {})", total, points_burned, transaction_id),
        body,
    )
    .await
}
