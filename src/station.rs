use tokio::{spawn, sync::{Mutex, mpsc, oneshot}};
use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, time::Duration};
use crate::model::{ItemKind, PreparedItem};


// Events sent from workers to the dashboard
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    ItemDone(PreparedItem),
}

// Job sent to a station worker
#[derive(Debug)]
pub struct Job {
    pub order_id: u8,
    pub item_kind: ItemKind,
    pub duration: Duration,
    pub if_done: oneshot::Sender<PreparedItem>,
    pub progress_tx: mpsc::UnboundedSender<ProgressEvent>,
}

// Handle to a kitchen station
#[derive(Debug, Clone)]
pub struct StationHandle {
    pub name: &'static str,
    pub tx: mpsc::Sender<Job>,
    pub buffer: usize,
    pub workers: usize,
    pub active: Arc<AtomicUsize>,
}


impl StationHandle {
    // Current queue length
    pub fn queue_len(&self) -> usize {
        self.buffer.saturating_sub(self.tx.capacity())
    }

    // Number of active workers
    pub fn active(&self) -> usize {
        self.active.load(Ordering::Relaxed)
    }
}


// Worker task that processes a single job
async fn worker(station_name: &'static str, active: Arc<AtomicUsize>, job: Job) {
    active.fetch_add(1, Ordering::Relaxed);
    tokio::time::sleep(job.duration).await;
    let prepared_item = PreparedItem {
        order_id: job.order_id,
        item: job.item_kind,
    };
    let _ = job.progress_tx.send(ProgressEvent::ItemDone(prepared_item.clone()));
    let _ = job.if_done.send(prepared_item);
    active.fetch_sub(1, Ordering::Relaxed);
    let _ = station_name;
}


// Create a station with a bounded queue and worker pool
pub fn station_channel(name: &'static str, workers: usize, buffer: usize) -> StationHandle {
    let (tx, rx) = mpsc::channel::<Job>(buffer);
    let rx = Arc::new(Mutex::new(rx));
    let active = Arc::new(AtomicUsize::new(0));

    for _ in 0..workers {
        let rx = Arc::clone(&rx);
        let active_clone = Arc::clone(&active);
        spawn(async move {
            loop {
                let job = {
                    let mut rx = rx.lock().await;
                    rx.recv().await
                };
                match job {
                    Some(job) => worker(name, active_clone.clone(), job).await,
                    None => break,
                }
            }
        });
    }

    StationHandle {
        name,
        tx,
        buffer,
        workers,
        active,
    }
}