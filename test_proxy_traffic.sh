#!/bin/bash

# Configuration
PROXY_HOST="127.0.0.1"
PROXY_PORT="9095"
TARGET_URL="http://testphp.vulnweb.com/AJAX/index.php"
DB_PATH="./proxxy.db"

echo "🚀 Testing Proxy Traffic via $PROXY_HOST:$PROXY_PORT"

# 1. Send request through proxy
echo "1️⃣  Sending CURL request..."
RESPONSE=$(curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w "%{http_code}" $TARGET_URL)

if [ "$RESPONSE" -eq 200 ]; then
    echo "✅ Request successful (HTTP 200)"
else
    echo "❌ Request failed (HTTP $RESPONSE)"
    exit 1
fi

# 2. Check Database
echo "2️⃣  Checking Database ($DB_PATH)..."

# Ensure sqlite3 is installed
if ! command -v sqlite3 &> /dev/null; then
    echo "⚠️  sqlite3 command not found. Cannot automatically check DB."
    echo "   Please run: sqlite3 $DB_PATH 'SELECT count(*) FROM http_transactions;'"
    exit 0
fi

# Query the latest transaction using LIKE to handle trailing slashes
COUNT=$(sqlite3 $DB_PATH "SELECT count(*) FROM http_transactions WHERE req_url LIKE '$TARGET_URL%';")

if [ "$COUNT" -gt 0 ]; then
    echo "✅ Database Verification Successful!"
    echo "   Found $COUNT transaction(s) match for $TARGET_URL"
    
    echo "   Latest Transaction Details:"
    sqlite3 -header -column $DB_PATH "SELECT request_id, req_method, req_url, res_status, datetime(req_timestamp, 'unixepoch') as time FROM http_transactions WHERE req_url LIKE '$TARGET_URL%' ORDER BY req_timestamp DESC LIMIT 1;"
else
    echo "❌ NO Transaction found in Database!"
    echo "   Please check if Orchestrator is running and Agent is connected."
fi
