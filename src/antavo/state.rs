use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct CustomerState {
    pub score: i64,
    pub spent: i64,
    pub reserved: i64,
    pub pending: i64,
    pub point_balance: i64,
    pub spendable: i64,
}

impl CustomerState {
    pub fn from_json(v: &Value) -> Self {
        let get = |key: &str| v.get(key).and_then(|x| x.as_i64()).unwrap_or(0);
        Self {
            score: get("score"),
            spent: get("spent"),
            reserved: get("reserved"),
            pending: get("pending"),
            point_balance: get("point_balance"),
            spendable: get("spendable"),
        }
    }

    pub fn print(&self) {
        println!("  score:         {:>8}", self.score);
        println!("  spent:         {:>8}", self.spent);
        println!("  reserved:      {:>8}", self.reserved);
        println!("  pending:       {:>8}", self.pending);
        println!("  point_balance: {:>8}", self.point_balance);
        println!("  spendable:     {:>8}", self.spendable);
    }

    pub fn print_diff(before: &CustomerState, after: &CustomerState) {
        fn fmt_field(label: &str, before: i64, after: i64) {
            let delta = after - before;
            let delta_str = if delta == 0 {
                "     —".to_string()
            } else if delta > 0 {
                format!("  (+{})", delta)
            } else {
                format!("  ({})", delta)
            };
            println!("  {:<15} {:>8}  →  {:>8}{}", label, before, after, delta_str);
        }

        fmt_field("score:", before.score, after.score);
        fmt_field("spent:", before.spent, after.spent);
        fmt_field("reserved:", before.reserved, after.reserved);
        fmt_field("pending:", before.pending, after.pending);
        fmt_field("point_balance:", before.point_balance, after.point_balance);
        fmt_field("spendable:", before.spendable, after.spendable);
    }
}
