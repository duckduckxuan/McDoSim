use tokio::{spawn, sync::{mpsc, oneshot, Mutex}};
use std::{time::Duration, sync::Arc};
use rand::{Rng, SeedableRng};

#[derive(Debug)]
enum Burger {
    BigMac,
    Cheeseburger,
    McFish,
    McCrispy,
    McWrap,
}

#[derive(Debug)]
enum Snack {
    Fries,
    Nuggets,
}

#[derive(Debug)]
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

#[derive(Debug)]
enum ItemKind {
    Burger(Burger),
    Snack(Snack),
    Drink(Drink),
}

#[derive(Debug)]
enum Station {
    Grill,
    Fryer,
    Drink,
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

#[derive(Debug)]
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
}

#[derive(Debug, Clone)]
struct Kitchen {
    grill_tx: mpsc::Sender<Job>,
    fryer_tx: mpsc::Sender<Job>,
    drink_tx: mpsc::Sender<Job>,
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
    for i in 0..num_lines {
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

async fn worker(name: &'static str, job: Job) {
    println!("{} started job for order {}", name, job.order_id);
    tokio::time::sleep(job.duration).await;
    let prepared_item = PreparedItem {
        order_id: job.order_id,
        item: job.item_kind,
    };
    let _ = job.if_done.send(prepared_item);
    println!("{} completed job for order {}", name, job.order_id);
}

fn station_channel(name: &'static str, workers: usize, buffer: usize) -> mpsc::Sender<Job> {
    let (tx, rx) = mpsc::channel(buffer);
    let rx = Arc::new(Mutex::new(rx));

    for _ in 0..workers {
        let rx = Arc::clone(&rx);
        spawn(async move {
            loop {
                let job = {
                    let mut rx = rx.lock().await;
                    rx.recv().await
                };
                match job {
                    Some(job) => worker(name, job).await,
                    None => break,
                }
            }
        });
    }
    tx
}

fn main() {
    let orders = generator_orders(10, 42);
    for order in orders {
        println!("Order ID: {}", order.id);
        for line in order.lines {
            match line.item {
                ItemKind::Burger(burger) => {
                    println!("  Burger: {:?}, Quantity: {}, Size: {:?}", burger, line.quantity, line.size);
                }
                ItemKind::Snack(snack) => {
                    println!("  Snack: {:?}, Quantity: {}, Size: {:?}", snack, line.quantity, line.size);
                }
                ItemKind::Drink(drink) => {
                    println!("  Drink: {:?}, Quantity: {}, Size: {:?}", drink, line.quantity, line.size);
                }
            }
        }
    }
}