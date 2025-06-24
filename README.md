# Quantus CLI

A modern command line interface for interacting with the Quantus Network, featuring built-in quantum-safe wallet management and real blockchain operations.

## 🌟 Features

- **Quantum-Safe Wallets**: Built with Dilithium post-quantum cryptography
- **Generic Pallet Calls**: Call ANY blockchain function using metadata-driven parsing
- **Real Chain Operations**: Send tokens, query balances, explore metadata
- **Smart Type Detection**: Automatic parsing of addresses, balances, and data types
- **Developer Tools**: Pre-built test wallets and utilities
- **Modern CLI**: Built with Rust and Clap for excellent UX
- **Cross-Platform**: Runs on macOS, Linux, and Windows
- **Beautiful UI**: Colorized output with emoji indicators and progress spinners
- **Smart Balance Display**: Automatic formatting with proper decimals and token symbols
- **Password Convenience**: Multiple authentication options including environment variables

## 🚀 Quick Start

### Installation

```bash
# Clone and build
git clone <repository-url>
cd quantus-cli
cargo build --release

# The binary will be available as `quantus`
```

### First Steps

```bash
# Get help
quantus --help

# Create your first wallet
quantus wallet create --name my-wallet

# Create test wallets for development
quantus developer create-test-wallets --verbose

# Check a wallet's balance
quantus balance --address 5H7DdvKue19FQZpRKc2hmBfSBGEczwvdnVYDNZC3W95UDyGP

# Send tokens
quantus send --from crystal_alice --to 5H7DdvKue19FQZpRKc2hmBfSBGEczwvdnVYDNZC3W95UDyGP --amount 10.5

# Call any blockchain function generically
quantus call --pallet Balances --call transfer_allow_death --args '["5H7DdvKue19FQZpRKc2hmBfSBGEczwvdnVYDNZC3W95UDyGP", "5"]' --from crystal_alice

# Explore the blockchain
quantus metadata --no-docs
```

## 📋 All Commands

### Global Options
Available for all commands:
- `--verbose` / `-v`: Enable debug logging with detailed output
- `--node-url <URL>`: Specify node endpoint (default: `ws://127.0.0.1:9944`)

## 💼 Wallet Management

### Create Wallet
Creates a new quantum-safe wallet with Dilithium post-quantum cryptography:

```bash
# Create with prompted password
quantus wallet create --name my-wallet

# Create with password parameter (not recommended for production)
quantus wallet create --name my-wallet --password mypassword

# Verbose output shows detailed creation process
quantus wallet create --name my-wallet --verbose
```

**Output:**
```
🔐 Creating new quantum wallet...
Wallet name: my-wallet
Address: 5H7DdvKue19FQZpRKc2hmBfSBGEczwvdnVYDNZC3W95UDyGP
Key type: ML-DSA-87 (Dilithium)
Created: 2024-01-15 10:30:45 UTC
✅ Wallet created successfully!
```

### List Wallets
Shows all available wallets:

```bash
quantus wallet list
```

**Output:**
```
📁 Found 3 wallets:

1. crystal_alice
   Address: 5H7DdvKue19FQZpRKc2hmBfSBGEczwvdnVYDNZC3W95UDyGP
   Type: ML-DSA-87 (Dilithium)
   Created: 2024-01-15 09:15:30 UTC

2. crystal_bob
   Address: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
   Type: ML-DSA-87 (Dilithium)
   Created: 2024-01-15 09:15:31 UTC

3. my-wallet
   Address: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy
   Type: ML-DSA-87 (Dilithium)
   Created: 2024-01-15 10:30:45 UTC
```

### View Wallet Details
Display detailed information about wallets:

```bash
# View specific wallet
quantus wallet view --name my-wallet

# View all wallets (same as list but different format)
quantus wallet view --all
```

### Export Wallet
Export wallet data in various formats:

```bash
# Export mnemonic phrase (default)
quantus wallet export --name my-wallet --format mnemonic

# Export private key
quantus wallet export --name my-wallet --format private-key
```

### Import Wallet
Import a wallet from mnemonic phrase:

```bash
# Import with prompted mnemonic
quantus wallet import --name imported-wallet

# Import with mnemonic parameter
quantus wallet import --name imported-wallet --mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# Import with password
quantus wallet import --name imported-wallet --password mypassword
```

### Delete Wallet
Remove a wallet (with safety confirmation):

```bash
# Delete with confirmation prompt
quantus wallet delete --name my-wallet

# Force delete without confirmation (be careful!)
quantus wallet delete --name my-wallet --force
```

