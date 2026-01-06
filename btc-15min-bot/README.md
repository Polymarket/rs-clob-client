# Bitcoin 15min Trading Bot for Polymarket

A fully automated, production-grade trading bot for Polymarket's "Bitcoin 15min Up or Down" markets. Built in Rust using the official [rs-clob-client](https://github.com/Polymarket/rs-clob-client).

## 🎯 Strategy Overview

The bot implements a **volume-based momentum strategy** with **trailing take profit**:

1. **Market Discovery**: Automatically finds the next upcoming Bitcoin 15min market
2. **Volume Analysis**: 60 seconds before market start, analyzes recent UP vs DOWN trading volume
3. **Entry Decision**: Enters position on the side with higher volume (momentum following)
4. **Trailing Take Profit**: Dynamically tracks best price and exits on retracement
5. **Continuous Trading**: Automatically moves to next market and repeats

### Key Features

✅ **Fully Automated** - Runs 24/7 without manual intervention
✅ **Dynamic Market Discovery** - Never hardcodes market IDs
✅ **Risk Management** - Trailing stops, max hold time, slippage protection
✅ **Production Ready** - Error handling, retry logic, logging
✅ **Configurable** - All parameters adjustable via config file
✅ **Dry Run Mode** - Test without risking real funds

## 📋 Requirements

### System Requirements

- **Rust**: 1.88 or newer
- **Operating System**: Linux, macOS, or Windows
- **Network**: Stable internet connection for 24/7 operation

### Account Requirements

- **Polymarket Account** with funded wallet
- **USDC Balance** on Polygon network
- **Token Approvals** set for USDC and Conditional Tokens (CTF)

## 🚀 Quick Start

### 1. Installation

```bash
# Clone the repository
cd rs-clob-client/btc-15min-bot

# Build the bot
cargo build --release
```

### 2. Configuration

```bash
# Copy example configuration
cp config.toml.example config.toml
cp .env.example .env

# Edit configuration files
nano config.toml  # Set trading parameters
nano .env         # Set your PRIVATE_KEY
```

**IMPORTANT**: Never commit your `config.toml` or `.env` files with real credentials!

### 3. Set Token Approvals

Before trading, you must approve Polymarket contracts to spend your tokens:

```bash
# Run the approval example from parent directory
cd ..
cargo run --example approvals --features bridge

# Follow the prompts to approve:
# 1. USDC spending
# 2. Conditional Token Framework (CTF) spending
```

### 4. Test in Dry Run Mode

```bash
# Enable dry run mode in config.toml
[operational]
dry_run = true

# Run the bot
./target/release/btc-15min-bot
```

The bot will simulate all actions without placing real orders.

### 5. Live Trading

```bash
# Disable dry run mode in config.toml
[operational]
dry_run = false

# Run the bot
./target/release/btc-15min-bot

# Optional: Run in background with nohup
nohup ./target/release/btc-15min-bot > bot.log 2>&1 &
```

## ⚙️ Configuration Guide

### Blockchain Settings

```toml
[blockchain]
private_key = ""  # Leave empty, set via PRIVATE_KEY env var
clob_endpoint = "https://clob.polymarket.com"
chain_id = 137  # Polygon Mainnet
```

### Strategy Parameters

```toml
[strategy]
market_query = "Bitcoin 15min"  # Search query for markets
entry_timing_seconds = 60       # Analyze 60s before start
trade_size_usdc = "5.00"        # $5 per trade
min_volume_differential = "0.10" # Require 10% volume difference
volume_lookback_seconds = 3600  # Analyze last 1 hour of trades
```

**Strategy Tuning**:
- Increase `entry_timing_seconds` (e.g., 120) for more data but less fresh
- Increase `min_volume_differential` (e.g., 0.20) for higher conviction trades
- Adjust `volume_lookback_seconds` to balance recency vs. sample size

### Risk Management

```toml
[risk]
trailing_stop_percentage = "0.05"  # 5% trailing stop
max_position_hold_seconds = 900    # 15 min max hold
max_slippage = "0.02"              # 2% slippage tolerance
min_profit_target = "0.02"         # 2% profit to activate trailing
```

**Risk Tuning**:
- **Tighter trailing stop** (0.03 = 3%): Locks in profits faster but may exit too early
- **Wider trailing stop** (0.08 = 8%): Allows more room for volatility but risks giving back gains
- **Higher min_profit_target** (0.05 = 5%): Only trail after significant profit

