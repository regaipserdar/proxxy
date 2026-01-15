import browser from 'webextension-polyfill'

export interface NativeMessage {
  id: string
  module: 'har' | 'lsr'
  action: string
  payload?: any
  timestamp: number
}

export interface NativeResponse {
  id: string
  success: boolean
  data?: any
  error?: string
}

export interface StatusUpdate {
  module: 'har' | 'lsr'
  status: string
  data: any
}

export class NativeMessagingHost {
  private port: browser.Runtime.Port | null = null
  private messageQueue: Map<string, PendingMessage> = new Map()
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000
  private reconnectTimer: number | null = null
  private isDestroyed = false
  
  // Event listeners
  onConnect: Array<() => void> = []
  onDisconnect: Array<() => void> = []
  onStatusUpdate: Array<(update: StatusUpdate) => void> = []
  
  constructor(private hostName: string) {
    console.log(`[NativeHost] Initialized for: ${hostName}`)
  }
  
  async connect(): Promise<void> {
    if (this.isDestroyed) {
      console.log('[NativeHost] Cannot connect - host is destroyed')
      return
    }

    try {
      console.log(`[NativeHost] Attempting to connect to: ${this.hostName}`)
      this.port = browser.runtime.connectNative(this.hostName)
      this.port.onMessage.addListener(this.handleMessage.bind(this))
      this.port.onDisconnect.addListener(this.handleDisconnect.bind(this))
      
      this.reconnectAttempts = 0
      this.onConnect.forEach(listener => listener())
      
      console.log(`[NativeHost] Connected to native host: ${this.hostName}`)
    } catch (error) {
      console.error(`[NativeHost] Failed to connect to native host:`, error)
      this.scheduleReconnect()
      throw error
    }
  }
  
  async sendCommand(module: string, action: string, payload?: any): Promise<NativeResponse> {
    if (!this.port) {
      throw new Error('Not connected to native host')
    }
    
    const id = crypto.randomUUID()
    const message: NativeMessage = {
      id,
      module: module as 'har' | 'lsr',
      action,
      payload,
      timestamp: Date.now()
    }
    
    return new Promise((resolve, reject) => {
      // Set up timeout
      const timeout = setTimeout(() => {
        this.messageQueue.delete(id)
        reject(new Error('Command timeout'))
      }, 30000)
      
      // Store pending message
      this.messageQueue.set(id, {
        resolve: (response) => {
          clearTimeout(timeout)
          resolve(response)
        },
        reject: (error) => {
          clearTimeout(timeout)
          reject(error)
        }
      })
      
      // Send message
      try {
        this.port!.postMessage(message)
      } catch (error) {
        clearTimeout(timeout)
        this.messageQueue.delete(id)
        reject(error)
      }
    })
  }
  
  async testConnection(): Promise<boolean> {
    try {
      console.log('[NativeHost] Testing connection...')
      
      // First check if we're connected
      if (!this.isConnected()) {
        console.log('[NativeHost] Not connected, attempting to connect...')
        await this.connect()
      }
      
      // Send a simple ping command
      const response = await this.sendCommand('system', 'ping', { timestamp: Date.now() })
      
      if (response.success) {
        console.log('[NativeHost] Connection test successful')
        return true
      } else {
        console.warn('[NativeHost] Connection test failed:', response.error)
        return false
      }
    } catch (error) {
      console.error('[NativeHost] Connection test failed:', error)
      return false
    }
  }
  
  private handleMessage(message: NativeResponse | StatusUpdate) {
    if ('module' in message && 'status' in message) {
      // Status update
      this.onStatusUpdate.forEach(listener => listener(message as StatusUpdate))
    } else {
      // Command response
      const response = message as NativeResponse
      const pending = this.messageQueue.get(response.id)
      
      if (pending) {
        this.messageQueue.delete(response.id)
        
        if (response.success) {
          pending.resolve(response)
        } else {
          pending.reject(new Error(response.error || 'Command failed'))
        }
      }
    }
  }
  
  private handleDisconnect() {
    console.log('[NativeHost] Native host disconnected')
    
    // Clear any existing reconnect timer
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    
    // Reject all pending messages
    this.messageQueue.forEach(pending => {
      pending.reject(new Error('Native host disconnected'))
    })
    this.messageQueue.clear()
    
    this.port = null
    this.onDisconnect.forEach(listener => listener())
    
    // Attempt to reconnect if not destroyed
    if (!this.isDestroyed && this.reconnectAttempts < this.maxReconnectAttempts) {
      this.scheduleReconnect()
    } else if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('[NativeHost] Max reconnection attempts reached')
    }
  }
  
  private scheduleReconnect() {
    if (this.reconnectTimer || this.isDestroyed) {
      return
    }
    
    this.reconnectAttempts++
    const delay = Math.min(this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1), 30000) // Max 30 seconds
    
    console.log(`[NativeHost] Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempts})`)
    
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null
      this.connect().catch(error => {
        console.error('[NativeHost] Reconnection failed:', error)
      })
    }, delay)
  }
  
  disconnect() {
    console.log('[NativeHost] Disconnecting from native host')
    this.isDestroyed = true
    
    // Clear reconnect timer
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    
    // Reject all pending messages
    this.messageQueue.forEach(pending => {
      pending.reject(new Error('Native host disconnected'))
    })
    this.messageQueue.clear()
    
    if (this.port) {
      this.port.disconnect()
      this.port = null
    }
  }
  
  isConnected(): boolean {
    return this.port !== null && !this.port.error
  }
}

interface PendingMessage {
  resolve: (response: NativeResponse) => void
  reject: (error: Error) => void
}