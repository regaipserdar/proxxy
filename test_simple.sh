#!/bin/bash

echo "Testing single endpoint..."

URL="http://localhost:8000/"
HTTP_CODE=$(curl -x http://127.0.0.1:9095 -s -o /dev/null -w "%{http_code}" "$URL" 2>/dev/null)

echo "URL: $URL"
echo "HTTP Code: [$HTTP_CODE]"

if [[ "$HTTP_CODE" =~ ^[2-4] ]]; then
    echo "✅ SUCCESS"
else
    echo "❌ FAILED (code: $HTTP_CODE)"
fi