### Operational Settings

```toml
[operational]
price_poll_interval_seconds = 3      # Check price every 3 seconds
market_refresh_interval_seconds = 300 # Search for markets every 5 min
retry_attempts = 3                   # Retry failed API calls 3 times
retry_delay_ms = 1000                # Wait 1s between retries
dry_run = false                      # Set to true for testing
```

## 📊 How It Works

### State Machine

The bot operates as a state machine with 5 states:

```
Discovery → Waiting → Analyzing → InPosition → Discovery (repeat)
                                       ↓
                                    Error (recovery)
```

1. **Discovery**: Find next upcoming Bitcoin 15min market
2. **Waiting**: Sleep until entry timing (60s before start)
3. **Analyzing**: Fetch trade volume, calculate UP vs DOWN, generate signal
4. **InPosition**: Monitor price, update trailing stop, exit on trigger
5. **Error**: Handle failures, safely exit positions, retry

### Volume Analysis

The strategy analyzes recent trading activity:

```rust
UP Volume = Sum of all UP token trades (in USDC)
DOWN Volume = Sum of all DOWN token trades (in USDC)

Volume Differential = |UP - DOWN| / (UP + DOWN)

Signal:
- If differential < 10%: Skip (no clear trend)
- If UP > DOWN: BUY UP
- If DOWN > UP: BUY DOWN
```

### Trailing Take Profit

The trailing stop dynamically adjusts to lock in profits:

```
For UP position:
1. Track highest price seen
2. If price drops 5% from high → SELL

For DOWN position:
1. Track lowest price seen
2. If price rises 5% from low → SELL
```

**Important**: Trailing only activates after reaching minimum profit target (2% by default).

## 🔧 Advanced Usage

### Custom Logging

```bash
# Debug logging
RUST_LOG=debug ./target/release/btc-15min-bot

# Trace logging (very verbose)
RUST_LOG=trace ./target/release/btc-15min-bot

# Module-specific logging
RUST_LOG=btc_15min_bot::strategy=debug ./target/release/btc-15min-bot
```

### Running as a Service (Linux)

Create a systemd service file `/etc/systemd/system/btc-bot.service`:

```ini
[Unit]
Description=Bitcoin 15min Trading Bot
After=network.target

[Service]
Type=simple
User=your_username
WorkingDirectory=/path/to/btc-15min-bot
ExecStart=/path/to/btc-15min-bot/target/release/btc-15min-bot
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable btc-bot
sudo systemctl start btc-bot
sudo systemctl status btc-bot
```

View logs:

```bash
sudo journalctl -u btc-bot -f
```

### Monitoring

Key metrics to monitor:

- **Position Entry Rate**: How often the bot enters trades
- **Win Rate**: Percentage of profitable trades
- **Average Profit**: Average P&L per trade
- **Max Drawdown**: Largest loss from peak
- **Volume Differential**: Are signals strong enough?

The bot logs all trading activity. Parse logs to extract performance metrics.

## 🏗️ Architecture

### Project Structure

```
btc-15min-bot/
├── src/
│   ├── main.rs           # Main bot loop and orchestration
│   ├── config.rs         # Configuration management
│   ├── market.rs         # Market discovery and filtering
│   ├── strategy.rs       # Volume analysis and signal generation
│   ├── trader.rs         # Order execution and position tracking
│   ├── trailing_stop.rs  # Trailing take profit logic
│   └── utils.rs          # Helper functions
├── Cargo.toml
├── config.toml.example
├── .env.example
└── README.md
```

### Component Overview

- **Config**: Loads/validates TOML config and environment variables
- **MarketDiscovery**: Queries Polymarket API for Bitcoin 15min markets
- **VolumeStrategy**: Analyzes trade history to generate signals
- **Trader**: Executes market orders, tracks positions
- **TrailingStop**: Manages dynamic exit logic
- **Utils**: Retry logic, formatting, time utilities

### Data Flow

```
Market API → MarketDiscovery → Next Market
                                    ↓
Trade API → VolumeStrategy → Trading Signal
                                    ↓
Signal + Market → Trader → Position Opened
                                    ↓
Price API → TrailingStop → Exit Signal
                                    ↓
Exit Signal → Trader → Position Closed
```

## 🛡️ Safety & Risk Warnings

⚠️ **TRADING RISKS**

