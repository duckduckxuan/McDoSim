mod model;
mod random;
mod station;
mod kitchen;
mod dashboard;
mod utils;

use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use crate::{kitchen::{Kitchen, process_order}, random::generator_orders, station::{ProgressEvent, station_channel}, utils::order_total_items};


#[tokio::main]
async fn main() {
    // Generate random orders
    let orders_n = 10;
    let orders = generator_orders(orders_n, 42);

    // Dump orders to a text file for reference
    let mut order_dump = String::new();
    for order in &orders {
        order_dump.push_str(&format!("Order ID: {}\n", order.id));
        for line in &order.lines {
            order_dump.push_str(&format!(
                "  item={:?}, quantity={}, size={:?}\n",
                line.item, line.quantity, line.size
            ));
        }
        order_dump.push('\n');
    }
    std::fs::write("orders.txt", &order_dump).unwrap();

    // Precompute total items per order for dashboard progress tracking
    let mut totals = vec![0usize; 256];
    for o in &orders {
        totals[o.id as usize] = order_total_items(o);
    }

    // Initialize kitchen stations with concurrency limits
    let kitchen = Kitchen {
        grill: station_channel("Grill", 3, 10),
        fryer: station_channel("Fryer", 2, 10),
        drink: station_channel("Drink", 2, 10),
    };

    // Progress reporting channel
    let (progress_tx, progress_rx) = mpsc::unbounded_channel::<ProgressEvent>();
    let start = Instant::now();

    // Dashboard task, event-driven with debounce
    tokio::spawn(dashboard::dashboard_task(
        progress_rx,
        kitchen.clone(),
        totals.clone(),
        start,
        orders_n,
    ));

    // Launch concurrent order processing
    let mut set = tokio::task::JoinSet::new();
    for order in orders {
        let kitchen = kitchen.clone();
        let progress_tx = progress_tx.clone();
        set.spawn(async move {
            process_order(kitchen, order, progress_tx).await;
        });
    }
    drop(progress_tx);

    // Await all order processing tasks
    while let Some(res) = set.join_next().await {
        res.unwrap();
    }

    // Allow dashboard to render final state
    tokio::time::sleep(Duration::from_millis(500)).await;
}