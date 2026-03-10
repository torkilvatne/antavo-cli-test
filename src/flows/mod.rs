pub mod flow_01;
pub mod flow_02;
pub mod flow_03;
pub mod flow_04;
pub mod flow_05;
pub mod flow_06;
pub mod flow_07;
pub mod flow_08;
pub mod flow_09;
pub mod flow_10;
pub mod flow_11;

pub fn tx_suffix() -> i64 {
    jiff::Timestamp::now().as_millisecond() % 1_000_000
}

pub fn list() {
    println!("Available flows:");
    println!("  01  Earn Points: Checkout Pending → Accept");
    println!("  02  Earn Points: Checkout Pending → Reject");
    println!("  03  Direct Point Operations (add/spend/unspend/fix/sub)");
    println!("  04  Burn Points at Checkout (points_burned)");
    println!("  05  Reserve and Release Points");
    println!("  06  Reserve Overflow + Burn (Part A: overdraft, Part B: same tx_id)");
    println!("  07  Earn + Burn in Same Pending Checkout");
    println!("  08  Full Refund After Accepted Checkout");
    println!("  09  Partial Refund (two 50% refunds)");
    println!("  10  Checkout Reject With Interleaved Events");
    println!("  11  checkout_update: Burn Adjustment Scenarios (FR/NFR 2.2)");
}
