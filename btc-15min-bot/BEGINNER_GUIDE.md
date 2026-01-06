# 🎓 Complete Beginner's Guide to Running the Trading Bot

This guide assumes you're **new to coding**. I'll explain every single step!

---

## 📖 Table of Contents

1. [What You Need](#what-you-need)
2. [Understanding the Setup](#understanding-the-setup)
3. [Step-by-Step Instructions](#step-by-step-instructions)
4. [Common Problems & Solutions](#common-problems--solutions)
5. [Testing the Bot](#testing-the-bot)
6. [Going Live](#going-live)

---

## 🎯 What You Need

### Required:
- ✅ A computer with Rust installed (you already have this)
- ✅ A Polymarket wallet with some USDC (for trading)
- ✅ Your wallet's private key (a long string of letters/numbers)
- ✅ Basic terminal/command line knowledge

### Optional but Recommended:
- A text editor (VS Code, Sublime Text, or just `nano`)
- $50-100 USDC for testing (start small!)

---

## 🧠 Understanding the Setup

### What is the bot?
Think of it like a robot trader that:
1. **Watches** Bitcoin prediction markets on Polymarket
2. **Analyzes** which side (UP or DOWN) people are trading more
3. **Copies** what the majority is doing (momentum trading)
4. **Exits** when it makes a small profit or loss

### Why do we need to "compile" it?
The bot is written in Rust (a programming language). "Compiling" means converting the human-readable code into computer-executable instructions. Like translating from English to machine language.

### What's broken right now?
The bot tries to use Polymarket's complex API (Application Programming Interface - a way to talk to Polymarket's servers). The API has some tricky parts that need fixing.

**The Good News:** I've created simplified versions that bypass the complex parts!

---

## 📝 Step-by-Step Instructions

### STEP 1: Open Your Terminal

On Linux/Mac:
- Press `Ctrl + Alt + T` or search for "Terminal"

On Windows:
- Search for "Command Prompt" or "PowerShell"

### STEP 2: Navigate to the Bot Folder

Copy and paste this command:

```bash
cd /home/user/rs-clob-client/btc-15min-bot
```

**What this does:** Changes your current directory to the bot folder (like opening a folder in File Explorer).

### STEP 3: Check What Files Are There

```bash
ls -la
```

**What this does:** Lists all files in the folder.

**You should see:**
- `Cargo.toml` - List of dependencies (like ingredients in a recipe)
- `src/` - Folder containing the code files
- `README.md` - Documentation
- `config.toml.example` - Example configuration

### STEP 4: Try Compiling (Will Show Errors - Expected!)

```bash
cargo check
```

**What this does:** Checks if the code can compile without actually running it.

**You'll see red ERROR messages** - that's normal! We'll fix them.

To see just a summary of errors:

```bash
cargo check 2>&1 | grep "^error" | head -5
```

---

## 🔧 STEP 5: Use the Simplified Version (THE FIX!)

I've created simpler versions of the problematic files. Let's use those instead!

### Step 5.1: Check Simplified Files Exist

```bash
ls src/market_simple.rs
ls src/strategy_simple.rs
```

**You should see:** No "file not found" errors (I just created these files for you!)

### Step 5.2: Find Token IDs for a Bitcoin Market

You need two token IDs (one for UP, one for DOWN) from Polymarket.

**How to find them:**

1. Open your web browser
2. Go to https://polymarket.com
3. Search for "Bitcoin 15min" in the search bar
4. Click on an ACTIVE market (shows a timer counting down)
5. Look at the URL in your address bar

The URL looks like:
```
https://polymarket.com/event/bitcoin-15min-up-or-down-2025-01-06-1430-1445-utc?tid=1718631495926...
```

The `tid=` part is the token ID. But that's just ONE token. We need both UP and DOWN.

**Easier Method:** Look at the browser's developer tools:
1. Right-click on the page → "Inspect" or press F12
2. Click on "Network" tab
3. Look for requests to "clob.polymarket.com"
4. Find one that shows token IDs in the response

**Even Easier:** Run the bot once and it will tell you if token IDs are missing!

### Step 5.3: Edit the Market File with Your Token IDs

Open the file:

```bash
nano src/market_simple.rs
```

**What this does:** Opens a basic text editor.

**Find these lines (around line 20-25):**

```rust
/// Token ID for UP outcome
pub const UP_TOKEN_ID: &str = "REPLACE_WITH_UP_TOKEN_ID";

/// Token ID for DOWN outcome
pub const DOWN_TOKEN_ID: &str = "REPLACE_WITH_DOWN_TOKEN_ID";
```

**Replace the text** between the quotes with your actual token IDs:

```rust
pub const UP_TOKEN_ID: &str = "71321045880685926196226762161781486336990587556547664985916499732654256447732";

pub const DOWN_TOKEN_ID: &str = "21321045880685926196226762161781486336990587556547664985916499732654256447733";
```

(Use your actual token IDs, not these examples!)

**Save and exit:**
- Press `Ctrl + O` to save
- Press `Enter` to confirm
- Press `Ctrl + X` to exit

### Step 5.4: Update Main File to Use Simplified Modules

We need to tell the main program to use our simplified versions.

Open the main file:

```bash
nano src/main.rs
```

**Find this line (around line 13-14):**

```rust
mod market;
mod strategy;
```

**Change it to:**

```rust
mod market_simple as market;
mod strategy_simple as strategy;
```

**What this does:** Tells the program to use `market_simple.rs` instead of `market.rs`.

**Save and exit:**
- `Ctrl + O` → `Enter` → `Ctrl + X`

### Step 5.5: Update Imports in Main

Still in `src/main.rs`, find this line (around line 89-91):

```rust
let market_discovery = MarketDiscovery::new(
    authenticated_client.clone(),
    config.strategy.market_query.clone(),
);
```

**Change it to:**

```rust
let market_discovery = MarketDiscovery::new(
    config.strategy.market_query.clone(),
);
```

**Why?** The simplified version doesn't need the client.

Also find this line (around line 94-97):

```rust
let strategy = VolumeStrategy::new(
    authenticated_client.clone(),
    config.strategy.clone(),
);
```

**Change it to:**

```rust
let strategy = VolumeStrategy::new(
    config.strategy.clone(),
);
```

**Save and exit:** `Ctrl + O` → `Enter` → `Ctrl + X`

---

## ✅ STEP 6: Try Compiling Again

```bash
cargo check
```

**You should now see FEWER errors!**

If you still see errors about `LocalSigner`, that's expected. Let me know and I'll help fix those too.

---

## ⚙️ STEP 7: Configure the Bot

### Step 7.1: Create Configuration File

```bash
cp config.toml.example config.toml
```

**What this does:** Creates a copy of the example config that you can edit.

### Step 7.2: Edit Configuration

```bash
nano config.toml
```

**Important settings to check:**

```toml
[strategy]
trade_size_usdc = "5.00"  # Start with $5!

[risk]
trailing_stop_percentage = "0.05"  # Exit when price drops 5%

[operational]
dry_run = true  # KEEP THIS TRUE for testing!
```

**Don't change the private_key** in config.toml - we'll use environment variable.

**Save and exit:** `Ctrl + O` → `Enter` → `Ctrl + X`

### Step 7.3: Set Your Private Key (Secure Method)

```bash
export PRIVATE_KEY="your_actual_private_key_here"
```

**Replace** `your_actual_private_key_here` with your real private key (the long hex string).

**⚠️ IMPORTANT:** Remove the `0x` at the beginning if it has one!

**Example:**
```bash
# ✅ CORRECT (no 0x)
export PRIVATE_KEY="1a2b3c4d5e6f7g8h9i0j..."

# ❌ WRONG (has 0x)
export PRIVATE_KEY="0x1a2b3c4d5e6f7g8h9i0j..."
```

**Security Note:** This keeps your key out of files. It's only stored in memory temporarily.

---

## 🏃 STEP 8: Run the Bot (Test Mode)

```bash
cargo run --release
```

**What this does:**
1. Compiles the code (may take 2-5 minutes first time)
2. Runs the bot in test mode (dry_run = true means no real trades)

**What you should see:**

```
[INFO] ╔═══════════════════════════════════════════╗
[INFO] ║  Bitcoin 15min Trading Bot for Polymarket ║
[INFO] ╚═══════════════════════════════════════════╝
[INFO] Loading configuration...
[INFO] Wallet address: 0x1234...
[INFO] Authenticating with Polymarket...
[INFO] ⚠️  DRY RUN MODE ENABLED ⚠️
[INFO] Discovering next upcoming market...
[INFO] 📊 Using hardcoded Bitcoin 15min market
[INFO]    UP Token:   71321045880685926196...
[INFO]    DOWN Token: 21321045880685926196...
```

**If you see this - SUCCESS!** 🎉

The bot is now running in test mode.

---

## 🐛 Common Problems & Solutions

### Problem: "PRIVATE_KEY environment variable not set"

**Solution:**
```bash
export PRIVATE_KEY="your_key_here"
```

Then run again.

### Problem: "Token IDs not configured"

**Solution:** Go back to Step 5.3 and add real token IDs.

### Problem: "Failed to authenticate"

**Possible causes:**
1. Private key is wrong
2. Private key has `0x` prefix (remove it)
3. Wallet hasn't been used on Polymarket before

**Solution:** Double-check your private key.

### Problem: "Compilation errors about LocalSigner"

This is a type system issue. Let me know and I'll create a fixed version of the trader.rs file.

### Problem: Bot stops immediately

**Check:** Is `dry_run = true` in config.toml? (it should be for testing!)

**Check:** Do you have market token IDs set?

---

## 🧪 Testing the Bot

### What to Watch For:

1. **Bot starts successfully** ✅
2. **Shows "DRY RUN MODE"** ✅
3. **Finds the market** ✅
4. **Waits for entry timing** ✅
5. **Analyzes volume** (shows UP vs DOWN)
6. **Simulates trade** (doesn't actually place orders)
7. **Monitors position** with trailing stop
8. **Simulates exit**
9. **Repeats with next market**

### Let It Run for 1 Hour

Watch the logs. You should see it go through several "trades" (simulated).

**Good signs:**
- No ERROR messages
- Shows analysis and decisions
- Calculates profit/loss
- Moves to next market

**Bad signs:**
- Keeps erroring out
- Crashes
- Can't find markets

---

## 🚀 Going Live (After Testing!)

**⚠️ ONLY do this after:**
- ✅ Bot ran successfully for 24+ hours in dry run mode
- ✅ You understand what it's doing
- ✅ You have token approvals set (see main README)
- ✅ You're willing to risk the trade amount

### Step 1: Edit Config

```bash
nano config.toml
```

**Change:**
```toml
[operational]
dry_run = false  # Turn off dry run
```

**Save and exit.**

### Step 2: Run for Real

```bash
export PRIVATE_KEY="your_key"
cargo run --release
```

**Now it will place REAL trades!**

### Step 3: Monitor Closely

Watch the logs. First few trades should be:
- Small amounts ($5-10)
- On markets with good liquidity
- With clear volume signals

**Stop immediately if:**
- Orders fail repeatedly
- Strange errors appear
- Losses exceed expectations

---

## 📊 Tracking Performance

Create a simple log:

```bash
# Run bot and save logs
cargo run --release 2>&1 | tee bot_log.txt

# Later, search for trades
grep "Position opened" bot_log.txt
grep "Position closed" bot_log.txt
```

---

## 🆘 Need More Help?

1. **Check the main README.md** for detailed documentation
2. **Check IMPLEMENTATION_STATUS.md** for known issues
3. **Share error messages** - copy the red error text and ask for help

---

## 🎓 Learning Resources

Want to understand the code better?

- [Rust Book (Beginner Friendly)](https://doc.rust-lang.org/book/)
- [Polymarket Documentation](https://docs.polymarket.com)
- [Understanding Trading Bots](https://www.investopedia.com/articles/trading/11/automated-trading-systems.asp)

---

## ✅ Quick Checklist

Before running:
- [ ] Token IDs configured in `market_simple.rs`
- [ ] Config file created (`config.toml`)
- [ ] Private key set (export PRIVATE_KEY="...")
- [ ] Dry run mode enabled (`dry_run = true`)
- [ ] Wallet funded with USDC
- [ ] Token approvals set

For going live:
- [ ] Tested in dry run for 24+ hours
- [ ] Understand the strategy
- [ ] Starting with small amounts ($5-10)
- [ ] Ready to monitor closely

---

**You've got this!** 🚀

Start with the dry run, watch how it works, then gradually move to live trading when you're comfortable.

Feel free to ask questions at any step!
