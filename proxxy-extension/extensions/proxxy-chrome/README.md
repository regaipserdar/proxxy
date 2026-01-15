# Proxxy Chrome Extension

A Chrome browser extension that provides in-browser controls for Proxxy HAR recording and LSR (Login Sequence Recorder) functionality.

## Features

### HAR Recording
- Start/Stop/Clear HAR recording directly from DevTools
- Real-time request counter and statistics
- Configurable filters (domain, resource type)
- One-click HAR file download

### LSR Recording
- Login sequence recording from browser
- Profile management (save, load, delete)
- Replay functionality with progress tracking
- Step-by-step visualization

### Quick Access
- Toolbar popup for status overview
- Quick action buttons
- Connection status indicator
- Direct DevTools panel access

## Architecture

```
┌─────────────────────┐         ┌──────────────────────┐
│  Chrome Extension   │ ◄─────► │   Proxxy Backend     │
│  (UI Controls)      │  Native │   (Business Logic)   │
│                     │  Msg    │                      │
│  - DevTools Panel   │         │  - HAR Manager       │
│  - Popup/Toolbar    │         │  - LSR Recorder      │
│  - Background SW    │         │  - Session Manager   │
└─────────────────────┘         └──────────────────────┘
```

## Installation

### Development

1. Clone this repository
2. Install dependencies:
   ```bash
   cd extensions/proxxy-chrome
   npm install
   ```

3. Build the extension:
   ```bash
   npm run build
   ```

4. Load in Chrome:
   - Open `chrome://extensions/`
   - Enable "Developer mode"
   - Click "Load unpacked"
   - Select the `dist` folder

### Production

1. Build the extension:
   ```bash
   npm run build
   ```

2. Zip the `dist` folder for distribution
3. Submit to Chrome Web Store or distribute as unpacked extension

## Native Host Setup

The extension requires the Proxxy native messaging host to be installed:

1. Ensure Proxxy is installed on your system
2. Run the native host installation:
   ```bash
   proxxy extension install
   ```

3. The native host manifest will be automatically registered

## Configuration

Before using the extension, you'll want to configure server settings:

1. Right-click the Proxxy extension icon in your toolbar
2. Select "Options" from the context menu
3. **Server Configuration**:
   - Set your Proxxy server URL (e.g., `http://localhost:8080`)
   - Add an API key if your server requires authentication
   - Configure connection timeout (5-60 seconds)
   - Test the connection with the provided button
4. **Native Host Configuration**:
   - Auto-detects native host path on Windows, macOS, and Linux
   - Enable auto-connect for seamless integration
5. **Recording Settings**:
   - Set default HAR filters
   - Configure auto-download for HAR files
   - Choose LSR profile storage location
6. **UI Settings**:
   - Select your preferred theme (Dark/Light/Auto)
   - Enable desktop notifications
   - Enable compact mode for smaller screens

## Usage

### HAR Recording

1. Open Chrome DevTools on any page
2. Click "Proxxy" tab
3. Configure filters if needed
4. Click "Start Recording"
5. Browse the web - requests will be captured in real-time
6. Click "Stop" when finished
7. Download HAR file for analysis

### LSR Recording

1. Navigate to the login page you want to record
2. Open Proxxy DevTools panel
3. Switch to "LSR Recorder" tab
4. Enter a profile name
5. Click "Start Recording"
6. Perform the login sequence step-by-step
7. Click "Stop Recording"
8. Save the profile for later replay
9. Use the "Profiles" tab to manage and replay saved sequences

### Quick Actions (Toolbar Popup)

- View current connection status
- Quick start/stop for HAR and LSR recording
- Access full DevTools panel
- Open configuration options

## Development

### Project Structure

```
src/
├── background/          # Service worker and native messaging
│   ├── index.ts         # Main background script
│   ├── native-host.ts   # Native messaging client
│   └── state.ts         # State management
├── devtools/            # DevTools panel
│   ├── devtools.ts      # DevTools entry point
│   ├── panel.ts         # Main panel controller
│   ├── har-panel.ts     # HAR recording UI
│   ├── lsr-panel.ts     # LSR recording UI
│   ├── panel.html       # Panel HTML
│   └── panel.css        # Panel styles
├── popup/               # Toolbar popup
│   ├── popup.ts         # Popup controller
│   ├── popup.html       # Popup HTML
│   └── popup.css        # Popup styles
└── manifest.json        # Extension manifest
```

### Building

```bash
# Development build with watch
npm run dev

# Production build
npm run build

# Type checking
npm run typecheck

# Linting
npm run lint
```

## Native Messaging Protocol

The extension communicates with Proxxy via Chrome's native messaging protocol:

### Message Format

```typescript
interface NativeMessage {
  id: string
  module: 'har' | 'lsr'
  action: string
  payload?: any
  timestamp: number
}
```

### Supported Commands

#### HAR Module
- `har_start` - Start recording
- `har_stop` - Stop recording
- `har_clear` - Clear buffer
- `har_export` - Download HAR file
- `har_status` - Get current state

#### LSR Module
- `lsr_record_start` - Begin recording
- `lsr_record_stop` - End recording
- `lsr_replay` - Execute profile
- `lsr_list_profiles` - Get saved profiles
- `lsr_delete_profile` - Remove profile

## Permissions

The extension requires the following permissions:

- `debugger` - For DevTools integration
- `tabs` - For current tab context
- `storage` - For settings and state persistence
- `nativeMessaging` - For Proxxy communication
- `downloads` - For HAR file downloads
- `<all_urls>` - For comprehensive request capture

## Security Considerations

- Extension ID is validated in native host manifest
- All messages from extension are validated on the backend
- No sensitive data is stored in extension storage
- Native communication limited to stdio protocol

## Troubleshooting

### Extension shows "Disconnected"
1. Ensure Proxxy is running
2. Check native host installation: `proxxy extension install`
3. Restart Chrome and reload the extension

### DevTools panel not appearing
1. Open DevTools (F12 or Ctrl+Shift+I)
2. Look for "Proxxy" tab
3. If missing, reload the extension and reopen DevTools

### HAR recording not working
1. Check native host connection
2. Verify Proxxy HAR module is enabled
3. Check browser console for error messages

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the same license as Proxxy.

## Support

For issues and questions:
- GitHub Issues: [Proxxy Issues](https://github.com/anomalyco/proxxy/issues)
- Documentation: [Proxxy Docs](https://docs.proxxy.dev)