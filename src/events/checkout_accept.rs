use anyhow::Result;
use serde_json::json;

use crate::antavo::{client::AntavoClient, state::CustomerState};
use crate::events::send_and_show;

pub async fn checkout_accept(
    client: &AntavoClient,
    transaction_id: &str,
) -> Result<CustomerState> {
    let body = json!({
        "customer": client.customer_id,
        "action": "checkout_accept",
        "data": {
            "transaction_id": transaction_id
        }
    });
    send_and_show(
        client,
        &format!("checkout_accept (tx: {})", transaction_id),
        body,
    )
    .await
}
