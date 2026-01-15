import browser from 'webextension-polyfill'

interface PopupState {
  connected: boolean
  har: {
    recording: boolean
    requests: number
    status: string
  }
  lsr: {
    recording: boolean
    profiles: number
    status: string
  }
}

class ProxxyPopup {
  private state: PopupState = {
    connected: false,
    har: {
      recording: false,
      requests: 0,
      status: 'idle'
    },
    lsr: {
      recording: false,
      profiles: 0,
      status: 'idle'
    }
  }
  
  private elements: {
    statusDot: HTMLElement
    statusText: HTMLElement
    harStatus: HTMLElement
    harRequests: HTMLElement
    harQuickStart: HTMLButtonElement
    harQuickStop: HTMLButtonElement
    lsrStatus: HTMLElement
    lsrProfiles: HTMLElement
    lsrQuickRecord: HTMLButtonElement
    lsrQuickStop: HTMLButtonElement
    openDevTools: HTMLButtonElement
    settings: HTMLButtonElement
    troubleshooting: HTMLElement
    checkConnection: HTMLButtonElement
    installGuide: HTMLButtonElement
  }
  
  constructor() {
    this.elements = this.getElements()
    this.init()
  }
  
  private getElements() {
    return {
      statusDot: document.getElementById('status-dot') as HTMLElement,
      statusText: document.getElementById('status-text') as HTMLElement,
      harStatus: document.getElementById('har-status') as HTMLElement,
      harRequests: document.getElementById('har-requests') as HTMLElement,
      harQuickStart: document.getElementById('har-quick-start') as HTMLButtonElement,
      harQuickStop: document.getElementById('har-quick-stop') as HTMLButtonElement,
      lsrStatus: document.getElementById('lsr-status') as HTMLElement,
      lsrProfiles: document.getElementById('lsr-profiles') as HTMLElement,
      lsrQuickRecord: document.getElementById('lsr-quick-record') as HTMLButtonElement,
      lsrQuickStop: document.getElementById('lsr-quick-stop') as HTMLButtonElement,
      openDevTools: document.getElementById('open-devtools') as HTMLButtonElement,
      settings: document.getElementById('settings') as HTMLButtonElement,
      troubleshooting: document.getElementById('troubleshooting') as HTMLElement,
      checkConnection: document.getElementById('check-connection') as HTMLButtonElement,
      installGuide: document.getElementById('install-guide') as HTMLButtonElement,
      // Local config elements
      localServerUrl: document.getElementById('local-server-url') as HTMLInputElement,
      localNativePath: document.getElementById('local-native-path') as HTMLInputElement,
      localAutoConnect: document.getElementById('local-auto-connect') as HTMLInputElement,
      saveLocalConfig: document.getElementById('save-local-config') as HTMLButtonElement,
      testLocalConfig: document.getElementById('test-local-config') as HTMLButtonElement,
      localConfigStatus: document.getElementById('local-config-status') as HTMLElement,
      localConfigMessage: document.getElementById('local-config-message') as HTMLElement
    }
  }
  
  private async init() {
    this.setupEventListeners()
    await this.loadState()
    this.updateUI()
  }
  