**Interactive confirmation:**
```
⚠️  You are about to delete wallet 'my-wallet'
📍 Address: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy
⚠️  This action cannot be undone!

To confirm deletion, type the wallet name: my-wallet
```

## 💰 Blockchain Operations

### Query Balance
Check account balances with automatic formatting:

```bash
quantus balance --address 5H7DdvKue19FQZpRKc2hmBfSBGEczwvdnVYDNZC3W95UDyGP
```

**Output:**
```
✅ Balance: 1152921500.346108076 DEV
```

### Send Tokens
Transfer tokens between accounts with multiple convenience options:

```bash
# Basic send
quantus send --from crystal_alice --to 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty --amount 10

# Send with decimal amounts
quantus send --from my-wallet --to crystal_bob --amount 10.5 --verbose

# Send with password parameter
quantus send --from my-wallet --to crystal_bob --amount 0.0001 --password mypassword

# Send with password from file (for scripting)
quantus send --from my-wallet --to crystal_bob --amount 100 --password-file /path/to/password.txt
```

**Password Convenience Options:**
1. **Environment Variables**: Set `QUANTUS_WALLET_PASSWORD` or `QUANTUS_WALLET_PASSWORD_CRYSTAL_ALICE`
2. **CLI Flags**: Use `--password` or `--password-file`
3. **Empty Password**: Automatically tries empty password for development wallets
4. **Interactive Prompt**: Falls back to secure password prompt

**Verbose Output:**
```
🚀 Preparing to send tokens...
   From: crystal_alice (5H7DdvKue19FQZpRKc2hmBfSBGEczwvdnVYDNZC3W95UDyGP)
   To: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
   Amount: 10.5 DEV (10500000000000 raw units)

✅ Empty password works for wallet 'crystal_alice'
🔗 Connecting to Quantus node: ws://127.0.0.1:9944
✅ Connected to Quantus node successfully!
💰 Balance before: 1152921500.346108076 DEV
🚀 Creating transfer transaction...
✍️  Creating balance transfer extrinsic...
🔗 Waiting for confirmation... / (3s)
📋 Transaction hash: 0x1234567890abcdef...
🔗 Included in block: 0xabcdef1234567890...
💸 Transaction fee: 1.410246299 DEV
✅ Transaction completed successfully!
💰 Balance after: 1152921489.935861777 DEV
```

### Generic Pallet Calls
**🚀 NEW!** Call ANY pallet function on the blockchain using metadata-driven argument parsing:

```bash
# Basic call - transfer tokens using generic interface
quantus call --pallet Balances --call transfer_allow_death --args '["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", "10.5"]' --from crystal_alice

# Call with tip to prioritize transaction
quantus call --pallet Balances --call transfer_allow_death --args '["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", "5"]' --from my-wallet --tip 0.1

# Generate call data only (hex-encoded)
quantus call --pallet Balances --call transfer_allow_death --args '["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", "1"]' --call-data-only

# Create offline extrinsic without submitting
quantus call --pallet System --call remark --args '["0x48656c6c6f20576f726c64"]' --from crystal_alice --offline

# Use password options (same as send command)
quantus call --pallet Balances --call force_transfer --args '["5H7Dd...", "5FHne...", "100"]' --from admin-wallet --password-file /secure/password.txt
```

**Smart Argument Parsing:**
The system automatically detects and converts argument types:

- **📍 SS58 Addresses**: `"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"` → AccountId
- **💰 Balance Amounts**: `"10.5"` → 10500000000 (with proper decimals)
- **🔢 Numbers**: `"42"` → u64/u128 as needed
- **✅ Booleans**: `"true"`, `"false"` → bool
- **📦 Hex Bytes**: `"0x48656c6c6f"` → Vec<u8>
- **📝 Strings**: `"Hello World"` → String/Vec<u8>

**Three Operation Modes:**

1. **Live Submission** (default): Submit to blockchain and wait for confirmation
2. **Call Data Only** (`--call-data-only`): Generate hex-encoded call data
3. **Offline Extrinsic** (`--offline`): Create signed extrinsic without submitting

**Example Output:**
```bash
quantus call --pallet Balances --call transfer_allow_death --args '["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "10.5"]' --from crystal_alice --verbose
```

