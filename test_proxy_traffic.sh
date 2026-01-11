#!/bin/bash

# ==========================================
# CONFIGURATION
# ==========================================
PROXY_HOST="127.0.0.1"
PROXY_PORT="9095"
TARGET_BASE="http://testphp.vulnweb.com"

# Renkler
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Proxxy Traffic Generator${NC}"
echo "----------------------------------------"
echo "Proxy:  $PROXY_HOST:$PROXY_PORT"
echo "Target: $TARGET_BASE"
echo "----------------------------------------"

# İstek Gönderme Fonksiyonu
send_request() {
    METHOD=$1
    URL_PATH=$2   # BURAYI DUZELTTIM (PATH -> URL_PATH)
    DESC=$3
    DATA=$4
    
    URL="$TARGET_BASE$URL_PATH"
    
    echo -ne "Sending ${YELLOW}$METHOD${NC} $URL_PATH ($DESC)... "
    
    if [ "$METHOD" == "POST" ]; then
        HTTP_CODE=$(curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w "%{http_code}" -X POST -d "$DATA" "$URL")
    elif [ "$METHOD" == "PUT" ]; then
        HTTP_CODE=$(curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w "%{http_code}" -X PUT -d "$DATA" "$URL")
    elif [ "$METHOD" == "DELETE" ]; then
        HTTP_CODE=$(curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w "%{http_code}" -X DELETE "$URL")
    else
        # Default GET
        HTTP_CODE=$(curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w "%{http_code}" "$URL")
    fi

    if [[ "$HTTP_CODE" =~ ^2 ]]; then
        echo -e "[${GREEN}$HTTP_CODE${NC}] ✅"
    elif [[ "$HTTP_CODE" =~ ^3 ]]; then
        echo -e "[${BLUE}$HTTP_CODE${NC}] ➡️"
    elif [[ "$HTTP_CODE" =~ ^4 ]]; then
        echo -e "[${YELLOW}$HTTP_CODE${NC}] ⚠️"
    else
        echo -e "[${RED}$HTTP_CODE${NC}] ❌"
    fi
    
    sleep 0.5 
}

# ==========================================
# SCENARIOS
# ==========================================

send_request "GET" "/" "Homepage Visit"
send_request "GET" "/search.php?test=query" "Search Query"
send_request "POST" "/userinfo.php" "Login Attempt" "uname=test&pass=test"
send_request "GET" "/non_existent_page_123" "404 Check"
send_request "GET" "/AJAX/index.php" "AJAX Endpoint"
send_request "POST" "/api/mock" "API JSON Post" '{"action":"test","id":1}'
send_request "PUT" "/profile/update" "Update Profile" "email=test@example.com"
send_request "DELETE" "/items/15" "Delete Item"

echo "----------------------------------------"
echo -e "${GREEN}✅ Test batch completed! Check your Proxxy Dashboard.${NC}"