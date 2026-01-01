#!/bin/bash
# verify_api.sh

BASE_URL="http://localhost:8080"
echo "Waiting for API to be healthy..."
until curl -s $BASE_URL/health > /dev/null; do
    echo "Waiting for backend..."
    sleep 5
done

echo "✅ Backend is UP"

# 1. Test Chat Persistence
echo "Testing Chat Persistence..."
USER_ID=$(uuidgen)
ASSISTANT_ID=$(uuidgen)
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

curl -s -X POST "$BASE_URL/api/chat/persist" \
  -H "Content-Type: application/json" \
  -d "{
    \"user_message\": {
      \"id\": \"$USER_ID\",
      \"role\": \"user\",
      \"content\": \"Test Message User\",
      \"timestamp\": \"$TIMESTAMP\"
    },
    \"assistant_message\": {
      \"id\": \"$ASSISTANT_ID\",
      \"role\": \"assistant\",
      \"content\": \"Test Message Assistant\",
      \"timestamp\": \"$TIMESTAMP\"
    }
  }" | grep "persisted" && echo "✅ Chat Persist Success" || echo "❌ Chat Persist Failed"

# Verify in history
curl -s "$BASE_URL/api/chat/history" | grep "Test Message User" && echo "✅ Chat History Validated" || echo "❌ Chat History Verification Failed"

# 2. Test Trade CRUD
echo "Testing Trade Creation..."
# Assuming create_trade works with minimal valid payload
# Requires valid Order types. 
# JSON payload structure should match CreateTradeRequest
curl -s -X POST "$BASE_URL/api/v1/trades" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTC/USD",
    "side": "Buy",
    "order_type": "Market",
    "quantity": "0.1"
  }' > create_trade_response.json

cat create_trade_response.json
grep "success" create_trade_response.json && echo "✅ Trade Created" || echo "❌ Trade Creation Failed"

TRADE_ID=$(cat create_trade_response.json | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
echo "Trade ID: $TRADE_ID"

if [ ! -z "$TRADE_ID" ]; then
    echo "Testing Get Trade..."
    curl -s "$BASE_URL/api/v1/trades/$TRADE_ID" | grep "BTC/USD" && echo "✅ Get Trade Success" || echo "❌ Get Trade Failed"

    echo "Testing Cancel Trade..."
    curl -s -X DELETE "$BASE_URL/api/v1/trades/$TRADE_ID" | grep "cancelled" && echo "✅ Cancel Trade Success" || echo "❌ Cancel Trade Failed"
fi

echo "Verification Complete"