```
🔮 Quantus CLI
🚀 EXECUTE Generic Call: Balances.transfer_allow_death
📡 Connecting to Quantus node: ws://127.0.0.1:9944
✅ Connected successfully! Chain: Quantus Network
🔍 Found Balances pallet (index: 2)
🔍 Found transfer_allow_death call (index: 0)
✅ Call signature validated: 2 arguments required
🔧 Parsing arguments based on call signature...
   • Arg 1: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY" → AccountId (SS58 detected)
   • Arg 2: "10.5" → Balance (10500000000 with 9 decimals)
✅ Arguments parsed successfully!
🔑 Trying empty password first...
✅ Empty password works for wallet 'crystal_alice'
🎯 Creating balance transfer extrinsic...
🔧 Creating extrinsic using compose_extrinsic! macro...
✅ Extrinsic created successfully!
📤 Submitting to chain and watching for inclusion...
✅ Transaction included in block!
📋 Transaction Hash: 0x27b05359125a94fb6c7dac4b90148e6f2e57f2d84f1ef63d78c497110e441e35
📦 Block Hash: 0x82f64f6b4e58cc090b33fcd02a649357f9c5b59a7a87719a671687964de7e7fd
📋 Events:
  • Balances.Transfer: 10.5 DEV transferred
  • TransactionPayment.TransactionFeePaid: 0.000000014084 DEV
  • System.ExtrinsicSuccess: Transaction completed
✅ Transaction completed successfully!
```

**Call Data Only Mode:**
```bash
quantus call --pallet Balances --call transfer_allow_death --args '["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "10.5"]' --call-data-only
```

```
🔮 Quantus CLI
🔧 CALL DATA ONLY: Balances.transfer_allow_death
📡 Connecting to node for metadata...
✅ Call data generated successfully!
📋 Hex-encoded call data:
0x020000d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d070049d97102
```

**Supported Pallets:**
Any pallet available on the chain! Common examples:
- **Balances**: `transfer_allow_death`, `transfer_keep_alive`, `force_transfer`
- **System**: `remark`, `set_heap_pages`, `kill_storage`
- **Timestamp**: `set`
- **Utility**: `batch`, `batch_all`
- **And many more!** Use `quantus metadata` to explore

**Advanced Features:**
- **Metadata-Driven**: Automatically discovers available pallets and calls
- **Type Safety**: Validates argument count against call signatures
- **Error Recovery**: Clear error messages for invalid calls or arguments
- **Compose Macros**: Uses `compose_call!` and `compose_extrinsic!` for maximum compatibility
- **Password Convenience**: Same smart password handling as other commands

### System Information
Query blockchain system information:

```bash
quantus system
```

### Metadata Exploration
Explore all available blockchain functionality:

```bash
# Full metadata with documentation
quantus metadata

# Compact view without docs
quantus metadata --no-docs
```

**Output:**
```
🏗️  Chain Metadata Information:
📦 Found 22 pallets:

1. System (Index: 0)
   📞 Calls (6):
      1. remark
         📝 Make some on-chain remark
         📥 Parameters:
           • remark: Vec<u8>
      2. set_heap_pages
         📥 Parameters:
           • pages: u64
   💾 Storage (16):
      1. Account
      2. BlockHash
   📡 Events (6):
      1. ExtrinsicSuccess
      2. ExtrinsicFailed

2. Balances (Index: 10)
   📞 Calls (9):
      1. transfer_allow_death
         📝 Transfer some liquid free balance to another account
         📥 Parameters:
           • dest: AccountIdLookupOf<T>
           • value: u128
      2. force_transfer
         📥 Parameters:
           • source: AccountIdLookupOf<T>
           • dest: AccountIdLookupOf<T>
           • value: u128

...

💡 Use this information to implement new extrinsic calls!
💡 Each call can be submitted using the submit_extrinsic! macro
```

## 🧪 Developer Tools

### Create Test Wallets
Generate standard test wallets for development:

```bash
quantus developer create-test-wallets --verbose
```

**Output:**
```
🧪 DEVELOPER Creating standard test wallets...

✅ Created crystal_alice
   Address: 5H7DdvKue19FQZpRKc2hmBfSBGEczwvdnVYDNZC3W95UDyGP
   Description: Alice's test wallet for development

✅ Created crystal_bob  
   Address: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
   Description: Bob's test wallet for development

✅ Created crystal_charlie
   Address: 5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy
   Description: Charlie's test wallet for development

🎉 Test wallet creation complete!
✅ Created 3 test wallets with empty passwords for easy development
💡 Use these wallets for testing - they use empty passwords by default
```

**Test Wallets:**
- `crystal_alice`: Primary test account with empty password
- `crystal_bob`: Secondary test account with empty password  
- `crystal_charlie`: Third test account with empty password

## 📊 Version Information

```bash
quantus version
```

**Output:**
```
Quantus CLI v0.1.0
Build: Command line interface for the Quantus Network
```

## 🔧 Environment Variables

