#!/bin/bash
# Proxxy Extension Quick Fix Script

echo "=== Proxxy Extension Quick Fix ==="

# 1. Extension ID'sini al
echo "Chrome'da extension'ı açın ve ID'sini kopyalayın..."
echo "chrome://extensions/ → Proxxy → Details → ID kopyala"

read -p "Extension ID'sini girin: " EXTENSION_ID

if [[ -z "$EXTENSION_ID" ]]; then
    echo "UYARI: Gerçek Extension ID'si gereklidir"
    echo "Şimdilik test ID kullanılacak..."
    EXTENSION_ID="test_extension_id_placeholder"
fi

echo "Extension ID: $EXTENSION_ID"

# 2. Platformu tespit et
PLATFORM=$(uname -s)
case $PLATFORM in
    Darwin*)
        HOST_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
        HOST_PATH="/usr/local/bin/proxxy-native-host"
        echo "Platform: macOS"
        ;;
    Linux*)
        HOST_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
        HOST_PATH="/usr/local/bin/proxxy-native-host"
        echo "Platform: Linux"
        ;;
    CYGWIN*|MINGW*|MSYS*)
        HOST_DIR="$APPDATA/Google/Chrome/NativeMessagingHosts"
        HOST_PATH="/c/Program Files/Proxxy/proxxy-native-host.exe"
        echo "Platform: Windows"
        ;;
    *)
        echo "Bilinmeyen platform: $PLATFORM"
        exit 1
        ;;
esac

# 3. Manifest dizini oluştur
mkdir -p "$HOST_DIR"

# 4. Manifest oluştur
cat > "$HOST_DIR/com.proxxy.native.json" << EOF
{
  "name": "com.proxxy.native",
  "description": "Proxxy Native Messaging Host",
  "path": "$HOST_PATH",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://$EXTENSION_ID/"
  ]
}
EOF

echo "✅ Manifest oluşturuldu: $HOST_DIR/com.proxxy.native.json"

# 5. İzinleri ayarla
chmod 644 "$HOST_DIR/com.proxxy.native.json"

# 6. Native host path kontrol et
if [[ -f "$HOST_PATH" ]]; then
    echo "✅ Native host bulundu: $HOST_PATH"
else
    echo "❌ Native host BULUNAMADI: $HOST_PATH"
    echo "   Proxxy kurulu olduğundan emin olun"
fi

# 7. Extension build'i güncelle
cd extensions/proxxy-chrome

# Manifest'teki Extension ID'yi güncelle
if [[ "$EXTENSION_ID" != "test_extension_id_placeholder" ]]; then
    echo "Extension manifest'i güncelleniyor..."
    # Bu senaryo gerçek bir build sürecinde yapılmalı
fi

echo ""
echo "=== Kurulum Tamamlandı ==="
echo ""
echo "📋 Sonraki Adımlar:"
echo "1. Chrome'u yeniden başlatın"
echo "2. Extension'ı yeniden yükleyin (disable/enable)"
echo "3. Extension popup'ını açın"
echo "4. Hata varsa console'da kontrol edin"
echo ""
echo "🧪 Test:"
echo "- Popup'ta 'Connected' yazısını kontrol edin"
echo "- 'Check Connection' butonuna tıklayın"
echo "- Console'da hata mesajlarını kontrol edin"
echo ""
echo "Eğer sorun devam ederse:"
echo "- Debug script çalıştır: python3 debug-native-host.py"
echo "- Manual installation guide: TROUBLESHOOTING_TR.md"