  private setupEventListeners() {
    // Add null checks for all elements
    if (this.elements.harQuickStart) {
      this.elements.harQuickStart.addEventListener('click', () => this.quickAction('har_start'))
    }
    if (this.elements.harQuickStop) {
      this.elements.harQuickStop.addEventListener('click', () => this.quickAction('har_stop'))
    }
    if (this.elements.lsrQuickRecord) {
      this.elements.lsrQuickRecord.addEventListener('click', () => this.quickAction('lsr_record_start'))
    }
    if (this.elements.lsrQuickStop) {
      this.elements.lsrQuickStop.addEventListener('click', () => this.quickAction('lsr_record_stop'))
    }
    
    if (this.elements.openDevTools) {
      this.elements.openDevTools.addEventListener('click', () => this.openDevTools())
    }
    if (this.elements.settings) {
      this.elements.settings.addEventListener('click', () => this.openSettings())
    }
    
    if (this.elements.checkConnection) {
      this.elements.checkConnection.addEventListener('click', () => this.checkConnection())
    }
    if (this.elements.installGuide) {
      this.elements.installGuide.addEventListener('click', () => this.openInstallationGuide())
    }
    
    // Local config event listeners
    if (this.elements.saveLocalConfig) {
      this.elements.saveLocalConfig.addEventListener('click', () => this.saveLocalConfig())
    }
    if (this.elements.testLocalConfig) {
      this.elements.testLocalConfig.addEventListener('click', () => this.testLocalConfig())
    }
    
    // Auto-save local config on change
    if (this.elements.localServerUrl) {
      this.elements.localServerUrl.addEventListener('input', () => this.onLocalConfigChange())
    }
    if (this.elements.localNativePath) {
      this.elements.localNativePath.addEventListener('input', () => this.onLocalConfigChange())
    }
    if (this.elements.localAutoConnect) {
      this.elements.localAutoConnect.addEventListener('change', () => this.onLocalConfigChange())
    }
  }
  
  private async loadState() {
    try {
      const [stateResponse, settingsResponse] = await Promise.all([
        browser.runtime.sendMessage({ action: 'get_state' }),
        browser.storage.local.get(['settings']),
        browser.storage.local.get(['localConfig'])
      ])
      
      if (stateResponse.success) {
        this.state = { ...this.state, ...stateResponse.data }
      }
      
      // Load local config if exists
      if (settingsResponse.settings) {
        console.log('Settings loaded:', settingsResponse.settings)
      }
      
      if (settingsResponse.localConfig) {
        this.loadLocalConfig(settingsResponse.localConfig)
      }
      
    } catch (error) {
      console.error('Error loading popup state:', error)
      this.state.connected = false
    }
  }
  
  private loadLocalConfig(localConfig: any) {
    if (localConfig) {
      if (this.elements.localServerUrl) {
        this.elements.localServerUrl.value = localConfig.serverUrl || ''
      }
      if (this.elements.localNativePath) {
        this.elements.localNativePath.value = localConfig.nativePath || ''
      }
      if (this.elements.localAutoConnect) {
        this.elements.localAutoConnect.checked = localConfig.autoConnect || false
      }
    }
  }
  
  private saveLocalConfig() {
    const localConfig = {
      serverUrl: this.elements.localServerUrl?.value || '',
      nativePath: this.elements.localNativePath?.value || '',
      autoConnect: this.elements.localAutoConnect?.checked || false
    }
    
    browser.storage.local.set({ localConfig }).then(() => {
      this.showLocalConfigStatus('Local config saved successfully!', 'success')
    }).catch((error) => {
      this.showLocalConfigStatus('Failed to save local config', 'error')
    })
  }
  
  private async testLocalConfig() {
    const serverUrl = this.elements.localServerUrl?.value
    if (!serverUrl) {
      this.showLocalConfigStatus('Please enter a server URL', 'error')
      return
    }
    
    this.showLocalConfigStatus('Testing connection...', 'testing')
    
    try {
      const response = await fetch(`${serverUrl}/api/health`, {
        method: 'GET',
        signal: AbortSignal.timeout(10000)
      })
      
      if (response.ok) {
        this.showLocalConfigStatus('Connection successful!', 'success')
      } else {
        this.showLocalConfigStatus(`Server responded with status ${response.status}`, 'error')
      }
    } catch (error) {
      this.showLocalConfigStatus(`Connection failed: ${(error as Error).message}`, 'error')
    }
  }
  
  private onLocalConfigChange() {
    this.showLocalConfigStatus('Unsaved changes', 'info')
  }
  