- Prediction markets are highly volatile
- Past performance does not guarantee future results
- Volume analysis may not predict price direction
- Network delays can cause slippage
- Smart contract risks exist

⚠️ **OPERATIONAL RISKS**

- Bot runs 24/7 and will trade automatically
- Ensure sufficient USDC balance at all times
- Monitor for API outages or network issues
- Private key compromise = total loss of funds

⚠️ **RECOMMENDED SAFEGUARDS**

- Start with small trade sizes ($5-$10)
- Run in dry run mode first for 24 hours
- Monitor closely during first week
- Set up alerts for errors/unexpected behavior
- Keep private keys in secure environment (hardware wallet, KMS)
- Never share private keys or commit to version control

## 🐛 Troubleshooting

### "Failed to authenticate with Polymarket"

- Check that PRIVATE_KEY is set correctly in `.env`
- Verify private key has no `0x` prefix and is 64 hex characters
- Ensure wallet has been used on Polymarket before

### "No upcoming markets found"

- Bitcoin 15min markets may not always be available
- Check Polymarket website to verify markets exist
- Adjust `market_query` if market naming changes

### "Failed to fetch trades"

- API rate limiting may be active
- Increase `retry_delay_ms` and `retry_attempts`
- Check network connectivity

### "Market order failed"

- Insufficient USDC balance
- Tokens not approved (run approvals example)
- Insufficient liquidity in market
- Check `max_slippage` setting

### Bot crashes or hangs

- Check logs for error messages
- Verify all dependencies are up to date
- Ensure Rust version is 1.88+
- Try increasing `retry_attempts`

## 📈 Performance Optimization

### Reduce Latency

- Run on a VPS close to Polymarket servers (US East Coast)
- Use a low-latency Polygon RPC endpoint
- Decrease `price_poll_interval_seconds` to 1-2 seconds

### Improve Signal Quality

- Increase `min_volume_differential` to 0.15-0.20 for higher conviction
- Adjust `volume_lookback_seconds` based on market conditions
- Consider adding filters (minimum total volume, trade count)

### Scale Trading

- Increase `trade_size_usdc` gradually after proven results
- Run multiple instances for different markets (requires separate configs)
- Implement position sizing based on confidence/volume differential

## 🧪 Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --lib market
cargo test --lib trailing_stop

# Run with output
cargo test -- --nocapture
```

### Integration Testing

```bash
# Test in dry run mode
# Set dry_run = true in config.toml
cargo run --release

# Monitor for:
# - Market discovery working
# - Volume analysis completing
# - Signal generation logic
# - Trailing stop calculations
```

## 📝 Logging Examples

### Successful Trade Flow

```
[INFO] Discovering next upcoming market...
[INFO] Found next market: Bitcoin 15min Up or Down? (14:30 - 14:45 UTC) | Starts in: 8m 23s
[INFO] Entry timing reached, proceeding to analysis
[INFO] Analyzing market volume...
[INFO] Volume Analysis | UP: $1,234.50 (45 trades) | DOWN: $678.90 (23 trades) | Diff: 29.03%
[INFO] Generated signal: UP with 29.03% confidence
[INFO] Entering UP position on market: Bitcoin 15min Up or Down? (14:30 - 14:45 UTC)
[INFO] Current UP price: 0.5200
[INFO] Position opened: UP 9.62 shares @ 0.5200 ($5.00)
[INFO] New high for UP position: 0.5450 (previous: 0.5200)
[INFO] Minimum profit target 2.00% reached (current: 4.81%), activating trailing stop
[INFO] Trailing stop triggered! Price retraced 5.02% from best
[INFO] Exiting UP position | Reason: Trailing Stop
[INFO] Exit price: 0.5177 | P&L: -$0.12 (-2.37%)
[INFO] Position closed successfully
```

## 📚 Additional Resources

- [Polymarket Documentation](https://docs.polymarket.com)
- [rs-clob-client GitHub](https://github.com/Polymarket/rs-clob-client)
- [Polygon Network](https://polygon.technology)
- [Rust Book](https://doc.rust-lang.org/book/)

## 📄 License

This project is provided as-is for educational and research purposes. Use at your own risk.

## ⚖️ Disclaimer

This software is for educational purposes only. Trading prediction markets involves substantial risk of loss. The authors are not responsible for any financial losses incurred through the use of this software. Always conduct your own research and never risk more than you can afford to lose.

---

**Built with ❤️ in Rust | Powered by Polymarket rs-clob-client**
