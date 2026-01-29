> [!WARNING]
> **USE AT YOUR OWN RISK.** This project is not a "plug-and-play" profit tool.
>
> * **Active Development:** This code is a **Work in Progress (WIP)**. It contains experimental logic and has not been tested thoroughly. Unexpected behavior or crashes could result in losing your entire balance.
> * **Financial Risk:** Automated trading systems involve significant risk, you should assume that **loss of funds is highly likely**.

# Polymarket Reward Farmer

Collects [polymarket liquidity rewards](https://docs.polymarket.com/polymarket-learn/trading/liquidity-rewards) by placing and maintaining orders at a predefined spread on configured markets.
Upon getting filled, the bot will set the spread for the opposite side to zero to attempt to neutralize the position.

## Status

Work in progress. This project is under active development and not ready for use. Heavy testing is required.

## Features

- **Configurable:** Adjust the order size, quoting spread and max exposure per market.
- **Multi-market support:** Quote multiple markets simultaneously.
- **Risk management:** Inventory management per market by adjusting quoting spread depending on max_exposure in config.
- **Panic:** Will attempt to cancel all orders for a market upon failure to place or cancel.

## Quick Start Guide

### 1. Prerequisites
Ensure you have the [Rust toolchain](https://rustup.rs/) installed.
Ensure you have a polymarket account with a funder address.

### 2. Environment Setup
Create a `.env` file in the project root with the following variables:

```bash
PRIVATE_KEY=your_private_key_here
FUNDER_ADDRESS=your_polymarket_address_here
```

### 3. Configuration
Copy `config.example.toml` to `config.toml` and configure your markets:

```bash
cp config.example.toml config.toml
```

Edit `config.toml` to set up your markets:

```toml
[[markets]]
slug = "your-market-slug"
order_size = 5
spread = 0.02
max_exposure = 5
```

### 4. Build and Run
```bash
cargo run --release
```

The bot will prompt you to confirm before starting, as it's still in development.
