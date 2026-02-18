# aeon-market-scanner-example

Example project for the `aeon-market-scanner-rs` crate.

## Run

```bash
cargo run -- price BTCUSDT
# Prints bid/ask prices + bid_qty/ask_qty (amounts)
cargo run -- stream binance BTCUSDT ETHUSDT
cargo run -- scan-cex BTCUSDT binance bybit
cargo run -- scan-cex-overrides

# DEX leg uses KyberSwap on BNB Chain (BSC) by default (BTCB/USDT)
cargo run -- scan-dex binance bybit mexc okx gateio kucoin bitget btcturk htx coinbase kraken bitfinex upbit cryptocom 1000
cargo run -- scan-dex binance bybit mexc okx gateio kucoin bitget btcturk htx coinbase kraken bitfinex upbit cryptocom 25000

# Cex ws arb
cargo run -- scan-arb-ws
```

## Amounts

- `scan-dex ... [quote_amount]`: **quote_amount is in the quote token units** (here: **USDT on BSC**).
  - Example: `1000` means “route/simulate using ~1000 USDT”.
- The printed `qty=...` in results is the **estimated executable base quantity** for that opportunity.
