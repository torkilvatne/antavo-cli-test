use anyhow::Result;
use serde_json::json;

use crate::antavo::{client::AntavoClient, state::CustomerState};
use crate::events::send_and_show;

pub async fn refund(client: &AntavoClient, transaction_id: &str) -> Result<CustomerState> {
    let body = json!({
        "customer": client.customer_id,
        "action": "refund",
        "data": {
            "transaction_id": transaction_id
        }
    });
    send_and_show(
        client,
        &format!("refund (tx: {})", transaction_id),
        body,
    )
    .await
}
