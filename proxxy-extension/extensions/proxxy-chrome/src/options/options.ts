import browser from 'webextension-polyfill'

interface ExtensionSettings {
  server: {
    url: string
    apiKey: string
    timeout: number
  }
  nativeHost: {
    path: string
    autoConnect: boolean
  }
  recording: {
    defaultHARFilter: string
    autoDownloadHAR: boolean
    lsrProfilePath: string
  }
  ui: {
    theme: string
    showNotifications: boolean
    compactMode: boolean
  }
  advanced: {
    logLevel: string
    enableTelemetry: boolean
  }
}

const DEFAULT_SETTINGS: ExtensionSettings = {
  server: {
    url: 'http://localhost:8080',
    apiKey: '',
    timeout: 30
  },
  nativeHost: {
    path: '',
    autoConnect: true
  },
  recording: {
    defaultHARFilter: 'all',
    autoDownloadHAR: false,
    lsrProfilePath: ''
  },
  ui: {
    theme: 'dark',
    showNotifications: true,
    compactMode: false
  },
  advanced: {
    logLevel: 'info',
    enableTelemetry: false
  }
}

class ProxxyOptions {
  private settings: ExtensionSettings
  private elements: { [key: string]: HTMLElement } = {}
  private hasUnsavedChanges = false

  constructor() {
    this.settings = { ...DEFAULT_SETTINGS }
    this.init()
  }

  private async init() {
    this.cacheElements()
    this.setupEventListeners()
    this.loadSettings()
    this.updateExtensionInfo()
    this.detectNativeHostPath()
  }

  private cacheElements() {
    const elementIds = [
      'server-url', 'api-key', 'timeout', 'test-connection',
      'connection-status', 'connection-indicator', 'connection-message',
      'native-host-path', 'browse-path', 'auto-connect',
      'default-har-filter', 'auto-download-har', 'lsr-profile-path', 'browse-profile-path',
      'theme', 'show-notifications', 'compact-mode',
      'log-level', 'enable-telemetry',
      'reset-defaults', 'cancel', 'save',
      'save-status', 'documentation-link', 'support-link', 'github-link',
      'extension-version'
    ]

    elementIds.forEach(id => {
      const element = document.getElementById(id)
      if (element) {
        this.elements[id] = element
      }
    })
  }

  private setupEventListeners() {
    // Form inputs
    this.addEventListener('server-url', 'input', () => this.onInputChange())
    this.addEventListener('api-key', 'input', () => this.onInputChange())
    this.addEventListener('timeout', 'input', () => this.onInputChange())
    this.addEventListener('native-host-path', 'input', () => this.onInputChange())
    this.addEventListener('auto-connect', 'change', () => this.onInputChange())
    this.addEventListener('default-har-filter', 'change', () => this.onInputChange())
    this.addEventListener('auto-download-har', 'change', () => this.onInputChange())
    this.addEventListener('lsr-profile-path', 'input', () => this.onInputChange())
    this.addEventListener('theme', 'change', () => this.onInputChange())
    this.addEventListener('show-notifications', 'change', () => this.onInputChange())
    this.addEventListener('compact-mode', 'change', () => this.onInputChange())
    this.addEventListener('log-level', 'change', () => this.onInputChange())
    this.addEventListener('enable-telemetry', 'change', () => this.onInputChange())

    // Buttons
    this.addEventListener('test-connection', 'click', () => this.testConnection())
    this.addEventListener('browse-path', 'click', () => this.browseNativeHostPath())
    this.addEventListener('browse-profile-path', 'click', () => this.browseProfilePath())
    this.addEventListener('reset-defaults', 'click', () => this.resetToDefaults())
    this.addEventListener('save', 'click', () => this.saveSettings())
    this.addEventListener('cancel', 'click', () => this.cancelChanges())

    // External links
    this.addEventListener('documentation-link', 'click', () => this.openLink('https://docs.proxxy.dev'))
    this.addEventListener('support-link', 'click', () => this.openLink('https://github.com/anomalyco/proxxy/issues'))
    this.addEventListener('github-link', 'click', () => this.openLink('https://github.com/anomalyco/proxxy'))

    // Warn before leaving if unsaved
    window.addEventListener('beforeunload', (e) => {
      if (this.hasUnsavedChanges) {
        e.preventDefault()
        e.returnValue = 'You have unsaved changes. Are you sure you want to leave?'
      }
    })
  }

  private addEventListener(id: string, event: string, handler: () => void) {
    const element = this.elements[id]
    if (element) {
      element.addEventListener(event, handler)
    }
  }

  private async loadSettings() {
    try {
      const stored = await browser.storage.local.get(['settings'])
      if (stored.settings) {
        this.settings = this.mergeSettings(DEFAULT_SETTINGS, stored.settings)
      }
      this.populateForm()
    } catch (error) {
      console.error('Error loading settings:', error)
      this.showMessage('Error loading settings', 'error')
    }
  }

