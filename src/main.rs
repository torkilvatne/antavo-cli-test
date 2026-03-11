#![recursion_limit = "256"]
mod antavo;
mod config;
mod escher;
mod events;
mod flows;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use serde_json::Value;
use uuid::Uuid;

use antavo::{client::AntavoClient, state::CustomerState};
use config::Config;
use events::send_and_show;

#[derive(Parser)]
#[command(name = "antavo-flow-test")]
#[command(about = "Interactive CLI for testing Antavo loyalty API flows")]
struct Cli {
    #[arg(short = 'c', long, global = true)]
    customer: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show the current customer state
    Get,

    /// Fire a raw JSON event (auto-fetches state diff after)
    ///
    /// Example: antavo-flow-test event '{"action":"point_add","data":{"points":100,"reason":"test"}}'
    Event {
        /// JSON event body (customer field will be injected if missing)
        json: String,
    },

    /// Run a predefined flow or list all flows
    Flow {
        #[command(subcommand)]
        sub: FlowCommand,
    },

    /// Reset customer points to zero spendable
    ///
    /// Drains spendable to 0 using point_sub. Warns if reserved or pending > 0.
    Reset,

    /// Create a new Antavo customer (opt_in) and print the new customer ID
    Create,

    /// Set the active customer ID in .env
    Use {
        /// The customer ID to set as active
        customer_id: String,
    },

    /// Add or subtract points
    Point {
        #[command(subcommand)]
        sub: PointCommand,
    },

    /// Fetch transaction history (GET /customers/{id}/transactions)
    ///
    /// Example: transactions --id TX-F11-C-177325
    Transactions {
        /// Filter by a specific transaction ID
        #[arg(long)]
        id: Option<String>,
    },

    /// Fetch a transaction by ID, or list all transactions
    ///
    /// Example: tx TX-F11-C-177325
    /// Example: tx
    /// Example: tx --status pending
    Tx {
        /// Transaction ID (optional — omit to list all)
        id: Option<String>,
        /// Filter by status: pending, accepted, rejected
        #[arg(long)]
        status: Option<String>,
    },

    /// Fire a checkout event with auto-generated transaction_id and today's date
    ///
    /// Example: checkout --total 2000
    /// Example: checkout --total 2000 --burn 150 --channel restaurant
    Checkout {
        /// Sale total in NOK
        #[arg(long)]
        total: f64,
        /// Points to burn (optional)
        #[arg(long)]
        burn: Option<i64>,
        /// Channel (default: hotel)
        #[arg(long, default_value = "hotel")]
        channel: String,
    },

    /// Adjust a pending transaction's burn or total by delta, then call checkout_update
    ///
    /// Example: txdelta TX-001 --burn 50 --total -200
    Txdelta {
        /// Transaction ID
        id: String,
        /// Delta to apply to points_burned (positive or negative)
        #[arg(long, allow_hyphen_values = true)]
        burn: Option<i64>,
        /// Delta to apply to total (positive or negative)
        #[arg(long, allow_hyphen_values = true)]
        total: Option<f64>,
    },
}

#[derive(Subcommand)]
enum PointCommand {
    /// Add points to the customer
    Add {
        points: i64,
        #[arg(long, default_value = "manual")]
        reason: String,
    },
    /// Subtract points from the customer
    Sub {
        points: i64,
        #[arg(long, default_value = "manual")]
        reason: String,
    },
}

