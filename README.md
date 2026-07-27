# Market Geometry Visualizer

I wanted to have a way to visualize market data from a live Binance order book data ingestion. This has a fixed-presision state engine that converts book levels into geometric coordinates and renders them as evolving point clouds.
This does not interpret or analyze the data, it simply visualizes it.

---

## Overview

The application separates network ingestion, state maintenance, and rendering into distinct execution layers to ensure smooth visualization under high‑frequency updates.  
Incoming order‑book levels are normalized into geometric coordinates (`CloudPoint`) and displayed using a hardware‑accelerated `eframe`/`egui` canvas.

---

## Features

### Asynchronous Order‑Book Ingestion
- REST snapshot (`/api/v3/depth?limit=50`)
- WebSocket incremental deltas (`@depth`)
- `tokio` runtime for non‑blocking network I/O
- `serde` JSON deserialization into structured updates

### Cross‑Thread Decoupling
- `crossbeam_channel` for async → sync boundary
- Dedicated ingestion thread maintains the order‑book state
- UI thread remains responsive under heavy update load

### Fixed‑Precision Price Representation
- Prices stored as `i64` using `price * 1_000_000`
- Prevents floating‑point drift
- Ensures stable ordering and geometric mapping

### Geometric Coordinate Mapping
Each order‑book level becomes a `CloudPoint`:

- **x**: price offset from mid‑price (percentage)
- **y**: normalized size
- **depth**: index within snapshot
- **side**: bid/ask polarity
- **ts**: UNIX timestamp for temporal decay

### Temporal Visual Decay
- Older points fade using alpha scaling
- Recent activity appears bright
- Highlights liquidity motion and structural changes

### Multi‑Symbol Support
- Each symbol maintains its own point cloud
- Ingestion runs concurrently via `tokio::spawn`

---


## Project Structure

- **`main.rs`** — runtime orchestration, GUI startup, ingestion thread spawning  
- **`binance.rs`** — REST snapshot fetcher  
- **`order_book.rs`** — fixed‑precision order‑book engine  
- **`point_cloud.rs`** — geometric mapping into `CloudPoint`  
- **`visualizer.rs`** — real‑time rendering loop  
- **`RawBookMessage`** — WebSocket delta format

---

## Usage

### Prerequisites

Ensure Rust stable toolchain is installed:

```bash
rustc --version
```

### Build

```bash
git clone https://github.com/your/repo
cd market-geometry
cargo build --release
```

### Run

```bash
cargo run --release -- BTCUSDT ETHUSDT
```