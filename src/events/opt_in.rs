use anyhow::Result;
use serde_json::json;
use uuid::Uuid;

use crate::antavo::client::AntavoClient;
use crate::antavo::state::CustomerState;

/// Result of a customer creation — includes the new customer ID.
pub struct CreatedCustomer {
    pub customer_id: String,
    #[allow(dead_code)]
    pub state: CustomerState,
}

/// Create a new Antavo customer via opt_in.
///
/// Generates a fresh UUID for the customer ID and a unique email using the
/// current timestamp. All other fields use the standard QA defaults.
/// Returns the new customer ID — add it to .env as ANTAVO_CUSTOMER_ID to
/// use it in subsequent commands.
pub async fn opt_in(client: &AntavoClient) -> Result<CreatedCustomer> {
    let customer_id = Uuid::new_v4().to_string();

    // date_stamp: YYYYMMDD, id: last 4 hex chars of UUID for uniqueness
    let now = jiff::Timestamp::now();
    let date_stamp = now.strftime("%Y%m%d").to_string();
    let id = &customer_id.replace('-', "")[28..]; // last 4 hex chars

    let body = json!({
        "customer": customer_id,
        "action": "opt_in",
        "data": {
            "email": format!("antavo.qa+{}+{}@antavo.com", date_stamp, id),
            "first_name": format!("Antavo QA {}", id),
            "last_name": format!("Test {}", date_stamp),
            "country": "NO",
            "postal_code": "0680",
            "city": "Oslo",
            "region": "Oslo",
            "gender": "male",
            "mobile_phone": format!("+36301234{}", id),
            "birth_date": "2000-01-01",
            "enrollment_date": "2025-01-01",
            "enrollment_channel": "Center",
            "enrollment_property_code": "TSALFALT",
            "enrollment_subchannel_code": "TSALFALT",
            "is_email_consent": true,
            "is_sms_consent": true,
            "is_push_consent": true,
            "is_advertising_consent": true,
            "is_thon_app_downloaded": true,
            "is_membership_consent": true,
            "is_employee": false,
            "is_migrated": false,
            "migrated_tier": "Gold",
            "is_previous_member_center": true,
            "is_previous_member_hotel": true,
            "is_special_treatment": false,
        }
    });

    println!("\n[opt_in] Creating new customer...");
    println!("  customer_id: {}", customer_id);
    println!("  Request: {}", serde_json::to_string(&body)?);

    let response = client.post_event(body).await?;
    println!("  Response: {}", serde_json::to_string(&response)?);

    // Fetch the newly created customer's state
    let raw = client.get_customer_raw_for(&customer_id).await?;
    let state = CustomerState::from_json(&raw);

    println!("  Initial state:");
    state.print();

    Ok(CreatedCustomer { customer_id, state })
}
