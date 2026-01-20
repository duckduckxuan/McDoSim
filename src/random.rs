use rand::{Rng, SeedableRng};
use crate::model::*;


// Random generation of burger
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

// Random generation of snack
fn random_snack(rng: &mut impl Rng) -> Snack {
    match rng.gen_range(0..2) {
        0 => Snack::Fries,
        1 => Snack::Nuggets,
        _ => unreachable!(),
    }
}

// Random generation of drink
fn random_drink(rng: &mut impl Rng) -> Drink {
    match rng.gen_range(0..4) {
        0 => Drink::Cola,
        1 => Drink::Fanta,
        2 => Drink::Sprite,
        3 => Drink::Lipton,
        _ => unreachable!(),
    }
}

// Random generation of size
fn random_size(rng: &mut impl Rng) -> Size {
    match rng.gen_range(0..2) {
        0 => Size::Medium,
        1 => Size::Large,
        _ => unreachable!(),
    }
}

// Random generation of order line
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

// Random generation of order
fn random_order(rng: &mut impl Rng, id: u8) -> Order {
    let num_lines = rng.gen_range(2..7);
    let mut lines = Vec::new();
    for _i in 0..num_lines {
        lines.push(random_order_line(rng));
    }
    Order { id, lines }
}   

// Generate a list of random orders
pub fn generator_orders(n: u8, seed: u64) -> Vec<Order> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut orders = Vec::new();
    for i in 0..n {
        orders.push(random_order(&mut rng, i));
    }
    orders
}