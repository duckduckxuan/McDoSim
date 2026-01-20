# McDoSim

McDoSim is a terminal-based concurrent kitchen simulation written in Rust.  
It models a fast-food restaurant workflow using Tokio async tasks, bounded queues, worker pools, and an event-driven dashboard.

The project is designed as a **concurrency and architecture demonstration**.

## Features

- Asynchronous order processing with Tokio

- Multiple kitchen stations with independent worker pools
  - Grill station
  - Fryer station
  - Drink station

- Bounded queues with backpressure

- Real-time terminal dashboard
  - Active workers per station
  - Queue occupancy
  - Per-order progress

- Event-driven rendering
  - Dashboard redraws only when data changes
  - Debounce to avoid terminal flicker

- Deterministic simulation via seeded RNG

## Architecture Overview

Each prepared item is an independent job routed to the appropriate station based on its type.

Order Generator  
↓  
process_order  
↓  
Station Queues (mpsc)  
↓  
Workers (Tokio tasks)  
↓  
Progress Events  
↓  
Dashboard (event-driven, debounced)

## Project Structure

src/  
├── main.rs        // Application entry point  
├── model.rs       // Core domain models and enums  
├── random.rs      // Random order generation  
├── station.rs     // Worker pools, stations, job execution  
├── kitchen.rs     // Order dispatch and orchestration  
├── dashboard.rs   // Terminal dashboard rendering  
└── utils.rs       // Small helper functions

## Dashboard Design

The dashboard is **event-driven**, not timer-driven.

Key ideas:

- Progress events trigger state updates
- A dirty flag marks when data changes
- A short debounce window merges multiple updates
- Rendering happens only when needed

This avoids:

- Periodic full redraws
- Terminal flickering
- Wasted CPU cycles

## Concurrency Model

- Each station has:
  - A bounded `mpsc::channel`
  - A fixed number of worker tasks

- Workers pull jobs from the queue and simulate preparation with `tokio::sleep`

- Atomic counters track active workers per station

- Completion is signaled via:
  - `oneshot` channel for order completion
  - `mpsc::UnboundedSender` for dashboard updates

## Determinism

All random generation uses a fixed seed: `generator_orders(orders_n, 42)`

This guarantees:

- Reproducible simulations
- Predictable debugging
- Comparable performance runs

## How to Run

Requirements:

- Tokio
- rand

Run:

```bash
cargo run
```

The terminal will display a live dashboard showing station load and order progress.
Initial orders are also saved to orders.txt for post-run inspection.

## Why This Project Exists

This project demonstrates:

- Practical async Rust
- Correct use of Tokio primitives
- Backpressure handling
- Event-driven UI updates
- Clean modular architecture