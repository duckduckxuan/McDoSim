use tokio::{spawn, sync::{Mutex, mpsc, oneshot}};
use std::{io::{self, Write}, pin::Pin, sync::{atomic::{AtomicUsize, Ordering}, Arc}, time::{Duration, Instant}};
use rand::{Rng, SeedableRng};


#[derive(Debug, Clone)]
enum Burger {
    BigMac,
    Cheeseburger,
    McFish,
    McCrispy,
    McWrap,
}

#[derive(Debug, Clone)]
enum Snack {
    Fries,
    Nuggets,
}

#[derive(Debug, Clone)]
enum Drink {
    Cola,
    Fanta,
    Sprite,
    Lipton,
}

#[derive(Debug)]
enum Size {
    Medium,
    Large,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ItemKind {
    Burger(Burger),
    Snack(Snack),
    Drink(Drink),
}

#[derive(Debug, Clone)]
enum Station {
    Grill,
    Fryer,
    Drink,
}

#[derive(Debug, Clone)]
enum ProgressEvent {
    ItemDone(PreparedItem),
}

#[derive(Debug)]
struct OrderLine {
    item: ItemKind,
    quantity: u8,
    size: Size,
}

#[derive(Debug)]
struct Order {
    id: u8,
    lines: Vec<OrderLine>,
}

#[derive(Debug, Clone)]
struct PreparedItem {
    order_id: u8,
    item: ItemKind,
}

#[derive(Debug)]
struct Job {
    order_id: u8,
    item_kind: ItemKind,
    duration: Duration,
    if_done: oneshot::Sender<PreparedItem>,
    progress_tx: mpsc::UnboundedSender<ProgressEvent>,
}

#[derive(Debug, Clone)]
struct Kitchen {
    grill: StationHandle,
    fryer: StationHandle,
    drink: StationHandle,
}

#[derive(Debug, Clone)]
struct StationHandle {
    name: &'static str,
    tx: mpsc::Sender<Job>,
    buffer: usize,
    workers: usize,
    active: Arc<AtomicUsize>,
}


impl ItemKind {
    fn base_time(&self) -> Duration {
        match self {
            ItemKind::Burger(burger) => match burger {
                Burger::BigMac => Duration::from_secs(7),
                Burger::Cheeseburger => Duration::from_secs(5),
                Burger::McFish => Duration::from_secs(6),
                Burger::McCrispy => Duration::from_secs(8),
                Burger::McWrap => Duration::from_secs(4),
            }
            ItemKind::Snack(snack) => match snack {
                Snack::Fries => Duration::from_secs(3),
                Snack::Nuggets => Duration::from_secs(4),
            }
            ItemKind::Drink(_) => Duration::from_secs(1),
        }
    }

    fn station(&self) -> Station {
        match self {
            ItemKind::Burger(_) => Station::Grill,
            ItemKind::Snack(_) => Station::Fryer,
            ItemKind::Drink(_) => Station::Drink,
        }
    }
}


impl OrderLine {
    fn prep_time(&self) -> Duration {
        let base_time = self.item.base_time();
        match self.size {
            Size::Medium => base_time,
            Size::Large => base_time + Duration::from_secs(1),
        }
    }
}


impl StationHandle {
    fn queue_len(&self) -> usize {
        self.buffer.saturating_sub(self.tx.capacity())
    }

