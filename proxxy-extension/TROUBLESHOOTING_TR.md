# Proxxy Chrome Extension - Native Host Bağlantı Sorunu Çözümü

## 🚨 Mevcut Sorun
Extension sürekli olarak bağlanma/koparma döngüsünde kalıyor:
```
Disconnected from Proxxy native host
Attempting to reconnect in 1000ms (attempt 1)
Connected to Proxxy native host
Connected to native host: com.proxxy.native
Native host disconnected
```

## 🔍 Nedenler ve Çözümleri

### 1. ✅ Proxxy Kurulum Kontrolü
**Sorun:** Proxxy backend kurulu değil  
**Çözüm:**
```bash
# Kurulu mu kontrol et
proxxy --version

# macOS
brew install proxxy

# Linux
sudo apt-get install proxxy

# Windows
# https://github.com/anomalyco/proxxy/releases
```

### 2. 🆔 Extension ID'sini Doğru Ayarla
**Sorun:** Manifest dosyasında yanlış extension ID  
**Çözüm:**
1. Chrome'da `chrome://extensions/` aç
2. Proxxy extension'ı bul
3. "Details" butonuna tıkla
4. Extension ID'sini kopyala (örn: `abc123def456`)
5. ID'yi manifest'e gir

### 3. 📄 Native Host Manifest Kurulumu
**Otomatik Kurulum (Önerilen):**
```bash
# Kurulum script'ini çalıştır
./install-native-host.sh
# Extension ID'si istendiğinde gir
```

**Manuel Kurulum:**

#### macOS:
```bash
mkdir -p ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts
cat > ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/com.proxxy.native.json << EOF
{
  "name": "com.proxxy.native",
  "description": "Proxxy Native Messaging Host",
  "path": "/usr/local/bin/proxxy-native-host",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://GERÇEK_EXTENSION_ID/"
  ]
}
EOF
```

#### Linux:
```bash
mkdir -p ~/.config/google-chrome/NativeMessagingHosts
cat > ~/.config/google-chrome/NativeMessagingHosts/com.proxxy.native.json << EOF
{
  "name": "com.proxxy.native",
  "description": "Proxxy Native Messaging Host", 
  "path": "/usr/local/bin/proxxy-native-host",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://GERÇEK_EXTENSION_ID/"
  ]
}
EOF
```

#### Windows (Admin Command Prompt):
```batch
REG ADD "HKCU\Software\Google\Chrome\NativeMessagingHosts\com.proxxy.native" /ve REG_SZ /t REG_SZ /d "{\"name\":\"com.proxxy.native\",\"description\":\"Proxxy Native Messaging Host\",\"path\":\"C:\\Program Files\\Proxxy\\proxxy-native-host.exe\",\"type\":\"stdio\",\"allowed_origins\":[\"chrome-extension://GERÇEK_EXTENSION_ID/\"]}"
```

### 4. 🧪 Bağlantı Test Etme
**Debug Script ile Test:**
```bash
python3 /Users/rooter/Documents/proxxy/proxxy-extension/debug-native-host.py
```

**Extension Debug Console'da Test:**
1. F12 → Console tab
2. Aşağıdaki kodu yapıştır:
```javascript
chrome.runtime.sendMessage({
  action: 'check_connection'
}).then(response => {
  console.log('Bağlantı testi:', response);
});
```

### 5. 🔧 Geliştirilmiş Extension Özellikleri
Extension'a aşağıdaki iyileştirmeler eklendi:

#### ✨ Akıllı Bağlantı Yönetimi
- Geliştirilmiş hata ayıklama logları
- Daha iyi yeniden bağlantı mantığı
- Connection state takibi

#### 🛠️ Detaylı Hata Mesajları
- `[NativeHost]` prefix ile loglar
- Hangi adımda hata olduğu belirgin
- Timeout ve bağlantı sorunları ayrıştırıldı

#### 📋 Configuration Page
- Server URL configuration
- Native host path ayarı
- Bağlantı test butonu
- Platform-specific path detection

### 6. 🚀 Hızlı Çözüm Adımları
**En hızlı çözüm için:**

1. **Extension ID'sini öğren**
   ```
   chrome://extensions/ → Proxxy → Details → ID'yi kopyala
   ```

2. **Otomatik kurulum script'ini çalıştır**
   ```bash
   ./install-native-host.sh
   # Extension ID'sini gir
   ```

3. **Chrome'u yeniden başlat**
   
4. **Test et**
   - Extension popup'ı aç
   - "Check Connection" butonuna tıkla
   - Console'da sonuçları kontrol et

### 7. 📞 Destek
Eğer sorun devam ederse:
1. Console'daki hata loglarını kontrol et
2. Proxxy kurulumunu doğrula
3. Extension'i yeniden yükle
4. Debug script'i çalıştır

**Console'da aranacak loglar:**
- `[Background]` - Background script mesajları
- `[NativeHost]` - Native host mesajları
- Hata mesajları ve detayları

## ✅ Başarlı Kurulum Belirtileri
- ✅ Extension popup'ında "Connected" yazısı
- ✅ Console'da hata mesajı yok
- ✅ DevTools panel açılıyor
- ✅ HAR/LSR butonları çalışıyor

Bu adımları izleyerek native host bağlantı sorunlarını çözebilirsiniz! 🎯