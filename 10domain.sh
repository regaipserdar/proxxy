#!/bin/bash

PROXY="http://127.0.0.1:9095"
USER_AGENT="Proxxy-Scanner/2.1"
STATFILE="/tmp/proxxy_stats.txt"
> "$STATFILE"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

rand() { echo $((RANDOM % $1)); }

banner() {
echo -e "${BLUE}
██████╗ ██████╗  ███████╗██╗  ██╗██╗  ██╗██╗   ██╗
██╔══██╗██╔══██╗██╔════╝╚██╗██╔╝╚██╗██╔╝╚██╗ ██╔╝
██████╔╝██████╔╝█████╗   ╚███╔╝  ╚███╔╝  ╚████╔╝ 
██╔═══╝ ██╔══██╗██╔══╝   ██╔██╗  ██╔██╗   ╚██╔╝  
██║     ██║  ██║███████╗██╔╝ ██╗██╔╝ ██╗   ██║   
╚═╝     ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   

           P R O X X Y   M I T M   S C A N N E R testv1
${NC}"
}

send_request() {
    method=$1
    url=$2
    data=$3

    code=$(curl -s -o /dev/null -w "%{http_code}" -x "$PROXY" -k \
        -X "$method" \
        -H "User-Agent: $USER_AGENT" \
        -H "Content-Type: application/json" \
        ${data:+-d "$data"} \
        "$url")

    echo "$code" >> "$STATFILE"

    if echo "$code" | grep -q "^2"; then
        echo -e "${GREEN}[✓] $method $url -> $code${NC}"
    elif [ "$code" = "000" ]; then
        echo -e "${RED}[X] $method $url -> NO CONNECTION${NC}"
    else
        echo -e "${YELLOW}[!] $method $url -> $code${NC}"
    fi

    sleep 0.$((RANDOM % 6 + 2))
}

fuzz_domain() {
    base=$1; shift
    endpoints=("$@")

    echo -e "\n${BLUE}### TARGET: $base (${#endpoints[@]} endpoints) ###${NC}"

    count=$((RANDOM % 6 + 10)) # 10–15

    i=0
    while [ $i -lt $count ]; do
        ep=${endpoints[$(rand ${#endpoints[@]})]}

        case $((RANDOM % 4)) in
            0) m="GET";;
            1) m="POST";;
            2) m="PUT";;
            3) m="DELETE";;
        esac

        payload='{"scan":"proxxy","id":'$RANDOM'}'
        send_request "$m" "$base$ep" "$payload"
        i=$((i+1))
    done
}

banner

fuzz_domain "http://testphp.vulnweb.com" \
"/" "/login.php" "/listproducts.php?cat=1" "/search.php?xss=<script>" \
"/cart.php" "/userinfo.php" "/guestbook.php" "/admin/" "/phpinfo.php"

fuzz_domain "https://httpbin.org" \
"/get" "/post" "/delay/2" "/status/403" "/status/500" "/anything" "/headers" "/cookies"

fuzz_domain "https://reqres.in" \
"/api/users" "/api/login" "/api/register" "/api/unknown" "/api/users/2"

fuzz_domain "https://jsonplaceholder.typicode.com" \
"/posts" "/posts/1" "/comments" "/albums" "/photos"

fuzz_domain "https://scanme.nmap.org" \
"/" "/robots.txt"

echo -e "\n${BLUE}######## RESPONSE CODE REPORT ########${NC}"
sort "$STATFILE" | uniq -c

echo -e "\n${GREEN}🎯 Scan Simulation Completed${NC}"
