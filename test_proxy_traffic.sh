#!/bin/bash

# Configuration
PROXY_HOST="127.0.0.1"
PROXY_PORT="9095"
TARGET_URL="http://testphp.vulnweb.com/AJAX/index.php"


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


