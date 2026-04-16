# antavo-flow-test

Interactive CLI for testing Antavo loyalty API flows. Replaces manual Postman work — fire individual events or run full predefined flows, with automatic before/after state diffs after every action.

## Setup

### 1. Build

```bash
cargo build
```

### 2. Configure credentials

```bash
cp .env.example .env
```

Edit `.env`:

```env
ANTAVO_BASE_URL=https://api.<env>.antavo.com
ANTAVO_API_KEY=your_api_key
ANTAVO_API_SECRET=your_api_secret
ANTAVO_CREDENTIAL_SCOPE=<date>/<env>/api/antavo_request
ANTAVO_CUSTOMER_ID=your-test-customer-uuid
```

### 3. Create a test customer (optional)

If you don't have a test customer yet:

```bash
cargo run -- create
```

This generates a new customer with a random UUID and prints it. Copy the ID into `ANTAVO_CUSTOMER_ID` in your `.env`.

---

## Commands

All commands use the customer from `ANTAVO_CUSTOMER_ID` in `.env`.

### `get` — Show current customer state

```bash
cargo run -- get
```

```
Customer: a3f8c21d-9e47-4b1c-8d02-f5e67a890bcd
  score:                1100
  spent:                 300
  reserved:                0
  pending:                 0
  point_balance:          800
  spendable:              800
```

---

### `reset` — Full customer state reset

```bash
cargo run -- reset
```

Brings the customer to a clean baseline in three steps:
1. **Rejects** all `pending` transactions (via `checkout_reject`)
2. **Refunds** all `accepted` transactions (via `refund`)
3. **Drains** remaining `point_balance` to 0 via `point_sub`

> Warns if `reserved > 0` — reservations cannot be auto-released without the original transaction ID; run `release_points` manually first.

---

### `create` — Create a new test customer

```bash
cargo run -- create
```

Sends an `opt_in` event with a fresh UUID and generated email. Prints the new customer ID.

```
[opt_in] Creating new customer...
  customer_id: a3f8c21d-9e47-4b1c-8d02-f5e67a890bcd
  ...

✔ Customer created successfully!
  customer_id: a3f8c21d-9e47-4b1c-8d02-f5e67a890bcd

To use this customer, update your .env:
  ANTAVO_CUSTOMER_ID=a3f8c21d-9e47-4b1c-8d02-f5e67a890bcd
```

---

### `point add/sub` — Add or subtract points

```bash
cargo run -- point add 500
cargo run -- point sub 200
cargo run -- point add 500 --reason "bonus"
cargo run -- point sub 100 --reason "adjustment"
```

Shorthand for firing `point_add` / `point_sub` events. `--reason` defaults to `"manual"` if omitted. Prints state diff after.

---

### `use` — Set active customer

```bash
cargo run -- use <customer-id>
```

Updates `ANTAVO_CUSTOMER_ID` in `.env`. Equivalent to editing the file manually. Useful when switching between test customers.

---

### `tx` — Fetch transactions

```bash
cargo run -- tx                           # list all transactions
cargo run -- tx TX-001                    # fetch single transaction by ID
cargo run -- tx --status pending          # filter by status
cargo run -- tx --status accepted
cargo run -- tx --status rejected
```

GETs `/customers/{id}/transactions` or `/customers/{id}/transactions/{tx_id}`. Status filtering is done client-side. Useful for inspecting transaction status and verifying `burned`/`earned` values after checkout operations.

> The older `transactions --id TX-xxx` command also exists but uses a query param instead of a path segment.

---

### `accept` / `reject` — Accept or reject a pending checkout

```bash
cargo run -- accept TX-001
cargo run -- reject TX-001
```

Fires `checkout_accept` or `checkout_reject` for the given transaction ID. Prints state diff after.

---

### `rewards` — List, claim, or revoke rewards

```bash
cargo run -- rewards                          # list active rewards
cargo run -- rewards --claimed                # list claimed rewards
cargo run -- rewards --claim REWARD_ID        # claim a reward
cargo run -- rewards --revoke REWARD_ID       # revoke a previously claimed reward
```

**List** fetches `GET /customers/{id}/activities/rewards` and outputs each reward with `id`, `title`, `description`, `points`, and `claims` (always `null` for this endpoint — `claims` is only populated on the claimed-rewards endpoint).

**Claimed** fetches `GET /customers/{id}/rewards` and prints the raw JSON response of all rewards the customer has claimed.

**Claim** posts to `/customers/{id}/activities/rewards/{reward_id}/claim`. Deducts the required points and prints a state diff after.

**Revoke** posts to `/customers/{id}/activities/rewards/{reward_id}/revoke`. Restores spent points and prints a state diff after.

---

### `checkout` — Fire a checkout event

```bash
cargo run -- checkout --total 2000
cargo run -- checkout --total 2000 --burn 150
cargo run -- checkout --total 2000 --burn 150 --channel restaurant
```

Fires a `checkout` event with a generated `transaction_id` (e.g. `TX-a3f2c819`) and today's date. Defaults: `channel=hotel`, `currency=NOK`. Prints state diff after.

Use `tx` to inspect the resulting transaction, `txdelta` to adjust it, and `cargo run -- event '{"action":"checkout_accept",...}'` or a flow to complete it.

---

### `txdelta` — Adjust a pending transaction by delta