  private mergeSettings(defaults: ExtensionSettings, stored: Partial<ExtensionSettings>): ExtensionSettings {
    const merged = { ...defaults }
    
    // Deep merge each category separately
    Object.keys(stored).forEach(category => {
      const categoryKey = category as keyof ExtensionSettings
      if (merged[categoryKey] && stored[categoryKey]) {
        merged[categoryKey] = {
          ...merged[categoryKey],
          ...(stored[categoryKey] as any)
        }
      }
    })

    return merged
  }

  private populateForm() {
    // Server settings
    this.setInputValue('server-url', this.settings.server.url)
    this.setInputValue('api-key', this.settings.server.apiKey)
    this.setInputValue('timeout', this.settings.server.timeout.toString())

    // Native host settings
    this.setInputValue('native-host-path', this.settings.nativeHost.path || 'Auto-detected')
    this.setCheckboxValue('auto-connect', this.settings.nativeHost.autoConnect)

    // Recording settings
    this.setSelectValue('default-har-filter', this.settings.recording.defaultHARFilter)
    this.setCheckboxValue('auto-download-har', this.settings.recording.autoDownloadHAR)
    this.setInputValue('lsr-profile-path', this.settings.recording.lsrProfilePath || 'Default location')

    // UI settings
    this.setSelectValue('theme', this.settings.ui.theme)
    this.setCheckboxValue('show-notifications', this.settings.ui.showNotifications)
    this.setCheckboxValue('compact-mode', this.settings.ui.compactMode)

    // Advanced settings
    this.setSelectValue('log-level', this.settings.advanced.logLevel)
    this.setCheckboxValue('enable-telemetry', this.settings.advanced.enableTelemetry)

    this.hasUnsavedChanges = false
  }

  private setInputValue(id: string, value: string) {
    const element = this.elements[id] as HTMLInputElement
    if (element) {
      element.value = value
    }
  }

  private setSelectValue(id: string, value: string) {
    const element = this.elements[id] as HTMLSelectElement
    if (element) {
      element.value = value
    }
  }

  private setCheckboxValue(id: string, checked: boolean) {
    const element = this.elements[id] as HTMLInputElement
    if (element) {
      element.checked = checked
    }
  }

  private onInputChange() {
    this.hasUnsavedChanges = true
  }

  private async testConnection() {
    const serverUrl = (this.elements['server-url'] as HTMLInputElement).value
    const apiKey = (this.elements['api-key'] as HTMLInputElement).value
    const timeout = parseInt((this.elements['timeout'] as HTMLInputElement).value) * 1000

    if (!serverUrl) {
      this.showConnectionStatus('Please enter a server URL', 'error')
      return
    }

    const statusDiv = this.elements['connection-status'] as HTMLElement
    const indicator = this.elements['connection-indicator'] as HTMLElement
    const message = this.elements['connection-message'] as HTMLElement

    statusDiv.classList.remove('hidden')
    indicator.className = 'status-dot testing'
    message.textContent = 'Testing connection...'

    try {
      const response = await fetch(`${serverUrl}/api/health`, {
        method: 'GET',
        headers: apiKey ? { 'Authorization': `Bearer ${apiKey}` } : {},
        signal: AbortSignal.timeout(timeout)
      })

      if (response.ok) {
        const data = await response.json()
        this.showConnectionStatus(
          `Connected successfully! Server version: ${data.version || 'Unknown'}`,
          'success'
        )
      } else {
        this.showConnectionStatus(`Server responded with status ${response.status}`, 'error')
      }
    } catch (error) {
      if ((error as any).name === 'AbortError') {
        this.showConnectionStatus('Connection timed out', 'error')
      } else if ((error as any).name === 'TypeError') {
        this.showConnectionStatus('Invalid server URL or network error', 'error')
      } else {
        this.showConnectionStatus(`Connection failed: ${(error as Error).message}`, 'error')
      }
    }
  }

  private showConnectionStatus(message: string, status: 'success' | 'error' | 'testing') {
    const statusDiv = this.elements['connection-status'] as HTMLElement
    const indicator = this.elements['connection-indicator'] as HTMLElement
    const messageEl = this.elements['connection-message'] as HTMLElement

    statusDiv.classList.remove('hidden')
    indicator.className = `status-dot ${status}`
    messageEl.textContent = message
  }

  private async browseNativeHostPath() {
    // Fallback to simple input dialog since File System Access API is limited
    const currentPath = (this.elements['native-host-path'] as HTMLInputElement).value
    const newPath = prompt('Enter native host path:', currentPath || '')
    
    if (newPath !== null) {
      this.setInputValue('native-host-path', newPath)
      this.onInputChange()
    }
  }

