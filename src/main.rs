use aeon_market_scanner_rs::dex::chains::{ChainId, Token};
use aeon_market_scanner_rs::{
    ArbitrageScanner, Binance, Bitfinex, Bitget, Btcturk, Bybit, CEXTrait, CexExchange, Coinbase,
    Cryptocom, DexAggregator, FeeOverrides, Gateio, Htx, Kraken, Kucoin, MarketScannerError, Mexc,
    Upbit, OKX,
};

fn print_help() {
    eprintln!(
        r#"aeon-market-scanner-example

Usage:
  cargo run -- price <SYMBOL>
  cargo run -- stream [EXCHANGE] <SYMBOL> [SYMBOL...]
  cargo run -- scan-cex <SYMBOL> <EXCHANGE> [EXCHANGE...]
  cargo run -- scan-dex <EXCHANGE> [EXCHANGE...] [quote_amount]
  cargo run -- scan-cex-overrides

Examples:
  cargo run -- price BTCUSDT
  cargo run -- stream binance BTCUSDT ETHUSDT
  cargo run -- scan-cex BTCUSDT binance bybit
  cargo run -- scan-dex binance bybit 1000
  cargo run -- scan-dex binance bybit 25000
  cargo run -- scan-cex-overrides

Notes:
  scan-dex defaults to BTC/USDT (symbol=BTCUSDT, DEX=KyberSwap on BSC using BTCB/USDT).
  scan-dex quote_amount is in quote token units (here: USDT on BSC by default).

Exchanges:
  binance, bybit, mexc, okx, gateio, kucoin, bitget, btcturk, htx,
  coinbase, kraken, bitfinex, upbit, cryptocom
"#
    );
}

fn parse_cex_exchange(s: &str) -> Option<CexExchange> {
    match s.trim().to_ascii_lowercase().as_str() {
        "binance" => Some(CexExchange::Binance),
        "bybit" => Some(CexExchange::Bybit),
        "mexc" => Some(CexExchange::MEXC),
        "okx" => Some(CexExchange::OKX),
        "gateio" | "gate" => Some(CexExchange::Gateio),
        "kucoin" => Some(CexExchange::Kucoin),
        "bitget" => Some(CexExchange::Bitget),
        "btcturk" | "btc-turk" => Some(CexExchange::Btcturk),
        "htx" | "huobi" => Some(CexExchange::Htx),
        "coinbase" => Some(CexExchange::Coinbase),
        "kraken" => Some(CexExchange::Kraken),
        "bitfinex" => Some(CexExchange::Bitfinex),
        "upbit" => Some(CexExchange::Upbit),
        "cryptocom" | "crypto.com" | "crypto" => Some(CexExchange::Cryptocom),
        _ => None,
    }
}

