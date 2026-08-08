# Market Geometry Visualizer

A real‑time Rust application that ingests live Binance order‑book data and converts book levels into geometric coordinates.  
The system does **not** interpret or predict markets, it simply visualizes the *geometry* of liquidity as evolving point clouds.

This viewer helps you explore whether market microstructure has meaningful geometric or topological patterns.

---

## Features

- **Live Binance order‑book ingestion** (WebSocket + REST snapshot)
- **Fixed‑precision price engine** for stable geometric mapping
- **Real‑time point‑cloud rendering** using `eframe` + `egui`
- **Temporal fading** (newer points appear brighter)
- **Depth‑scaled radius** (larger depth -> larger radius)
- **Bid/Ask color coding** (blue = bid, red = ask)
- **Multi‑symbol support** with tab switching
- **Topological structure (TDA)**  
  - Betti‑0 (connected components)  
  - Betti‑1 (cycles in the 1‑skeleton graph)

---

## Prerequisites

Engusre rust is installed, refer to the rust documentation for installation instructions.

Clone and build:
```bash
git clone https://github.com/your/repo
cd market-geometry
cargo build --release
```
## Usage
```bash
cargo run --release -- BTCUSDT ETHUSDT
```
___

## Interpreting the Visualization
### Geometry

Each order‑book update is mapped into a point:

    x = normalized price offset

    y = normalized depth rank

    radius = depth magnitude

    color = bid (blue) or ask (red)

    alpha = temporal fading (newer = brighter)

This produces a dynamic “liquidity cloud.”
Topology (TDA)

### The viewer computes:

    Betti‑0 -> number of connected components

    Betti‑1 -> number of cycles in the 1‑skeleton graph

### These values quantify the structure of the geometry:

    High Betti‑0 -> fragmentation

    High Betti‑1 -> dense, loopy structure

    Low Betti‑0 -> unified liquidity blob

    Low Betti‑1 -> sparse or simple geometry

### Limitations

    This is not a trading tool.

    This is not a predictive model.

    This is not a regime classifier.

    It is purely a visualizer of geometric and topological structure.

    The TDA is still a work in progress, which requires polishing.