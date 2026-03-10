use anyhow::Result;
use serde_json::json;

use crate::antavo::{client::AntavoClient, state::CustomerState};
use crate::events::send_and_show;

pub async fn partial_refund(
    client: &AntavoClient,
    transaction_id: &str,
    amount: f64,
) -> Result<CustomerState> {
    let body = json!({
        "customer": client.customer_id,
        "action": "partial_refund",
        "data": {
            "transaction_id": transaction_id,
            "amount": amount
        }
    });
    send_and_show(
        client,
        &format!("partial_refund (amount={}, tx: {})", amount, transaction_id),
        body,
    )
    .await
}
