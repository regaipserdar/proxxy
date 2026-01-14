#!/bin/bash

# Renkler
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}##################################################${NC}"
echo -e "${BLUE}#      Proxxy Memory & Process Monitor           #${NC}"
echo -e "${BLUE}##################################################${NC}"

# 1. PID'leri bul (Virgülle ayrılmış liste formatında: 1234,5678)
# -f: Komut satırının tamamında ara (binary path vs dahil)
# -d ",": htop'un istediği virgül formatında birleştir
PIDS=$(pgrep -d "," -f "orchestrator|proxy-agent")

# 2. Kontrol et: Çalışıyorlar mı?
if [ -z "$PIDS" ]; then
    echo -e "${RED}[!] HATA: Orchestrator veya Proxy Agent çalışmıyor!${NC}"
    echo "Lütfen önce projeyi başlatın."
    exit 1
fi

# 3. Bilgi ver ve htop'u başlat
echo -e "${GREEN}[*] Hedef PID'ler bulundu: ${PIDS}${NC}"
echo -e "HTOP başlatılıyor... (Çıkmak için F10 veya q)"
sleep 1

# Sadece bu PID'leri gösteren htop'u aç
# --pid: Sadece belirtilen processleri göster
htop -p "$PIDS"