    fn active(&self) -> usize {
        self.active.load(Ordering::Relaxed)
    }
}



fn random_burger(rng: &mut impl Rng) -> Burger {
    match rng.gen_range(0..5) as usize {
        0 => Burger::BigMac,
        1 => Burger::Cheeseburger,
        2 => Burger::McFish,
        3 => Burger::McCrispy,
        4 => Burger::McWrap,
        _ => unreachable!(),
    }
}

fn random_snack(rng: &mut impl Rng) -> Snack {
    match rng.gen_range(0..2) {
        0 => Snack::Fries,
        1 => Snack::Nuggets,
        _ => unreachable!(),
    }
}

fn random_drink(rng: &mut impl Rng) -> Drink {
    match rng.gen_range(0..4) {
        0 => Drink::Cola,
        1 => Drink::Fanta,
        2 => Drink::Sprite,
        3 => Drink::Lipton,
        _ => unreachable!(),
    }
}

fn random_size(rng: &mut impl Rng) -> Size {
    match rng.gen_range(0..2) {
        0 => Size::Medium,
        1 => Size::Large,
        _ => unreachable!(),
    }
}

fn random_order_line(rng: &mut impl Rng) -> OrderLine {
    let item_type = rng.gen_range(0..3);
    let quantity = rng.gen_range(1..3);
    match item_type {
        0 => OrderLine {
            item: ItemKind::Burger(random_burger(rng)),
            quantity,
            size: random_size(rng),
        },
        1 => OrderLine {
            item: ItemKind::Snack(random_snack(rng)),
            quantity,
            size: random_size(rng),
        },
        2 => OrderLine {
            item: ItemKind::Drink(random_drink(rng)),
            quantity,
            size: random_size(rng),
        },
        _ => unreachable!(),
    }
}

fn random_order(rng: &mut impl Rng, id: u8) -> Order {
    let num_lines = rng.gen_range(2..7);
    let mut lines = Vec::new();
    for _i in 0..num_lines {
        lines.push(random_order_line(rng));
    }
    Order { id, lines }
}   

fn generator_orders(n: u8, seed: u64) -> Vec<Order> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut orders = Vec::new();
    for i in 0..n {
        orders.push(random_order(&mut rng, i));
    }
    orders
}



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


fn station_channel(name: &'static str, workers: usize, buffer: usize) -> StationHandle {
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


async fn process_order(kitchen: Kitchen, order: Order, progress_tx: mpsc::UnboundedSender<ProgressEvent>) -> Vec<PreparedItem> {
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


fn order_total_items(order: &Order) -> usize {
    order.lines.iter().map(|l| l.quantity as usize).sum()
}


fn kind_bucket(kind: &ItemKind) -> usize {
    match kind {
        ItemKind::Burger(_) => 0,
        ItemKind::Snack(_) => 1,
        ItemKind::Drink(_) => 2,
    }
}


fn redraw_screen(
    start: Instant,
    kitchen: &Kitchen,
    totals: &Vec<usize>,
    done_total: &Vec<usize>,
    done_kind: &Vec<[usize; 3]>,
    orders_n: u8,
) {
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



#[tokio::main]
async fn main() {
    let orders_n = 10;
    let orders = generator_orders(orders_n, 42);

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


    let mut totals = vec![0usize; 256];
    for o in &orders {
        totals[o.id as usize] = order_total_items(o);
    }

    let kitchen = Kitchen {
        grill: station_channel("Grill", 3, 10),
        fryer: station_channel("Fryer", 2, 10),
        drink: station_channel("Drink", 2, 10),
    };

    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<ProgressEvent>();
    let start = Instant::now();

    let kitchen_for_dash = kitchen.clone();
    let totals_for_dash = totals.clone();

    tokio::spawn(async move {
        let mut done_total = vec![0usize; 256];
        let mut done_kind = vec![[0usize; 3]; 256];

        let mut update = true;
        let debounce = Duration::from_millis(120);

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

                            update = true;

                            if pending.is_none() {
                                pending = Some(Box::pin(tokio::time::sleep(debounce)));
                            }
                        }
                        None => {
                            if update {
                                redraw_screen(
                                    start,
                                    &kitchen_for_dash,
                                    &totals_for_dash,
                                    &done_total,
                                    &done_kind,
                                    orders_n
                                );
                            }
                            break;
                        }
                    }
                }
                _ = async {
                    if let Some(s) = &mut pending {
                        s.as_mut().await;
                    }
                }, if pending.is_some() => {
                    if update {
                        redraw_screen(
                            start,
                            &kitchen_for_dash,
                            &totals_for_dash,
                            &done_total,
                            &done_kind,
                            orders_n
                        );
                        update = false;
                    }
                    pending = None;
                }
            }
        }
    });

    let mut set = tokio::task::JoinSet::new();
    for order in orders {
        let kitchen = kitchen.clone();
        let progress_tx = progress_tx.clone();
        set.spawn(async move {
            process_order(kitchen, order, progress_tx).await;
        });
    }
    drop(progress_tx);

    while let Some(res) = set.join_next().await {
        res.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(500)).await;
}