#[derive(Subcommand)]
enum FlowCommand {
    /// List available flows
    List,
    /// Run a flow by number (e.g. "06")
    Run { number: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::from_env()?;
    if let Some(id) = cli.customer {
        config.customer_id = Some(id);
    }
    let client = AntavoClient::new(config);

    match cli.command {
        Command::Get => {
            let state = client.get_customer_state().await?;
            println!("Customer: {}", client.customer_id_required()?);
            state.print();
        }

        Command::Event { json } => {
            let mut body: Value =
                serde_json::from_str(&json).map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;

            // Inject customer if not present
            if body.get("customer").is_none() {
                body.as_object_mut()
                    .ok_or_else(|| anyhow::anyhow!("Event JSON must be an object"))?
                    .insert(
                        "customer".to_string(),
                        Value::String(client.customer_id_required()?.to_string()),
                    );
            }

            let action = body
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("event")
                .to_string();

            send_and_show(&client, &action, body).await?;
        }

        Command::Flow { sub } => match sub {
            FlowCommand::List => flows::list(),
            FlowCommand::Run { number } => run_flow(&client, &number).await?,
        },

        Command::Reset => {
            reset(&client).await?;
        }

        Command::Create => {
            let created = events::opt_in::opt_in(&client).await?;
            write_customer_id_to_env(&created.customer_id)?;
            println!("\n✔ Customer created successfully!");
            println!("  customer_id: {}", created.customer_id);
            println!(".env updated: ANTAVO_CUSTOMER_ID={}", created.customer_id);
        }

        Command::Use { customer_id } => {
            write_customer_id_to_env(&customer_id)?;
            println!("Updated .env: ANTAVO_CUSTOMER_ID={}", customer_id);
        }

        Command::Point { sub } => match sub {
            PointCommand::Add { points, reason } => {
                events::point_add::point_add(&client, points, &reason).await?;
            }
            PointCommand::Sub { points, reason } => {
                events::point_sub::point_sub(&client, points, &reason).await?;
            }
        },

        Command::Checkout { total, burn, channel } => {
            let tx_id = format!("TX-{}", &Uuid::new_v4().to_string()[..8]);
            let tx_date = jiff::Timestamp::now().strftime("%Y-%m-%d").to_string();
            events::checkout::checkout(&client, events::checkout::CheckoutParams {
                transaction_id: tx_id,
                total,
                channel,
                transaction_date: tx_date,
                currency: "NOK".to_string(),
                points_burned: burn,
            }).await?;
        }

        Command::Transactions { id } => {
            let filter = id.as_deref();
            let result = client.get_customer_transactions(filter).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Command::Tx { id, status } => {
            let mut result = client.get_transaction(id.as_deref()).await?;
            if let Some(ref status_filter) = status {
                if let Some(data) = result.get_mut("data").and_then(|d| d.as_array_mut()) {
                    let filtered: Vec<Value> = data
                        .iter()
                        .filter(|tx| {
                            tx.get("status")
                                .and_then(|s| s.as_str())
                                .map(|s| s == status_filter.as_str())
                                .unwrap_or(false)
                        })
                        .cloned()
                        .collect();
                    *data = filtered;
                }
            }
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        Command::Txdelta {
            id,
            burn: burn_delta,
            total: total_delta,
        } => {
            let tx = client.get_transaction(Some(&id)).await?;

            let status = tx.get("status").and_then(|v| v.as_str()).unwrap_or("");
            if status != "pending" {
                bail!(
                    "Transaction {} has status '{}', not 'pending'. checkout_update requires pending.",
                    id,
                    status
                );
            }

            let current_burned: i64 = tx
                .get("burned")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let current_total: f64 = tx.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let channel = tx
                .get("channel")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Transaction {} missing 'channel' field", id))?
                .to_string();
            let currency = tx
                .get("currency")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Transaction {} missing 'currency' field", id))?
                .to_string();

            let new_burned = current_burned + burn_delta.unwrap_or(0);
            let new_total = current_total + total_delta.unwrap_or(0.0);

            if new_burned < 0 {
                bail!(
                    "New burned would be {} (current {} + delta {}), which is negative. Aborting.",
                    new_burned,
                    current_burned,
                    burn_delta.unwrap_or(0)
                );
            }

            // Print summary line
            let burn_str = match burn_delta {
                Some(d) => format!("burned {} → {} ({:+})", current_burned, new_burned, d),
                None => format!("burned {} (unchanged)", current_burned),
            };
            let total_str = match total_delta {
                Some(d) => format!("total {} → {} ({:+})", current_total, new_total, d),
                None => format!("total {} (unchanged)", current_total),
            };
            println!("{}: {}, {}", id, burn_str, total_str);

            events::checkout_update::checkout_update(
                &client,
                &id,
                new_total,
                Some(new_burned),
                &channel,
                &currency,
            )
            .await?;
        }
    }

    Ok(())
}

fn write_customer_id_to_env(id: &str) -> Result<()> {
    let path = std::path::Path::new(".env");
    let existing = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };
    let key = "ANTAVO_CUSTOMER_ID";
    let new_line = format!("{}={}", key, id);
    let updated = if existing
        .lines()
        .any(|l| l.starts_with(&format!("{}=", key)))
    {
        existing
            .lines()
            .map(|l| {
                if l.starts_with(&format!("{}=", key)) {
                    new_line.as_str()
                } else {
                    l
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    } else if existing.is_empty() || existing.ends_with('\n') {
        format!("{}{}\n", existing, new_line)
    } else {
        format!("{}\n{}\n", existing, new_line)
    };
    std::fs::write(path, updated)?;
    Ok(())
}

async fn run_flow(client: &AntavoClient, number: &str) -> Result<()> {
    match number.trim_start_matches('0') {
        "1" => flows::flow_01::run(client).await?,
        "2" => flows::flow_02::run(client).await?,
        "3" => flows::flow_03::run(client).await?,
        "4" => flows::flow_04::run(client).await?,
        "5" => flows::flow_05::run(client).await?,
        "6" => flows::flow_06::run(client).await?,
        "7" => flows::flow_07::run(client).await?,
        "8" => flows::flow_08::run(client).await?,
        "9" => flows::flow_09::run(client).await?,
        "10" => flows::flow_10::run(client).await?,
        "11" => flows::flow_11::run(client).await?,
        _ => bail!(
            "Unknown flow '{}'. Run `flow list` to see available flows.",
            number
        ),
    }
    Ok(())
}

async fn reset(client: &AntavoClient) -> Result<()> {
    use serde_json::json;

    println!(
        "Resetting customer {}...",
        client.customer_id_required()?
    );

    // Step 1: fetch all transactions and handle pending/accepted ones
    let tx_list = client.get_transaction(None).await?;
    let txs = tx_list
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    let pending: Vec<String> = txs
        .iter()
        .filter(|tx| tx.get("status").and_then(|s| s.as_str()) == Some("pending"))
        .filter_map(|tx| tx.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();

    let accepted: Vec<String> = txs
        .iter()
        .filter(|tx| tx.get("status").and_then(|s| s.as_str()) == Some("accepted"))
        .filter_map(|tx| tx.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();

    if pending.is_empty() && accepted.is_empty() {
        println!("No pending or accepted transactions to clear.");
    }

    for tx_id in &pending {
        println!("Rejecting pending transaction {}...", tx_id);
        events::checkout_reject::checkout_reject(client, tx_id).await?;
    }

    for tx_id in &accepted {
        println!("Refunding accepted transaction {}...", tx_id);
        events::refund::refund(client, tx_id).await?;
    }

    // Step 2: drain remaining point_balance to 0
    let state = client.get_customer_state().await?;
    println!("\nState after transaction cleanup:");
    state.print();

    if state.reserved > 0 {
        eprintln!(
            "WARNING: reserved={} — cannot auto-release without transaction IDs. \
             Run release_points manually before resetting.",
            state.reserved
        );
    }

    if state.point_balance <= 0 {
        println!("point_balance is already <= 0, nothing to drain.");
        return Ok(());
    }

    let drain = state.point_balance;
    println!("\nSending point_sub({}) to drain point_balance to 0...", drain);

    let body = json!({
        "customer": client.customer_id_required()?,
        "action": "point_sub",
        "data": {
            "points": drain,
            "reason": "reset"
        }
    });

    let before = client.get_customer_state().await?;
    client.post_event(body).await?;
    let after = client.get_customer_state().await?;

    println!("State diff:");
    CustomerState::print_diff(&before, &after);
    println!(
        "Reset complete. point_balance={}, spendable={}",
        after.point_balance, after.spendable
    );

    Ok(())
}
