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
    Salad,
}

#[derive(Debug)]
enum Drink {
    Cola,
    Fanta,
    Sprite,
    Lipton,
}

enum Size {
    Medium,
    Large,
}

enum ItemKind {
    Burger(Burger),
    Snack(Snack),
    Drink(Drink),
}

struct OrderLine {
    item: ItemKind,
    quantity: u8,
    size: Option<Size>,
}

struct Order {
    id: u8,
    lines: Vec<OrderLine>,
}

struct PreparedItem {
    order_id: u8,
    item: ItemKind,
}

use rand::{Rng, SeedableRng};

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
    match rng.gen_range(0..3) {
        0 => Snack::Fries,
        1 => Snack::Nuggets,
        2 => Snack::Salad,
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
    let quantity = rng.gen_range(1..5);
    match item_type {
        0 => OrderLine {
            item: ItemKind::Burger(random_burger(rng)),
            quantity,
            size: None,
        },
        1 => OrderLine {
            item: ItemKind::Snack(random_snack(rng)),
            quantity,
            size: None,
        },
        2 => OrderLine {
            item: ItemKind::Drink(random_drink(rng)),
            quantity,
            size: Some(random_size(rng)),
        },
        _ => unreachable!(),
    }
}

fn random_order(rng: &mut impl Rng, id: u8) -> Order {
    let num_lines = rng.gen_range(1..5);
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

fn main() {
    let orders = generator_orders(10, 42);
    for order in orders {
        println!("Order ID: {}", order.id);
        for line in order.lines {
            match line.item {
                ItemKind::Burger(burger) => {
                    println!("  Burger: {:?}, Quantity: {}", burger, line.quantity);
                }
                ItemKind::Snack(snack) => {
                    println!("  Snack: {:?}, Quantity: {}", snack, line.quantity);
                }
                ItemKind::Drink(drink) => {
                    println!("  Drink: {:?}, Quantity: {}", drink, line.quantity);
                }
            }
        }
    }
}