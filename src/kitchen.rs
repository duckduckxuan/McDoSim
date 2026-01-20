use tokio::sync::{mpsc, oneshot};
use crate::{model::*, station::*};


// Collection of all station handles
#[derive(Debug, Clone)]
pub struct Kitchen {
    pub grill: StationHandle,
    pub fryer: StationHandle,
    pub drink: StationHandle,
}


// Submit all jobs of an order and wait for completion
pub async fn process_order(kitchen: Kitchen, order: Order, progress_tx: mpsc::UnboundedSender<ProgressEvent>) -> Vec<PreparedItem> {
    let mut waiting:Vec<oneshot::Receiver<PreparedItem>> = Vec::new();

    for line in order.lines {
        for _ in 0..line.quantity {
            let (done_tx, done_rx) = oneshot::channel::<PreparedItem>();

            let job = Job {
                order_id: order.id,
                item_kind: line.item.clone(),
                duration: line.prep_time(),
                if_done: done_tx,
                progress_tx: progress_tx.clone(),
            };

            match line.item.station() {
                Station::Grill => kitchen.grill.tx.send(job).await.unwrap(),
                Station::Fryer => kitchen.fryer.tx.send(job).await.unwrap(),
                Station::Drink => kitchen.drink.tx.send(job).await.unwrap(),
            }

            waiting.push(done_rx);
        }
    }

    let mut prepared = Vec::with_capacity(waiting.len());

    for rx in waiting {
        if let Ok(item) = rx.await {
            prepared.push(item);
        }
    }

    prepared
}