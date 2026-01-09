#!/bin/bash

echo "🚀 Multi-Agent Load Test Starting..."
echo "📊 Testing 5 agents with 50000 requests each (250,000 total)"
echo "⏱️  Start time: $(date)"
echo ""

# Agent 1'i vur (Arka plana at)
echo "🔥 Agent-1 (Port 9095) starting..."
oha -n 50000 -c 10 -x http://127.0.0.1:9095 http://127.0.0.1:8000 > result_agent1.txt &
PID1=$!

# Agent 2'yi vur (Arka plana at)
echo "🔥 Agent-2 (Port 9096) starting..."
oha -n 50000 -c 10 -x http://127.0.0.1:9096 http://127.0.0.1:8000 > result_agent2.txt &
PID2=$!

# Agent 3'ü vur (Arka plana at)
echo "🔥 Agent-3 (Port 9097) starting..."
oha -n 50000 -c 10 -x http://127.0.0.1:9097 http://127.0.0.1:8000 > result_agent3.txt &
PID3=$!

# Agent 4'ü vur (Arka plana at)
echo "🔥 Agent-4 (Port 9098) starting..."
oha -n 50000 -c 10 -x http://127.0.0.1:9098 http://127.0.0.1:8000 > result_agent4.txt &
PID4=$!

# Agent 5'i vur (Arka plana at)
echo "🔥 Agent-5 (Port 9099) starting..."
oha -n 50000 -c 10 -x http://127.0.0.1:9099 http://127.0.0.1:8000 > result_agent5.txt &
PID5=$!

echo ""
echo "⏳ Waiting for all tests to complete..."

# İkisinin bitmesini bekle
wait $PID1 $PID2 $PID3 $PID4 $PID5

echo ""
echo "✅ All tests completed!"
echo "⏱️  End time: $(date)"
echo ""
echo "=" | tr '=' '=' | head -c 80; echo ""
echo "📊 RESULTS SUMMARY"
echo "=" | tr '=' '=' | head -c 80; echo ""

# Sonuçları ekrana bas
echo ""
echo "=== AGENT 1 RESULTS (Port 9095) ==="
grep "Success rate" result_agent1.txt
grep "Requests/sec" result_agent1.txt
grep "Average:" result_agent1.txt

echo ""
echo "=== AGENT 2 RESULTS (Port 9096) ==="
grep "Success rate" result_agent2.txt
grep "Requests/sec" result_agent2.txt
grep "Average:" result_agent2.txt

echo ""
echo "=== AGENT 3 RESULTS (Port 9097) ==="
grep "Success rate" result_agent3.txt
grep "Requests/sec" result_agent3.txt
grep "Average:" result_agent3.txt

echo ""
echo "=== AGENT 4 RESULTS (Port 9098) ==="
grep "Success rate" result_agent4.txt
grep "Requests/sec" result_agent4.txt
grep "Average:" result_agent4.txt

echo ""
echo "=== AGENT 5 RESULTS (Port 9099) ==="
grep "Success rate" result_agent5.txt
grep "Requests/sec" result_agent5.txt
grep "Average:" result_agent5.txt

echo ""
echo "=" | tr '=' '=' | head -c 80; echo ""

# Database verification
echo ""
echo "📦 DATABASE VERIFICATION"
echo "=" | tr '=' '=' | head -c 80; echo ""
DB_COUNT=$(sqlite3 proxxy.db "SELECT COUNT(*) FROM http_transactions;")
echo "Total transactions in database: $DB_COUNT"
echo "Expected: ~250,000 (5 agents × 50,000 requests)"

if [ "$DB_COUNT" -gt 200000 ]; then
    echo "✅ Database integrity: GOOD (>80% saved)"
else
    echo "⚠️  Database integrity: Some requests may have been dropped"
fi

echo ""
echo "🗑️  Cleaning up result files..."
# Geçici dosyaları temizle
rm result_agent1.txt result_agent2.txt result_agent3.txt result_agent4.txt result_agent5.txt

echo "✅ Test complete!"
