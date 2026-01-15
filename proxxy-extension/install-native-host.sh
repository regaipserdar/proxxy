# 🛠️ Proxxy Extension - Native Host Bağlantı Sorunu Çözümü

## 🚨 Sorun Tanımı
Extension sürekli olarak native host'a bağlanmaya çalışıyor ve hemen kopuyor:
```
Disconnected from Proxxy native host
Attempting to reconnect in 1000ms (attempt 1)
Connected to Proxxy native host
Connected to native host: com.proxxy.native
Native host disconnected
```

## 🔍 Muhtemel Nedenler

### 1. Native Host Kurulu Değil
- Proxxy backend kurulu değil
- Native host binary dosyası mevcut değil

### 2. Manifest Dosyası Eksik veya Yanlış
- Chrome native manifest dosyası kayıtlı değil
- Extension ID'si yanlış

### 3. İzin Problemleri
- Native host çalışma izni yok
- Dosya yolu erişim problemi

### 4. Platform Uyumsuzluğu
- Wrong native host for current OS
- Binary architecture mismatch (32/64 bit)

## 📋 Çözüm Adımları

### Adım 1: Proxxy Kurulumunu Kontrol Et
```bash
# Proxxy kurulu mu kontrol et
which proxxy
proxxy --version

# Eğer kurulu değilse, kur:
# macOS (Homebrew)
brew install proxxy

# Linux (APT)
sudo apt-get install proxxy

# Windows
# Download from https://github.com/anomalyco/proxxy/releases
```

### Adım 2: Extension ID'sini Al
1. Chrome'da `chrome://extensions/` aç
2. Proxxy extension'ı bul
3. "Details" butonuna tıkla
4. Extension ID'sini kopyala (örnek: `abc123def456`)

### Adım 3: Native Host Manifest'i Oluştur

#### macOS:
```bash
# Dizin oluştur
mkdir -p ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts

# Manifest oluştur ve düzenle
cat > ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/com.proxxy.native.json << EOF
{
  "name": "com.proxxy.native",
  "description": "Proxxy Native Messaging Host",
  "path": "/usr/local/bin/proxxy-native-host",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID/"
  ]
}
EOF

# YOUR_EXTENSION_ID'yi gerçek ID ile değiştir
```

#### Linux:
```bash
# Dizin oluştur
mkdir -p ~/.config/google-chrome/NativeMessagingHosts

# Manifest oluştur
cat > ~/.config/google-chrome/NativeMessagingHosts/com.proxxy.native.json << EOF
{
  "name": "com.proxxy.native",
  "description": "Proxxy Native Messaging Host",
  "path": "/usr/local/bin/proxxy-native-host",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID/"
  ]
}
EOF
```

#### Windows:
```batch
REM Registry kaydı oluştur
REG ADD "HKCU\Software\Google\Chrome\NativeMessagingHosts\com.proxxy.native" /ve REG_SZ /t REG_SZ /d "{\"name\":\"com.proxxy.native\",\"description\":\"Proxxy Native Messaging Host\",\"path\":\"C:\\Program Files\\Proxxy\\proxxy-native-host.exe\",\"type\":\"stdio\",\"allowed_origins\":[\"chrome-extension://YOUR_EXTENSION_ID/\"]}"
```

### Adım 4: Debug Modunda Test Et

#### Debug Script ile Test:
```bash
# Debug script çalıştır
python3 /Users/rooter/Documents/proxxy/proxxy-extension/debug-native-host.py
```

Chrome'da test için:
```javascript
// Console'da çalıştır:
chrome.runtime.sendMessage({
  action: 'check_connection'
}).then(response => {
  console.log('Connection test:', response);
});
```

### Adım 5: Manuel Kurulum Script'i

Manuel kurulum için script oluştur:
<tool_call>write
<arg_key>content</arg_key>
<arg_value>#!/bin/bash
# Proxxy Extension Native Host Kurulum Script'i

set -e

echo "=== Proxxy Native Host Kurulum ==="

# Extension ID'sini al
echo "Extension ID'sini girin (chrome://extensions/'den bulabilirsiniz):"
read -p "Extension ID: " EXTENSION_ID

if [[ -z "$EXTENSION_ID" ]]; then
    echo "Hata: Extension ID gereklidir"
    exit 1
fi

# Platformu tespit et
PLATFORM=$(uname -s)
case $PLATFORM in
    Darwin*)
        echo "macOS detected"
        HOST_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
        HOST_PATH="/usr/local/bin/proxxy-native-host"
        ;;
    Linux*)
        echo "Linux detected"
        HOST_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
        HOST_PATH="/usr/local/bin/proxxy-native-host"
        ;;
    *)
        echo "Unsupported platform: $PLATFORM"
        exit 1
        ;;
esac

# Dizin oluştur
mkdir -p "$HOST_DIR"

# Manifest oluştur
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

echo "Manifest oluşturuldu: $HOST_DIR/com.proxxy.native.json"
echo "Extension ID: $EXTENSION_ID"

# İzinleri kontrol et
chmod 644 "$HOST_DIR/com.proxxy.native.json"

# Native host path kontrol et
if [[ ! -f "$HOST_PATH" ]]; then
    echo "UYARI: Native host binary bulunamadı: $HOST_PATH"
    echo "Proxxy kurulu olduğundan emin olun"
else
    echo "Native host bulundu: $HOST_PATH"
fi

echo ""
echo "Kurulum tamamlandı!"
echo "Chrome'u yeniden başlatın"
echo ""
echo "Test için:"
echo "1. Proxxy extension'ı açın"
echo "2. Popup'da 'Check Connection' butonuna tıklayın"
echo "3. Console'da hataları kontrol edin"

# Debug test
echo ""
echo "Debug test (isteğe bağlı):"
echo "python3 /Users/rooter/Documents/proxxy/proxxy-extension/debug-native-host.py"