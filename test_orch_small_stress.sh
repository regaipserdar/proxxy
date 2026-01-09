#!/bin/bash

# ==========================================
# CONFIGURATION
# ==========================================
PROXY_HOST="127.0.0.1"
PROXY_PORT="9095"
TARGET_URL="http://example.com"
DB_PATH="./proxxy.db"

# STRESS TEST SETTINGS
TOTAL_REQUESTS=100    # Toplam atılacak istek sayısı
CONCURRENCY=10        # Aynı anda (paralel) atılacak istek sayısı

# ==========================================

echo "🚀 Proxxy Orchestrator Test & Stress Tool"
echo "----------------------------------------"
echo "Target: $PROXY_HOST:$PROXY_PORT"
echo "DB:     $DB_PATH"
echo "Load:   $TOTAL_REQUESTS requests ($CONCURRENCY parallel)"
echo "----------------------------------------"

# 1. FUNCTIONAL CHECK (Single Request)
echo ""
echo "1️⃣  Functional Check (Single Request)..."
RESPONSE=$(curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w "%{http_code}" $TARGET_URL)

if [ "$RESPONSE" -eq 200 ]; then
    echo "   ✅ Request successful (HTTP 200)"
else
    echo "   ❌ Request failed (HTTP $RESPONSE)"
    echo "      Aborting stress test."
    exit 1
fi

# Check for sqlite3
if ! command -v sqlite3 &> /dev/null; then
    echo "⚠️  sqlite3 command not found. Install it to verify DB records."
    exit 1
fi

# 2. CAPTURE INITIAL DB STATE
echo ""
echo "2️⃣  Snapshotting Database State..."
INITIAL_COUNT=$(sqlite3 $DB_PATH "SELECT count(*) FROM http_transactions;")
echo "   Initial DB Record Count: $INITIAL_COUNT"

# 3. STRESS TEST
echo ""
echo "3️⃣  🔥 Starting Stress Test ($TOTAL_REQUESTS requests)..."

START_TIME=$(date +%s.%N)

# Using xargs to run curls in parallel
# seq generates numbers, xargs -P runs them in parallel
# -s: silent, -o /dev/null: ignore body, -w: print http code
seq 1 $TOTAL_REQUESTS | xargs -n1 -P$CONCURRENCY -I {} bash -c "curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w '%{http_code}\n' $TARGET_URL" > results.txt

END_TIME=$(date +%s.%N)

# Calculate Duration
DURATION=$(echo "$END_TIME - $START_TIME" | bc)
RPS=$(echo "$TOTAL_REQUESTS / $DURATION" | bc)

echo "   ✅ Stress Test Completed in $(printf "%.2f" $DURATION) seconds."
echo "   ⚡ Performance: ~$RPS Requests/Sec"

# 4. ANALYZE RESULTS
echo ""
echo "4️⃣  Analyzing Results..."

# Count HTTP 200s
SUCCESS_COUNT=$(grep -c "200" results.txt)
FAIL_COUNT=$((TOTAL_REQUESTS - SUCCESS_COUNT))

if [ "$FAIL_COUNT" -eq 0 ]; then
    echo "   ✅ HTTP Requests: All $SUCCESS_COUNT succeeded."
else
    echo "   ⚠️  HTTP Requests: $SUCCESS_COUNT success, $FAIL_COUNT FAILED."
fi

# Cleanup temp file
rm results.txt

# 5. DB CONSISTENCY CHECK
echo ""
echo "5️⃣  Verifying Database Integrity..."
FINAL_COUNT=$(sqlite3 $DB_PATH "SELECT count(*) FROM http_transactions;")
ACTUAL_ADDED=$((FINAL_COUNT - INITIAL_COUNT))

echo "   Initial Count: $INITIAL_COUNT"
echo "   Final Count:   $FINAL_COUNT"
echo "   ---------------------"
echo "   Expected New Records: $TOTAL_REQUESTS"
echo "   Actual New Records:   $ACTUAL_ADDED"

if [ "$ACTUAL_ADDED" -eq "$TOTAL_REQUESTS" ]; then
    echo "   ✅ DATABASE INTEGRITY CONFIRMED! (No dropped logs)"
elif [ "$ACTUAL_ADDED" -lt "$TOTAL_REQUESTS" ]; then
    MISSING=$((TOTAL_REQUESTS - ACTUAL_ADDED))
    echo "   ❌ DATA LOSS DETECTED: $MISSING records are missing from DB."
    echo "      Possible Causes:"
    echo "      1. SQLite Locking (SQLITE_BUSY) - Too many concurrent writes."
    echo "      2. Rust Channel Buffer Overflow (Consumer too slow)."
    echo "      3. Proxy Agent crashed or dropped packets."
else
    # This rarely happens unless traffic came from elsewhere
    echo "   ⚠️  More records found than expected ($ACTUAL_ADDED). External traffic?"
fi

echo ""