  private async browseProfilePath() {
    // Fallback to simple input dialog
    const currentPath = (this.elements['lsr-profile-path'] as HTMLInputElement).value
    const newPath = prompt('Enter LSR profile path:', currentPath || '')
    
    if (newPath !== null) {
      this.setInputValue('lsr-profile-path', newPath)
      this.onInputChange()
    }
  }

  private async detectNativeHostPath() {
    // Try to auto-detect the native host path
    const platform = navigator.platform.toLowerCase()
    let defaultPath = ''

    if (platform.includes('win')) {
      defaultPath = 'C:\\Program Files\\Proxxy\\proxxy-native-host.exe'
    } else if (platform.includes('mac')) {
      defaultPath = '/usr/local/bin/proxxy-native-host'
    } else if (platform.includes('linux')) {
      defaultPath = '/usr/local/bin/proxxy-native-host'
    }

    if (defaultPath && !this.settings.nativeHost.path) {
      this.setInputValue('native-host-path', defaultPath)
    }
  }

  private async resetToDefaults() {
    if (!confirm('Are you sure you want to reset all settings to their default values? This action cannot be undone.')) {
      return
    }

    this.settings = { ...DEFAULT_SETTINGS }
    this.populateForm()
    this.showMessage('Settings reset to defaults', 'info')
  }

  private async saveSettings() {
    try {
      // Collect form values
      this.settings.server.url = (this.elements['server-url'] as HTMLInputElement).value
      this.settings.server.apiKey = (this.elements['api-key'] as HTMLInputElement).value
      this.settings.server.timeout = parseInt((this.elements['timeout'] as HTMLInputElement).value)

      this.settings.nativeHost.path = (this.elements['native-host-path'] as HTMLInputElement).value
      this.settings.nativeHost.autoConnect = (this.elements['auto-connect'] as HTMLInputElement).checked

      this.settings.recording.defaultHARFilter = (this.elements['default-har-filter'] as HTMLSelectElement).value
      this.settings.recording.autoDownloadHAR = (this.elements['auto-download-har'] as HTMLInputElement).checked
      this.settings.recording.lsrProfilePath = (this.elements['lsr-profile-path'] as HTMLInputElement).value

      this.settings.ui.theme = (this.elements['theme'] as HTMLSelectElement).value
      this.settings.ui.showNotifications = (this.elements['show-notifications'] as HTMLInputElement).checked
      this.settings.ui.compactMode = (this.elements['compact-mode'] as HTMLInputElement).checked

      this.settings.advanced.logLevel = (this.elements['log-level'] as HTMLSelectElement).value
      this.settings.advanced.enableTelemetry = (this.elements['enable-telemetry'] as HTMLInputElement).checked

      // Validate settings
      if (!this.validateSettings()) {
        return
      }

      await browser.storage.local.set({ settings: this.settings })
      this.hasUnsavedChanges = false
      this.showMessage('Settings saved successfully!', 'success')

      // Notify background script of changes
      browser.runtime.sendMessage({
        action: 'settings_updated',
        payload: this.settings
      })

    } catch (error) {
      console.error('Error saving settings:', error)
      this.showMessage('Error saving settings', 'error')
    }
  }

  private validateSettings(): boolean {
    const serverUrl = this.settings.server.url
    const timeout = this.settings.server.timeout

    if (!serverUrl) {
      this.showMessage('Server URL is required', 'error')
      return false
    }

    try {
      new URL(serverUrl)
    } catch {
      this.showMessage('Invalid server URL format', 'error')
      return false
    }

    if (timeout < 5 || timeout > 60) {
      this.showMessage('Timeout must be between 5 and 60 seconds', 'error')
      return false
    }

    return true
  }

  private cancelChanges() {
    if (this.hasUnsavedChanges) {
      if (!confirm('You have unsaved changes. Are you sure you want to cancel?')) {
        return
      }
    }
    
    this.loadSettings()
    window.close()
  }

  private showMessage(message: string, type: 'success' | 'error' | 'info') {
    const statusEl = this.elements['save-status'] as HTMLElement
    statusEl.textContent = message
    statusEl.className = `save-status ${type}`
    
    setTimeout(() => {
      statusEl.textContent = ''
      statusEl.className = 'save-status'
    }, 5000)
  }

  private openLink(url: string) {
    browser.tabs.create({ url })
  }

  private updateExtensionInfo() {
    const manifest = browser.runtime.getManifest()
    const versionEl = this.elements['extension-version'] as HTMLElement
    if (versionEl) {
      versionEl.textContent = `v${manifest.version}`
    }
  }
}

// Initialize options page when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  new ProxxyOptions()
})