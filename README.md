# Market Geometry Visualizer

I wanted to have a way to visualize market data from a live Binance order book data ingestion. This has a fixed-presision state engine that converts book levels into geometric coordinates and renders them as evolving point clouds.
This does not interpret or analyze the data, it simply visualizes it.

---

## Usage

### Prerequisites

Ensure Rust is installed.

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