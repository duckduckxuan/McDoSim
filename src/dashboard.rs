use std::{io::{self, Write}, pin::Pin, time::{Duration, Instant}};
use tokio::sync::mpsc;
use crate::{kitchen::Kitchen, utils::kind_bucket};
use crate::station::ProgressEvent;


// Render the dashboard to the terminal
pub fn redraw_screen(
    start: Instant,
    kitchen: &Kitchen,
    totals: &Vec<usize>,
    done_total: &Vec<usize>,
    done_kind: &Vec<[usize; 3]>,
    orders_n: u8
) {
    // Move cursor to top-left and clear screen
    print!("\x1B[H\x1B[0J");

    let elapsed = start.elapsed().as_secs_f32();
    println!("Time: {:.1}s\n", elapsed);

    println!("Pipeline");
    println!("Station | active/workers | queue/buffer");
    println!("------- | ------------- | -----------");

    for s in [&kitchen.grill, &kitchen.fryer, &kitchen.drink] {
        println!(
            "{:>7} | {:>2}/{:<11} | {:>3}/{:<6}",
            s.name,
            s.active(),
            s.workers,
            s.queue_len(),
            s.buffer
        );
    }

    println!("\nOrders");
    println!("id | done/total | burgers | snacks | drinks");
    println!("-- | ---------- | ------ | ------ | ------");

    for id in 0u8..orders_n {
        let idx = id as usize;
        let total = totals[idx];
        if total == 0 {
            continue;
        }
        println!(
            "{:>2} | {:>4}/{:<5} | {:>6} | {:>6} | {:>6}",
            id,
            done_total[idx],
            total,
            done_kind[idx][0],
            done_kind[idx][1],
            done_kind[idx][2],
        );
    }

    let _ = io::stdout().flush();
}


pub async fn dashboard_task(
    mut progress_rx: mpsc::UnboundedReceiver<ProgressEvent>,
    kitchen: Kitchen,
    totals: Vec<usize>,
    start: Instant,
    orders_n: u8,
) {
    let mut done_total = vec![0usize; 256];
    let mut done_kind = vec![[0usize; 3]; 256];

    // "dirty flag": data changed since last render
    let mut dirty = true; // render once at startup
    let debounce = Duration::from_millis(120);

    // Debounce timer: coalesce many events into one redraw
    let mut pending: Option<Pin<Box<tokio::time::Sleep>>> = None;

    loop {
        tokio::select! {
            ev = progress_rx.recv() => {
                match ev {
                    Some(ProgressEvent::ItemDone(item)) => {
                        let i = item.order_id as usize;
                        done_total[i] += 1;
                        let b = kind_bucket(&item.item);
                        done_kind[i][b] += 1;

                        dirty = true;

                        // Start debounce if not already running
                        if pending.is_none() {
                            pending = Some(Box::pin(tokio::time::sleep(debounce)));
                        }
                    }
                    None => {
                        // Channel closed; do a final render if needed and exit
                        if dirty {
                            redraw_screen(
                                start,
                                &kitchen,
                                &totals,
                                &done_total,
                                &done_kind,
                                orders_n,
                            );
                        }
                        break;
                    }
                }
            }

            // Debounce fires: redraw once if anything changed
            _ = async {
                if let Some(s) = &mut pending {
                    s.as_mut().await;
                }
            }, if pending.is_some() => {
                if dirty {
                    redraw_screen(
                        start,
                        &kitchen,
                        &totals,
                        &done_total,
                        &done_kind,
                        orders_n,
                    );
                    dirty = false;
                }
                pending = None;
            }
        }
    }

    let _ = io::stdout().flush();
}