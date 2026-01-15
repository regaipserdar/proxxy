import browser from 'webextension-polyfill'

export interface ExtensionState {
  connected: boolean
  currentTab?: number
  har: {
    recording: boolean
    requests: number
    totalSize: number
    duration: number
    status: 'idle' | 'recording' | 'stopped'
    filter: {
      domain: string
      type: string
    }
  }
  lsr: {
    recording: boolean
    profileName: string
    currentSteps: number
    profiles: number
    status: 'idle' | 'recording' | 'replaying'
    currentProfile?: string
  }
}

export class StateManager {
  private state: ExtensionState = {
    connected: false,
    har: {
      recording: false,
      requests: 0,
      totalSize: 0,
      duration: 0,
      status: 'idle',
      filter: {
        domain: '',
        type: ''
      }
    },
    lsr: {
      recording: false,
      profileName: '',
      currentSteps: 0,
      profiles: 0,
      status: 'idle'
    }
  }
  
  constructor() {
    this.loadPersistedState()
  }
  
  private async loadPersistedState() {
    try {
      const stored = await browser.storage.local.get(['state'])
      if (stored.state) {
        // Only load certain persisted values, reset runtime state
        this.state.har.filter = stored.state.har?.filter || this.state.har.filter
        this.state.lsr.profiles = stored.state.lsr?.profiles || this.state.lsr.profiles
      }
    } catch (error) {
      console.error('Error loading persisted state:', error)
    }
  }
  
  private async persistState() {
    try {
      await browser.storage.local.set({
        state: {
          har: {
            filter: this.state.har.filter
          },
          lsr: {
            profiles: this.state.lsr.profiles
          }
        }
      })
    } catch (error) {
      console.error('Error persisting state:', error)
    }
  }
  
  getState(): ExtensionState {
    return { ...this.state }
  }
  
  setConnectionStatus(connected: boolean) {
    this.state.connected = connected
  }
  
  setCurrentTab(tabId: number) {
    this.state.currentTab = tabId
  }
  
  updateHARState(updates: Partial<ExtensionState['har']>) {
    Object.assign(this.state.har, updates)
    this.persistState()
  }
  
  updateLSRState(updates: Partial<ExtensionState['lsr']>) {
    Object.assign(this.state.lsr, updates)
    this.persistState()
  }
  
  // Specific update methods for common operations
  startHARRecording() {
    this.state.har.recording = true
    this.state.har.status = 'recording'
    this.state.har.requests = 0
    this.state.har.totalSize = 0
    this.state.har.duration = 0
  }
  
  stopHARRecording() {
    this.state.har.recording = false
    this.state.har.status = 'stopped'
  }
  
  clearHARData() {
    this.state.har.requests = 0
    this.state.har.totalSize = 0
    this.state.har.duration = 0
  }
  
  updateHARStats(requests: number, totalSize: number, duration: number) {
    this.state.har.requests = requests
    this.state.har.totalSize = totalSize
    this.state.har.duration = duration
  }
  
  startLSRRecording(profileName: string) {
    this.state.lsr.recording = true
    this.state.lsr.status = 'recording'
    this.state.lsr.profileName = profileName
    this.state.lsr.currentSteps = 0
  }
  
  stopLSRRecording() {
    this.state.lsr.recording = false
    this.state.lsr.status = 'idle'
    this.state.lsr.profileName = ''
    this.state.lsr.currentSteps = 0
  }
  
  updateLSRSteps(steps: number) {
    this.state.lsr.currentSteps = steps
  }
  
  updateLSRProfiles(profiles: number) {
    this.state.lsr.profiles = profiles
  }
  
  startLSRReplay(profileId: string) {
    this.state.lsr.status = 'replaying'
    this.state.lsr.currentProfile = profileId
  }
  
  stopLSRReplay() {
    this.state.lsr.status = 'idle'
    this.state.lsr.currentProfile = undefined
  }
}