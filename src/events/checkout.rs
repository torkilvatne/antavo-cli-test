use anyhow::Result;
use serde_json::{Value, json};

use crate::antavo::{client::AntavoClient, state::CustomerState};
use crate::events::send_and_show;

pub struct CheckoutParams {
    pub transaction_id: String,
    pub total: f64,
    pub channel: String,
    pub transaction_date: String,
    pub currency: String,
    pub points_burned: Option<i64>,
}

impl Default for CheckoutParams {
    fn default() -> Self {
        Self {
            transaction_id: String::new(),
            total: 0.0,
            channel: "hotel".to_string(),
            transaction_date: "2026-03-03".to_string(),
            currency: "NOK".to_string(),
            points_burned: None,
        }
    }
}

pub async fn checkout(
    client: &AntavoClient,
    params: CheckoutParams,
) -> Result<CustomerState> {
    let mut data = json!({
        "transaction_id": params.transaction_id,
        "total": params.total,
        "channel": params.channel,
        "transaction_date": params.transaction_date,
        "currency": params.currency,
    });

    if let Some(burn) = params.points_burned {
        data.as_object_mut()
            .unwrap()
            .insert("points_burned".to_string(), Value::from(burn));
    }

    let body = json!({
        "customer": client.customer_id,
        "action": "checkout",
        "data": data
    });

    let label = if let Some(burn) = params.points_burned {
        format!("checkout (total={}, burn={}, tx: {})", params.total, burn, params.transaction_id)
    } else {
        format!("checkout (total={}, tx: {})", params.total, params.transaction_id)
    };

    send_and_show(client, &label, body).await
}
