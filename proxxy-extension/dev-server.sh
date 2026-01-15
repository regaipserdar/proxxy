#!/bin/bash
# Proxxy Extension Development Server

set -e

echo "=== Proxxy Extension Dev Server ==="

# Check if we're in the right directory
if [[ ! -d "extensions/proxxy-chrome" ]]; then
    echo "Hata: extensions/proxxy-chrome dizininde değilsiniz"
    echo "Önce: cd extensions/proxxy-chrome"
    exit 1
fi

cd extensions/proxxy-chrome

# Install dependencies if needed
if [[ ! -d "node_modules" ]]; then
    echo "Dependencies yükleniyor..."
    npm install
fi

# Development server'i başlat
echo ""
echo "🚀 Development server başlatılıyor..."
echo "📂 Extension build ediliyor: dist/"
echo "🔄 Hot reload aktif"
echo "🛠️ Local config UI aktif"
echo ""
echo "Extension kurulumu:"
echo "1. Chrome'da chrome://extensions/ aç"
echo "2. 'Load unpacked' butonuna tıkla"
echo "3. extensions/proxxy-chrome/dist/ klasörünü seç"
echo "4. Extension'ı yenile (Ctrl+R veya reload butonu)"
echo ""
echo "Hot reload: Kod değişikliklerinde otomatik yenileme"
echo "Local Config: Popup'da hızlı ayarlar"
echo "Server durdurmak için: Ctrl+C"
echo ""

# Start dev server with watch
npm run dev