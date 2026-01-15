import browser from 'webextension-polyfill'
import { NativeMessagingHost } from './native-host'
import { StateManager } from './state'

class ProxxyBackground {
  private nativeHost: NativeMessagingHost
  private stateManager: StateManager
  
  constructor() {
    this.nativeHost = new NativeMessagingHost('com.proxxy.native')
    this.stateManager = new StateManager()
    this.init()
  }
  
  private init() {
    this.setupEventListeners()
    this.connectToNativeHost()
  }
  
  private setupEventListeners() {
    // Extension installation/update
    browser.runtime.onInstalled.addListener((details) => {
      console.log('Proxxy Extension installed/updated:', details.reason)
      if (details.reason === 'install') {
        this.handleFirstInstall()
      }
    })
    
    // Message handling from popup, devtools, content scripts
    browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
      this.handleMessage(message, sender).then(sendResponse)
      return true // Keep message channel open for async response
    })
    
    // Native host connection events
    this.nativeHost.onConnect.push(() => {
      console.log('Connected to Proxxy native host')
      this.stateManager.setConnectionStatus(true)
      this.broadcastStateUpdate()
    })
    
    this.nativeHost.onDisconnect.push(() => {
      console.log('Disconnected from Proxxy native host')
      this.stateManager.setConnectionStatus(false)
      this.broadcastStateUpdate()
    })
    
    this.nativeHost.onStatusUpdate.push((update) => {
      this.handleStatusUpdate(update)
    })
    
    // Tab events for context
    browser.tabs.onActivated.addListener((activeInfo) => {
      this.stateManager.setCurrentTab(activeInfo.tabId)
    })
    
    browser.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
      if (changeInfo.status === 'complete' && tab.active) {
        this.stateManager.setCurrentTab(tabId)
      }
    })
  }
  
  private async connectToNativeHost() {
    try {
      console.log('[Background] Attempting to connect to native host...')
      await this.nativeHost.connect()
      this.stateManager.setConnectionStatus(true)
    } catch (error) {
      console.error('[Background] Failed to connect to native host:', error)
      this.stateManager.setConnectionStatus(false)
      
      // Don't retry automatically - let user manually trigger
      console.log('[Background] Native host not available. User will need to install or start Proxxy.')
    }
  }
  
  private async handleMessage(message: any, _sender: browser.Runtime.MessageSender) {
    try {
      switch (message.action) {
        // Connection and state
        case 'get_state':
          return { success: true, data: this.stateManager.getState() }
          
        case 'check_connection':
          console.log('[Background] Testing native host connection...')
          try {
            const connected = await this.nativeHost.testConnection()
            this.stateManager.setConnectionStatus(connected)
            return { success: connected, data: { connected } }
          } catch (error) {
            console.error('[Background] Connection test failed:', error)
            return { success: false, error: (error as Error).message }
          }
          
        // HAR commands
        case 'har_start':
          return await this.nativeHost.sendCommand('har', 'start', message.payload)
          
        case 'har_stop':
          return await this.nativeHost.sendCommand('har', 'stop', message.payload)
          
        case 'har_clear':
          return await this.nativeHost.sendCommand('har', 'clear', message.payload)
          
        case 'har_export':
          return await this.nativeHost.sendCommand('har', 'export', message.payload)
          
        case 'har_update_filter':
          return await this.nativeHost.sendCommand('har', 'update_filter', message.payload)
          
        // LSR commands
        case 'lsr_record_start':
          return await this.nativeHost.sendCommand('lsr', 'record_start', message.payload)
          
        case 'lsr_record_stop':
          return await this.nativeHost.sendCommand('lsr', 'record_stop', message.payload)
          
        case 'lsr_replay':
          return await this.nativeHost.sendCommand('lsr', 'replay', message.payload)
          
        case 'lsr_list_profiles':
          return await this.nativeHost.sendCommand('lsr', 'list_profiles', message.payload)
          
        case 'lsr_delete_profile':
          return await this.nativeHost.sendCommand('lsr', 'delete_profile', message.payload)
          
        // DevTools specific
        case 'focus_proxxy_panel':
          // This is a best-effort attempt to focus the Proxxy panel
          return { success: true }
          
        case 'settings_updated':
          this.handleSettingsUpdate(message.payload)
          return { success: true }
          
        default:
          return { success: false, error: `Unknown action: ${message.action}` }
      }
    } catch (error) {
      console.error(`Error handling message ${message.action}:`, error)
      return { success: false, error: (error as Error).message }
    }
  }
  
  private handleStatusUpdate(update: any) {
    console.log('Received status update:', update)
    
    // Update state based on the module
    switch (update.module) {
      case 'har':
        this.stateManager.updateHARState(update.data)
        break
      case 'lsr':
        this.stateManager.updateLSRState(update.data)
        break
    }
    
    // Broadcast update to all open panels
    this.broadcastStateUpdate()
  }
  
  private async broadcastStateUpdate() {
    try {
      const state = this.stateManager.getState()
      
      // Send to all DevTools panels
      const devTabs = await browser.tabs.query({})
      for (const tab of devTabs) {
        if (tab.id) {
          try {
            await browser.tabs.sendMessage(tab.id, {
              action: 'state_update',
              data: state
            })
          } catch (error) {
            // Tab might not have a content script, ignore
          }
        }
      }
      
      // Popup will get state when it opens via get_state
    } catch (error) {
      console.error('Error broadcasting state update:', error)
    }
  }
  
  private async handleFirstInstall() {
    try {
      // Setup hot reload for development
      if (process.env.NODE_ENV === 'development') {
        console.log('[Background] Development mode detected, setting up hot reload')
        this.setupHotReload()
      }

      // Check if settings already exist
      const existing = await browser.storage.local.get(['settings'])
      
      if (!existing.settings) {
        // Set default settings for first-time users
        const defaultSettings = {
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
        
        await browser.storage.local.set({ settings: defaultSettings })
      }
      
      // Could open a welcome page or setup guide
      console.log('Proxxy Extension installed successfully')
    } catch (error) {
      console.error('Error handling first install:', error)
    }
  }
  
  private handleSettingsUpdate(settings: any) {
    console.log('Settings updated:', settings)
    
    // Apply theme changes immediately
    if (settings.ui?.theme) {
      // Theme changes would be applied by individual components
      this.broadcastStateUpdate()
    }
    
    // Update native host connection settings
    if (settings.nativeHost?.autoConnect !== undefined) {
      // Reconnect if auto-connect was enabled
      if (settings.nativeHost.autoConnect && !this.stateManager.getState().connected) {
        this.connectToNativeHost()
      }
    }
  }
  
  private setupHotReload() {
    // Watch for changes in development
    console.log('[Background] Hot reload enabled')
  }
}

// Initialize the background service
new ProxxyBackground()