async fn stream_exchange<E: CEXTrait>(
    exchange: E,
    symbols: Vec<String>,
) -> Result<(), MarketScannerError> {
    if !exchange.supports_websocket() {
        eprintln!("WebSocket streaming is not supported for this exchange.");
        return Ok(());
    }

    println!("Streaming from {}", exchange.exchange_name());

    let symbol_refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
    let mut rx = exchange
        .stream_price_websocket(&symbol_refs, true, Some(10))
        .await?;

    while let Some(update) = rx.recv().await {
        println!(
            "[{:?}] {} bid={} (qty={}) ask={} (qty={})",
            update.exchange,
            update.symbol,
            update.bid_price,
            update.bid_qty,
            update.ask_price,
            update.ask_qty
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), MarketScannerError> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        print_help();
        return Ok(());
    };

    match cmd.as_str() {
        "price" => {
            let Some(symbol) = args.next() else {
                print_help();
                return Ok(());
            };

            let price = Binance::new().get_price(&symbol).await?;
            println!(
                "{} bid={} (qty={}) ask={} (qty={}) mid={}",
                price.symbol,
                price.bid_price,
                price.bid_qty,
                price.ask_price,
                price.ask_qty,
                price.mid_price
            );
            Ok(())
        }

        "stream" => {
            // stream [EXCHANGE] <SYMBOL> [SYMBOL...]
            // If EXCHANGE is omitted, defaults to Binance.
            let rest: Vec<String> = args.collect();
            if rest.is_empty() {
                print_help();
                return Ok(());
            }

            let (exchange, symbols) = if rest.len() >= 2 {
                match parse_cex_exchange(&rest[0]) {
                    Some(ex) => (ex, rest[1..].to_vec()),
                    None => (CexExchange::Binance, rest),
                }
            } else {
                (CexExchange::Binance, rest)
            };

            match exchange {
                CexExchange::Binance => stream_exchange(Binance::new(), symbols).await,
                CexExchange::Bybit => stream_exchange(Bybit::new(), symbols).await,
                CexExchange::MEXC => stream_exchange(Mexc::new(), symbols).await,
                CexExchange::OKX => stream_exchange(OKX::new(), symbols).await,
                CexExchange::Gateio => stream_exchange(Gateio::new(), symbols).await,
                CexExchange::Kucoin => stream_exchange(Kucoin::new(), symbols).await,
                CexExchange::Bitget => stream_exchange(Bitget::new(), symbols).await,
                CexExchange::Btcturk => stream_exchange(Btcturk::new(), symbols).await,
                CexExchange::Htx => stream_exchange(Htx::new(), symbols).await,
                CexExchange::Coinbase => stream_exchange(Coinbase::new(), symbols).await,
                CexExchange::Kraken => stream_exchange(Kraken::new(), symbols).await,
                CexExchange::Bitfinex => stream_exchange(Bitfinex::new(), symbols).await,
                CexExchange::Upbit => stream_exchange(Upbit::new(), symbols).await,
                CexExchange::Cryptocom => stream_exchange(Cryptocom::new(), symbols).await,
            }
        }

        "scan-cex" => {
            let Some(symbol) = args.next() else {
                print_help();
                return Ok(());
            };

            let mut exchanges: Vec<CexExchange> = Vec::new();
            for s in args {
                match parse_cex_exchange(&s) {
                    Some(ex) => exchanges.push(ex),
                    None => {
                        eprintln!("Unknown exchange: {s}");
                        print_help();
                        return Ok(());
                    }
                }
            }

            if exchanges.len() < 2 {
                eprintln!("scan-cex needs at least 2 exchanges.");
                print_help();
                return Ok(());
            }

            let opportunities = ArbitrageScanner::scan_arbitrage_opportunities(
                &symbol, &exchanges, None, None, None, None, None,
            )
            .await?;

            println!("Found {} opportunities", opportunities.len());
            if opportunities.is_empty() {
                return Ok(());
            }

            for opp in opportunities.iter().take(10) {
                println!(
                    "{} -> {} {} spread={:.6} ({:.3}%) qty={:.8}",
                    opp.source_exchange,
                    opp.destination_exchange,
                    opp.symbol,
                    opp.spread,
                    opp.spread_percentage,
                    opp.executable_quantity
                );
            }
            Ok(())
        }

        "scan-dex" => {
            // scan-dex <EXCHANGE...> [quote_amount]
            // Defaults: symbol=BTCUSDT, DEX pair=BTCB/USDT on BSC (KyberSwap)
            let symbol = "BTCUSDT".to_string();
            // If the last argument parses as f64, treat it as quote_amount; otherwise default 1000.0
            let rest: Vec<String> = args.collect();
            if rest.is_empty() {
                print_help();
                return Ok(());
            }

            let (exchange_args, quote_amount) =
                match rest.last().and_then(|s| s.parse::<f64>().ok()) {
                    Some(amount) if rest.len() >= 2 => (&rest[..rest.len() - 1], amount),
                    _ => (&rest[..], 1000.0),
                };

            let mut exchanges: Vec<CexExchange> = Vec::new();
            for s in exchange_args {
                match parse_cex_exchange(s) {
                    Some(ex) => exchanges.push(ex),
                    None => {
                        eprintln!("Unknown exchange: {s}");
                        print_help();
                        return Ok(());
                    }
                }
            }
            if exchanges.is_empty() {
                eprintln!("scan-dex needs at least 1 CEX exchange.");
                print_help();
                return Ok(());
            }

            // Default: BNB Chain (BSC) mainnet BTCB/USDT (for BTCUSDT)
            let btcb = Token::create(
                "0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c",
                "Binance BTC",
                "BTCB",
                18,
                ChainId::BSC,
            );
            let usdt = Token::create(
                "0x55d398326f99059fF775485246999027B3197955",
                "Tether USD",
                "USDT",
                18,
                ChainId::BSC,
            );

            println!(
                "DEX config: symbol={} chain=BSC base={}({}) quote={}({}) quote_amount={}",
                symbol, btcb.symbol, btcb.address, usdt.symbol, usdt.address, quote_amount
            );

            let opportunities = ArbitrageScanner::scan_arbitrage_opportunities(
                &symbol,
                &exchanges,
                Some(&[DexAggregator::KyberSwap]),
                Some(&btcb),
                Some(&usdt),
                Some(quote_amount),
                None,
            )
            .await?;

            println!("Found {} opportunities", opportunities.len());
            for opp in opportunities.iter().take(10) {
                println!(
                    "{} -> {} {} spread={:.6} ({:.3}%) qty={:.8}",
                    opp.source_exchange,
                    opp.destination_exchange,
                    opp.symbol,
                    opp.spread,
                    opp.spread_percentage,
                    opp.executable_quantity
                );
            }
            Ok(())
        }

        "scan-cex-overrides" => {
            let overrides = FeeOverrides::default()
                .with_cex_taker_fee(CexExchange::Binance, 0.00075) // 0.075%
                .with_cex_taker_fee(CexExchange::OKX, 0.0008); // 0.08%

            let opportunities = ArbitrageScanner::scan_arbitrage_opportunities(
                "BTCUSDT",
                &[CexExchange::Binance, CexExchange::OKX],
                None,
                None,
                None,
                None,
                Some(&overrides),
            )
            .await?;

            println!("Found {} opportunities", opportunities.len());
            for opp in opportunities.iter().take(10) {
                println!(
                    "{} -> {} {} spread={:.6} ({:.3}%) qty={:.8}",
                    opp.source_exchange,
                    opp.destination_exchange,
                    opp.symbol,
                    opp.spread,
                    opp.spread_percentage,
                    opp.executable_quantity
                );
            }

            Ok(())
        }

        "-h" | "--help" | "help" => {
            print_help();
            Ok(())
        }

        _ => {
            print_help();
            Ok(())
        }
    }
}
