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

# CEX WebSocket arbitrage stream
cargo run -- scan-arb-ws

# DEX pool price listener (Uniswap V2/V3 style). Requires POOL_LISTENER_RPC_WS in .env
cargo run -- pool-listener-v2
cargo run -- pool-listener-v3
```

## DEX pool listener

Stream live prices from a single Uniswap V2 or V3 style pool over WebSocket RPC.

- Set **`POOL_LISTENER_RPC_WS`** in `.env` (e.g. `wss://...` for BSC).
- Default example uses BSC chain 56, a V2 pool; config is in code (`pool-listener` command).
- **ListenMode**: `EveryBlock` (each new block) or `OnSwapEvent` (only on Swap events).
- **PriceDirection**: `Token1PerToken0` (e.g. USDT per BNB) or `Token0PerToken1`.
- **Reconnect**: `reconnect_attempts = 0` to disable; `reconnect_delay_ms` between attempts.

## Amounts

- `scan-dex ... [quote_amount]`: **quote_amount is in the quote token units** (here: **USDT on BSC**).
  - Example: `1000` means “route/simulate using ~1000 USDT”.
- The printed `qty=...` in results is the **estimated executable base quantity** for that opportunity.