### Password Management
- `QUANTUS_WALLET_PASSWORD`: Global password for all wallets
- `QUANTUS_WALLET_PASSWORD_<WALLET_NAME>`: Wallet-specific password (e.g., `QUANTUS_WALLET_PASSWORD_CRYSTAL_ALICE`)

### Node Configuration  
- Set via `--node-url` flag or default to `ws://127.0.0.1:9944`

## 💡 Usage Examples

### Complete Workflow Example
```bash
# 1. Set up development environment
export QUANTUS_WALLET_PASSWORD_CRYSTAL_ALICE=""
quantus developer create-test-wallets

# 2. Check initial balance
quantus balance --address 5H7DdvKue19FQZpRKc2hmBfSBGEczwvdnVYDNZC3W95UDyGP

# 3. Create your own wallet
quantus wallet create --name my-production-wallet

# 4. Send some test tokens (traditional way)
quantus send --from crystal_alice --to my-production-wallet --amount 100 --verbose

# 4b. Or use the generic call interface
quantus call --pallet Balances --call transfer_allow_death --args '["my-production-wallet", "50"]' --from crystal_alice

# 5. Verify the transfer
quantus balance --address $(quantus wallet view --name my-production-wallet | grep Address | cut -d' ' -f2)

# 6. Explore what else you can do
quantus metadata --no-docs

# 7. Try other generic calls
quantus call --pallet System --call remark --args '["0x48656c6c6f20576f726c64"]' --from crystal_alice
```

### Scripting Example
```bash
#!/bin/bash
# Script to distribute tokens to multiple accounts

export QUANTUS_WALLET_PASSWORD_CRYSTAL_ALICE=""

RECIPIENTS=(
    "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
    "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy"
)

for recipient in "${RECIPIENTS[@]}"; do
    echo "Sending 50 DEV to $recipient"
    quantus send --from crystal_alice --to "$recipient" --amount 50
    sleep 2  # Wait between transactions
done
```

## 🏗️ Architecture

### Quantum-Safe Cryptography
- **Dilithium (ML-DSA-87)**: Post-quantum digital signatures
- **Secure Storage**: AES-256-GCM + Argon2 encryption for wallet files
- **Future-Proof**: Ready for ML-KEM key encapsulation

### Smart Features
- **Dynamic Balance Formatting**: Automatically fetches chain decimals and token symbol
- **Progress Indicators**: Spinners during network operations
- **Error Recovery**: Comprehensive error handling with helpful messages
- **Development Mode**: Empty password detection for test wallets

### Real Blockchain Integration
- **Substrate Integration**: Direct connection to Quantus node via WebSocket
- **Metadata-Driven**: Discovers available functionality from chain metadata
- **Transaction Monitoring**: Real-time transaction confirmation and fee calculation
- **Extensible Architecture**: Macro-based extrinsic submission supports any pallet

## 🛠️ Current Status

**✅ Fully Implemented:**
- Complete wallet management (create, view, list, export, import, delete)
- Real token transfers with fee calculation
- **Generic pallet calls with smart type detection**
- **Metadata-driven function discovery and validation**
- Balance queries with proper formatting
- Chain metadata exploration
- Developer test wallet utilities
- Password convenience features
- Progress indicators and verbose logging

**🔮 Architecture Ready For:**
- Additional pallet integrations (staking, governance, etc.)
- Hardware wallet support
- Multi-chain support
- Advanced key derivation

## 📦 Technical Dependencies

### Core Runtime
- `clap`: Modern CLI argument parsing
- `tokio`: Async runtime for blockchain operations
- `substrate-api-client`: Direct Substrate chain integration
- `serde` + `serde_json`: Data serialization

### Cryptography
- `dilithium-crypto`: Post-quantum signatures
- `aes-gcm`: Symmetric encryption
- `argon2`: Password-based key derivation
- `bip39`: Mnemonic phrase generation

### User Experience
- `colored`: Terminal output colorization
- `chrono`: Date/time formatting
- `thiserror`: Structured error handling

## 🎯 Real-World Ready

The Quantus CLI is a **production-ready** tool that:

✅ **Handles Real Money**: All transactions are real and irreversible  
✅ **Quantum-Safe**: Uses post-quantum cryptography for future security  
✅ **Developer-Friendly**: Rich tooling and clear error messages  
✅ **Scriptable**: Environment variables and flags for automation  
✅ **Extensible**: Clean architecture for adding new blockchain features  

**⚠️ Security Note**: This tool handles real cryptocurrency. Always:
- Back up your wallet files and mnemonic phrases
- Use strong passwords for production wallets
- Test with small amounts first
- Keep your private keys secure

---

🚀 **Ready to explore the quantum future of blockchain!** 🚀 