```bash
cargo run -- txdelta TX-001 --burn 50         # increase burn by 50
cargo run -- txdelta TX-001 --burn -50        # decrease burn by 50
cargo run -- txdelta TX-001 --total 500       # increase total by 500
cargo run -- txdelta TX-001 --burn 50 --total -200   # both at once
```

Fetches the transaction, applies the given deltas to `points_burned` and/or `total`, then calls `checkout_update` with the new absolute values. Saves you from manually computing the new values when tweaking a pending checkout.

Prints a summary before sending:

```
TX-001: burned 200 → 250 (+50), total 2000 → 1800 (-200)
```

Guards:
- Transaction must have `status: pending` — bails otherwise (Antavo returns HTTP 400 on non-pending updates).
- `new_burned` must be `>= 0` — bails with a clear message if the delta would go negative.

---

### `event` — Fire a raw JSON event

```bash
cargo run -- event '<json>'
```

The `customer` field is injected automatically if omitted. State diff is printed after.

```bash
cargo run -- event '{"action":"point_add","data":{"points":500,"reason":"manual test"}}'
cargo run -- event '{"action":"checkout","data":{"transaction_id":"TX-001","total":2000,"channel":"hotel","transaction_date":"2026-03-03","currency":"NOK","points_burned":100}}'
cargo run -- event '{"action":"checkout_accept","data":{"transaction_id":"TX-001"}}'
```

---

### `flow list` — List predefined flows

```bash
cargo run -- flow list
```

```
Available flows:
  01  Earn Points: Checkout Pending → Accept
  02  Earn Points: Checkout Pending → Reject
  03  Direct Point Operations (add/spend/unspend/fix/sub)
  04  Burn Points at Checkout (points_burned)
  05  Reserve and Release Points
  06  Reserve Overflow + Burn (Part A: overdraft, Part B: same tx_id)
  07  Earn + Burn in Same Pending Checkout
  08  Full Refund After Accepted Checkout
  09  Partial Refund (two 50% refunds)
  10  Checkout Reject With Interleaved Events
  11  checkout_update Flow
```

---

### `flow run <number>` — Run a predefined flow

```bash
cargo run -- flow run 06
```

Each step prints a state diff:

```
=== Flow 06 Part A: Reserve Overflow Burn (different tx_id) ===
Expected: point_balance = -100 after checkout (overdraft allowed!)

[point_add (1000 pts)]
  Request: {"action":"point_add","customer":"...","data":{"points":1000,"reason":"Flow 06 Part A baseline"}}
  Response: {"action":"point_add","points":1000}
  State diff:
  score:            0  →  1000  (+1000)
  spent:            0  →     0  (  —)
  reserved:         0  →     0  (  —)
  pending:          0  →     0  (  —)
  point_balance:    0  →  1000  (+1000)
  spendable:        0  →  1000  (+1000)

[reserve_points (800 pts, tx: TX-F06-RESERVE)]
  ...
```

> **Tip:** Run `reset` before a flow to start from a known baseline.

---

## State Fields


| Field           | Formula                    | Notes                                        |
| --------------- | -------------------------- | -------------------------------------------- |
| `score`         | —                          | Lifetime points earned                       |
| `spent`         | —                          | Lifetime points burned/spent                 |
| `reserved`      | —                          | Points held by `reserve_points`              |
| `pending`       | —                          | Points earned, waiting for `checkout_accept` |
| `point_balance` | `score − spent − reserved` | The real balance; can go negative            |
| `spendable`     | `max(0, point_balance)`    | What the customer can actually spend         |


---

## Confirmed Antavo Behaviors

These were verified via live testing and are documented in the flow files under `.claude/knowledge/flows/`.

- `**points_burned` is immediate** — `spent` increases and `spendable` decreases on checkout submit, before `checkout_accept`.
- **Earned points go to `pending`** — credited to `score` only after `checkout_accept`.
- `**checkout_reject` reverses both** — undoes the burn (restores `spent`) and voids the pending earn.
- **Antavo allows overdraft** — `checkout` with `points_burned > spendable` goes through; `point_balance` goes negative.
- **Same `transaction_id` on `reserve_points` + `checkout`** — the reservation is consumed cleanly; no overdraft.
- `**checkout_accept` does NOT release reservations** — call `release_points` explicitly.
- **Hotel earn rate** — `CEIL(total × 0.05)` points per NOK (e.g. 2000 NOK = 100 pts). Channel `hotel` required.

---

## Project Structure

```
src/
├── main.rs               # CLI entry point (clap commands)
├── config.rs             # .env loader
├── escher/               # Escher HMAC-SHA256 signing
├── antavo/
│   ├── client.rs         # Signed HTTP client
│   └── state.rs          # CustomerState struct + diff display
├── events/               # One file per Antavo event type
│   ├── opt_in.rs
│   ├── point_add.rs
│   ├── point_sub.rs
│   ├── point_spend.rs
│   ├── point_fix.rs
│   ├── point_unspend.rs
│   ├── reserve_points.rs
│   ├── release_points.rs
│   ├── checkout.rs
│   ├── checkout_accept.rs
│   ├── checkout_reject.rs
│   ├── checkout_update.rs
│   ├── refund.rs
│   └── partial_refund.rs
└── flows/                # Predefined test flows (flow_01 – flow_11)
```

## Authentication

Uses Escher HMAC-SHA256 signing (same implementation as `lms-adapter`). The `Authorization` and `Date` headers are added automatically to every request — no manual signing needed.