  private showLocalConfigStatus(message: string, status: 'success' | 'error' | 'testing' | 'info') {
    if (!this.elements.localConfigStatus || !this.elements.localConfigMessage) return
    
    this.elements.localConfigStatus.classList.remove('hidden')
    this.elements.localConfigStatus.className = `config-status ${status}`
    this.elements.localConfigMessage.textContent = message
    
    // Auto-hide success messages
    if (status === 'success') {
      setTimeout(() => {
        this.elements.localConfigStatus.classList.add('hidden')
      }, 3000)
    }
  }
  
  private async quickAction(action: string) {
    try {
      const response = await browser.runtime.sendMessage({
        action: action
      })
      
      if (response.success) {
        await this.loadState()
        this.updateUI()
      } else {
        console.error(`Quick action ${action} failed:`, response.error)
      }
    } catch (error) {
      console.error(`Error executing quick action ${action}:`, error)
    }
  }
  
  private async openDevTools() {
    try {
      // Get current tab
      const tabs = await browser.tabs.query({ active: true, currentWindow: true })
      if (tabs[0]?.id) {
        // Focus on Proxxy panel (this might not work reliably, but it's worth trying)
        await browser.tabs.sendMessage(tabs[0].id, {
          action: 'focus_proxxy_panel'
        })
      }
    } catch (error) {
      console.error('Error opening DevTools:', error)
    }
  }
  
  private openSettings() {
    browser.runtime.openOptionsPage()
  }
  
  private async checkConnection() {
    try {
      const response = await browser.runtime.sendMessage({
        action: 'check_connection'
      })
      
      if (response.success) {
        this.state.connected = true
        this.updateUI()
      } else {
        alert('Connection test failed. Please ensure Proxxy is running and the native host is properly installed.')
      }
    } catch (error) {
      console.error('Connection check failed:', error)
      alert('Connection test failed. Please check the console for details.')
    }
  }
  
  private openInstallationGuide() {
    browser.tabs.create({
      url: 'https://github.com/anomalyco/proxxy#installation'
    })
  }
  
  private updateUI() {
    // Update connection status
    if (this.elements.statusDot) {
      this.elements.statusDot.className = `status-dot ${this.state.connected ? 'connected' : 'disconnected'}`
    }
    if (this.elements.statusText) {
      this.elements.statusText.textContent = this.state.connected ? 'Connected' : 'Disconnected'
    }
    
    // Update HAR status
    if (this.elements.harStatus) {
      this.elements.harStatus.textContent = this.state.har.status.charAt(0).toUpperCase() + this.state.har.status.slice(1)
    }
    if (this.elements.harRequests) {
      this.elements.harRequests.textContent = `${this.state.har.requests} requests`
    }
    if (this.elements.harQuickStart) {
      this.elements.harQuickStart.disabled = this.state.har.recording || !this.state.connected
    }
    if (this.elements.harQuickStop) {
      this.elements.harQuickStop.disabled = !this.state.har.recording || !this.state.connected
    }
    
    // Update LSR status
    if (this.elements.lsrStatus) {
      this.elements.lsrStatus.textContent = this.state.lsr.status.charAt(0).toUpperCase() + this.state.lsr.status.slice(1)
    }
    if (this.elements.lsrProfiles) {
      this.elements.lsrProfiles.textContent = `${this.state.lsr.profiles} profiles`
    }
    if (this.elements.lsrQuickRecord) {
      this.elements.lsrQuickRecord.disabled = this.state.lsr.recording || !this.state.connected
    }
    if (this.elements.lsrQuickStop) {
      this.elements.lsrQuickStop.disabled = !this.state.lsr.recording || !this.state.connected
    }
    
    // Show/hide troubleshooting
    if (this.elements.troubleshooting) {
      this.elements.troubleshooting.style.display = this.state.connected ? 'none' : 'block'
    }
    
    // Disable actions if not connected
    if (this.elements.openDevTools) {
      this.elements.openDevTools.disabled = !this.state.connected
    }
  }
}

// Initialize popup when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  new ProxxyPopup()
})