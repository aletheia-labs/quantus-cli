#!/bin/bash

# Script to get the latest block number and timestamp from Quantus node
# Usage: ./get_block_info.sh [node_url]

NODE_URL="${1:-http://127.0.0.1:9944}"

echo "🔗 Querying Quantus node: $NODE_URL"
echo

# Get latest block header
# echo "📋 Getting latest block header..."
HEADER_RESPONSE=$(curl -s -H "Content-Type: application/json" -d '{
  "id":1,
  "jsonrpc":"2.0",
  "method":"chain_getHeader",
  "params":[]
}' "$NODE_URL")

# Extract block number (hex)
BLOCK_HEX=$(echo "$HEADER_RESPONSE" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    print(data['result']['number'])
except:
    print('ERROR')
")

if [ "$BLOCK_HEX" = "ERROR" ]; then
    echo "❌ Failed to get block header"
    exit 1
fi

# Convert hex to decimal
BLOCK_DECIMAL=$((BLOCK_HEX))

# echo "📊 Latest Block:"
# echo "   Hex: $BLOCK_HEX"
# echo "   Decimal: $BLOCK_DECIMAL"
# echo

# Get timestamp from storage
# echo "🕐 Getting current timestamp..."
TIMESTAMP_RESPONSE=$(curl -s -H "Content-Type: application/json" -d '{
  "id":1,
  "jsonrpc":"2.0",
  "method":"state_getStorage",
  "params":["0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb"]
}' "$NODE_URL")

# Extract and decode timestamp
TIMESTAMP_INFO=$(echo "$TIMESTAMP_RESPONSE" | python3 -c "
import json, sys, struct, datetime

try:
    data = json.load(sys.stdin)
    hex_data = data['result'][2:]  # Remove '0x' prefix
    
    if len(hex_data) != 16:  # u64 = 8 bytes = 16 hex chars
        print('ERROR: Invalid timestamp length')
        sys.exit(1)
    
    # Decode little-endian u64
    bytes_data = bytes.fromhex(hex_data)
    timestamp_ms = struct.unpack('<Q', bytes_data)[0]
    timestamp_s = timestamp_ms / 1000
    
    # Format as human-readable
    dt = datetime.datetime.fromtimestamp(timestamp_s, tz=datetime.timezone.utc)
    human_time = dt.strftime('%Y-%m-%d %H:%M:%S UTC')
    
    print(f'{timestamp_ms}|{timestamp_s}|{human_time}')
except Exception as e:
    print(f'ERROR: {e}')
")

if [[ "$TIMESTAMP_INFO" == ERROR* ]]; then
    echo "❌ Failed to decode timestamp: $TIMESTAMP_INFO"
    exit 1
fi

# Parse timestamp info
IFS='|' read -r TIMESTAMP_MS TIMESTAMP_S HUMAN_TIME <<< "$TIMESTAMP_INFO"

# echo "⏰ Current Timestamp:"
# echo "   Milliseconds: $TIMESTAMP_MS"
# echo "   Seconds: $TIMESTAMP_S"
# echo "   Human readable: $HUMAN_TIME"
# echo

# Calculate block time (approximate)
echo "📈 Chain Info:"
echo "   Current block: #$BLOCK_DECIMAL"
echo "   Block time: $HUMAN_TIME"
echo

# echo "✅ Block info retrieved successfully!" 