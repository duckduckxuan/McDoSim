use std::time::Duration;


// Different kinds of burgers
#[derive(Debug, Clone)]
pub enum Burger {
    BigMac,
    Cheeseburger,
    McFish,
    McCrispy,
    McWrap,
}

// Different kinds of snacks
#[derive(Debug, Clone)]
pub enum Snack {
    Fries,
    Nuggets,
}

// Different kinds of drinks
#[derive(Debug, Clone)]
pub enum Drink {
    Cola,
    Fanta,
    Sprite,
    Lipton,
}

// Size of the item, affects preparation time
#[derive(Debug)]
pub enum Size {
    Medium,
    Large,
}

// Unified item type used throughout the pipeline
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ItemKind {
    Burger(Burger),
    Snack(Snack),
    Drink(Drink),
}

// Logical kitchen stations
#[derive(Debug, Clone)]
pub enum Station {
    Grill,
    Fryer,
    Drink,
}

// Represents a line in an order
#[derive(Debug)]
pub struct OrderLine {
    pub item: ItemKind,
    pub quantity: u8,
    pub size: Size,
}

// Represents a customer order consisting of multiple lines
#[derive(Debug)]
pub struct Order {
    pub id: u8,
    pub lines: Vec<OrderLine>,
}

// Finished item produced by a station
#[derive(Debug, Clone)]
pub struct PreparedItem {
    pub order_id: u8,
    pub item: ItemKind,
}


impl ItemKind { 
    // Base preparation time depending on item type
    pub fn base_time(&self) -> Duration {
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

    // Determine which station handles this item
    pub fn station(&self) -> Station {
        match self {
            ItemKind::Burger(_) => Station::Grill,
            ItemKind::Snack(_) => Station::Fryer,
            ItemKind::Drink(_) => Station::Drink,
        }
    }
}


impl OrderLine {
    // Total preparation time for this line
    pub fn prep_time(&self) -> Duration {
        let base_time = self.item.base_time();
        match self.size {
            Size::Medium => base_time,
            Size::Large => base_time + Duration::from_secs(1),
        }
    }
}