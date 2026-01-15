import browser from 'webextension-polyfill'

export interface HARState {
  recording: boolean
  requests: number
  totalSize: number
  duration: number
  status: 'idle' | 'recording' | 'stopped'
}

export class HARPanel {
  private state: HARState = {
    recording: false,
    requests: 0,
    totalSize: 0,
    duration: 0,
    status: 'idle'
  }
  
  private elements: {
    startBtn: HTMLButtonElement
    stopBtn: HTMLButtonElement
    clearBtn: HTMLButtonElement
    downloadBtn: HTMLButtonElement
    statusDot: HTMLElement
    statusText: HTMLElement
    requestCount: HTMLElement
    totalSize: HTMLElement
    duration: HTMLElement
    domainFilter: HTMLInputElement
    typeFilter: HTMLSelectElement
  }
  
  constructor() {
    this.elements = this.getElements()
  }
  
  private getElements() {
    return {
      startBtn: document.getElementById('har-start') as HTMLButtonElement,
      stopBtn: document.getElementById('har-stop') as HTMLButtonElement,
      clearBtn: document.getElementById('har-clear') as HTMLButtonElement,
      downloadBtn: document.getElementById('har-download') as HTMLButtonElement,
      statusDot: document.getElementById('har-status') as HTMLElement,
      statusText: document.getElementById('har-status-text') as HTMLElement,
      requestCount: document.getElementById('har-request-count') as HTMLElement,
      totalSize: document.getElementById('har-total-size') as HTMLElement,
      duration: document.getElementById('har-duration') as HTMLElement,
      domainFilter: document.getElementById('domain-filter') as HTMLInputElement,
      typeFilter: document.getElementById('type-filter') as HTMLSelectElement
    }
  }
  
  init() {
    this.setupEventListeners()
    this.updateUI()
  }
  
  private setupEventListeners() {
    this.elements.startBtn.addEventListener('click', () => this.startRecording())
    this.elements.stopBtn.addEventListener('click', () => this.stopRecording())
    this.elements.clearBtn.addEventListener('click', () => this.clearRecording())
    this.elements.downloadBtn.addEventListener('click', () => this.downloadHAR())
    this.elements.domainFilter.addEventListener('input', () => this.applyFilters())
    this.elements.typeFilter.addEventListener('change', () => this.applyFilters())
  }
  
  async startRecording() {
    try {
      const response = await browser.runtime.sendMessage({
        action: 'har_start',
        payload: {
          filter: this.getFilterConfig()
        }
      })
      
      if (response.success) {
        this.state.recording = true
        this.state.status = 'recording'
        this.state.duration = 0
        this.startDurationTimer()
        this.updateUI()
      } else {
        console.error('Failed to start HAR recording:', response.error)
      }
    } catch (error) {
      console.error('Error starting HAR recording:', error)
    }
  }
  
  async stopRecording() {
    try {
      const response = await browser.runtime.sendMessage({
        action: 'har_stop'
      })
      
      if (response.success) {
        this.state.recording = false
        this.state.status = 'stopped'
        this.stopDurationTimer()
        this.updateUI()
      } else {
        console.error('Failed to stop HAR recording:', response.error)
      }
    } catch (error) {
      console.error('Error stopping HAR recording:', error)
    }
  }
  
  async clearRecording() {
    try {
      const response = await browser.runtime.sendMessage({
        action: 'har_clear'
      })
      
      if (response.success) {
        this.state.requests = 0
        this.state.totalSize = 0
        this.state.duration = 0
        this.updateUI()
      } else {
        console.error('Failed to clear HAR recording:', response.error)
      }
    } catch (error) {
      console.error('Error clearing HAR recording:', error)
    }
  }
  
  async downloadHAR() {
    try {
      const response = await browser.runtime.sendMessage({
        action: 'har_export',
        payload: {
          filename: `capture_${Date.now()}.har`
        }
      })
      
      if (response.success) {
        // Create blob and trigger download
        const blob = new Blob([response.data.har], { type: 'application/json' })
        const url = URL.createObjectURL(blob)
        
        await browser.downloads.download({
          url: url,
          filename: response.data.filename,
          saveAs: true
        })
        
        URL.revokeObjectURL(url)
      } else {
        console.error('Failed to download HAR:', response.error)
      }
    } catch (error) {
      console.error('Error downloading HAR:', error)
    }
  }
  
  private getFilterConfig() {
    return {
      domain: this.elements.domainFilter.value,
      type: this.elements.typeFilter.value
    }
  }
  
  private applyFilters() {
    // Send filter updates to background
    browser.runtime.sendMessage({
      action: 'har_update_filter',
      payload: this.getFilterConfig()
    })
  }
  
  private durationTimer: number | null = null
  
  private startDurationTimer() {
    this.durationTimer = window.setInterval(() => {
      this.state.duration++
      this.updateDuration()
    }, 1000)
  }
  
  private stopDurationTimer() {
    if (this.durationTimer) {
      clearInterval(this.durationTimer)
      this.durationTimer = null
    }
  }
  
  private updateDuration() {
    const seconds = this.state.duration
    const minutes = Math.floor(seconds / 60)
    const remainingSeconds = seconds % 60
    
    this.elements.duration.textContent = minutes > 0 
      ? `${minutes}m ${remainingSeconds}s`
      : `${seconds}s`
  }
  
  updateUI() {
    // Update status indicator
    this.elements.statusDot.className = `status-dot ${this.state.status}`
    this.elements.statusText.textContent = this.state.status.charAt(0).toUpperCase() + this.state.status.slice(1)
    
    // Update button states
    this.elements.startBtn.disabled = this.state.recording
    this.elements.stopBtn.disabled = !this.state.recording
    this.elements.downloadBtn.disabled = this.state.status !== 'stopped'
    
    // Update stats
    this.elements.requestCount.textContent = this.state.requests.toString()
    this.elements.totalSize.textContent = this.formatBytes(this.state.totalSize)
    this.updateDuration()
  }
  
  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }
  
  // Method to update state from background messages
  updateState(updates: Partial<HARState>) {
    Object.assign(this.state, updates)
    this.updateUI()
  }
}