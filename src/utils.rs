use crate::model::{ItemKind, Order};


// Count total items in an order
pub fn order_total_items(order: &Order) -> usize {
    order.lines.iter().map(|l| l.quantity as usize).sum()
}


// Map item kind to dashboard column index
pub fn kind_bucket(kind: &ItemKind) -> usize {
    match kind {
        ItemKind::Burger(_) => 0,
        ItemKind::Snack(_) => 1,
        ItemKind::Drink(_) => 2,
    }
}