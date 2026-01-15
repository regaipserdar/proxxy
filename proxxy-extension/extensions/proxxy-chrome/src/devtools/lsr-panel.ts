import browser from 'webextension-polyfill'

export interface LSRStep {
  id: string
  type: 'navigate' | 'click' | 'type' | 'wait'
  description: string
  selector?: string
  value?: string
  timestamp: number
}

export interface LSRProfile {
  id: string
  name: string
  url: string
  steps: LSRStep[]
  createdAt: number
  lastReplayed?: number
  success?: boolean
}

export interface LSRState {
  recording: boolean
  profileName: string
  currentSteps: LSRStep[]
  profiles: LSRProfile[]
  status: 'idle' | 'recording' | 'replaying'
  currentProfile?: string
}

export class LSRPanel {
  private state: LSRState = {
    recording: false,
    profileName: '',
    currentSteps: [],
    profiles: [],
    status: 'idle'
  }
  
  private elements: {
    recordBtn: HTMLButtonElement
    stopBtn: HTMLButtonElement
    replayBtn: HTMLButtonElement
    statusDot: HTMLElement
    statusText: HTMLElement
    profileName: HTMLInputElement
    stepsList: HTMLElement
    profilesList: HTMLElement
  }
  
  constructor() {
    this.elements = this.getElements()
  }
  
  private getElements() {
    return {
      recordBtn: document.getElementById('lsr-record') as HTMLButtonElement,
      stopBtn: document.getElementById('lsr-stop') as HTMLButtonElement,
      replayBtn: document.getElementById('lsr-replay') as HTMLButtonElement,
      statusDot: document.getElementById('lsr-status') as HTMLElement,
      statusText: document.getElementById('lsr-status-text') as HTMLElement,
      profileName: document.getElementById('profile-name') as HTMLInputElement,
      stepsList: document.getElementById('steps-list') as HTMLElement,
      profilesList: document.getElementById('profiles-list') as HTMLElement
    }
  }
  
  init() {
    this.setupEventListeners()
    this.loadProfiles()
    this.updateUI()
  }
  
  private setupEventListeners() {
    this.elements.recordBtn.addEventListener('click', () => this.startRecording())
    this.elements.stopBtn.addEventListener('click', () => this.stopRecording())
    this.elements.replayBtn.addEventListener('click', () => this.replayProfile())
    this.elements.profileName.addEventListener('input', () => this.updateProfileName())
  }
  
  async startRecording() {
    const profileName = this.elements.profileName.value.trim()
    if (!profileName) {
      alert('Please enter a profile name')
      return
    }
    
    try {
      const response = await browser.runtime.sendMessage({
        action: 'lsr_record_start',
        payload: {
          profile_name: profileName,
          start_url: await this.getCurrentTabUrl()
        }
      })
      
      if (response.success) {
        this.state.recording = true
        this.state.profileName = profileName
        this.state.currentSteps = []
        this.state.status = 'recording'
        this.updateUI()
      } else {
        console.error('Failed to start LSR recording:', response.error)
      }
    } catch (error) {
      console.error('Error starting LSR recording:', error)
    }
  }
  
  async stopRecording() {
    try {
      const response = await browser.runtime.sendMessage({
        action: 'lsr_record_stop'
      })
      
      if (response.success) {
        this.state.recording = false
        this.state.status = 'idle'
        this.state.currentSteps = response.data.steps || []
        this.updateUI()
        this.loadProfiles()
      } else {
        console.error('Failed to stop LSR recording:', response.error)
      }
    } catch (error) {
      console.error('Error stopping LSR recording:', error)
    }
  }
  
  async replayProfile() {
    const selectedProfile = this.getSelectedProfile()
    if (!selectedProfile) {
      alert('Please select a profile to replay')
      return
    }
    
    try {
      this.state.status = 'replaying'
      this.updateUI()
      
      const response = await browser.runtime.sendMessage({
        action: 'lsr_replay',
        payload: {
          profile_id: selectedProfile.id
        }
      })
      
      if (response.success) {
        // Update profile with replay results
        selectedProfile.lastReplayed = Date.now()
        selectedProfile.success = response.data.success
        this.loadProfiles()
      } else {
        console.error('Failed to replay LSR profile:', response.error)
      }
      
      this.state.status = 'idle'
      this.updateUI()
    } catch (error) {
      console.error('Error replaying LSR profile:', error)
      this.state.status = 'idle'
      this.updateUI()
    }
  }
  
  private async getCurrentTabUrl(): Promise<string> {
    const tabs = await browser.tabs.query({ active: true, currentWindow: true })
    return tabs[0]?.url || ''
  }
  
  private updateProfileName() {
    this.state.profileName = this.elements.profileName.value
  }
  
  private getSelectedProfile(): LSRProfile | null {
    const selectedRadio = document.querySelector('input[name="profile"]:checked') as HTMLInputElement
    if (!selectedRadio) return null
    
    return this.state.profiles.find(p => p.id === selectedRadio.value) || null
  }
  
  async loadProfiles() {
    try {
      const response = await browser.runtime.sendMessage({
        action: 'lsr_list_profiles'
      })
      
      if (response.success) {
        this.state.profiles = response.data.profiles || []
        this.renderProfiles()
      }
    } catch (error) {
      console.error('Error loading LSR profiles:', error)
    }
  }
  
  private renderProfiles() {
    const container = this.elements.profilesList
    
    if (this.state.profiles.length === 0) {
      container.innerHTML = '<div class="empty-state">No saved profiles</div>'
      return
    }
    
    container.innerHTML = this.state.profiles.map(profile => `
      <div class="profile-item">
        <label class="profile-label">
          <input type="radio" name="profile" value="${profile.id}" 
                 ${profile.id === this.state.currentProfile ? 'checked' : ''}>
          <div class="profile-info">
            <div class="profile-name">${profile.name}</div>
            <div class="profile-details">
              ${profile.steps.length} steps • ${new Date(profile.createdAt).toLocaleDateString()}
              ${profile.lastReplayed ? `• Last replayed: ${new Date(profile.lastReplayed).toLocaleDateString()}` : ''}
            </div>
          </div>
          <div class="profile-status">
            ${profile.success !== undefined ? 
              `<span class="status-indicator ${profile.success ? 'success' : 'error'}">
                ${profile.success ? '✓' : '✗'}
              </span>` : ''}
          </div>
        </label>
        <button class="btn btn-small btn-danger delete-profile" data-profile-id="${profile.id}">
          Delete
        </button>
      </div>
    `).join('')
    
    // Add delete button listeners
    container.querySelectorAll('.delete-profile').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const profileId = (e.target as HTMLElement).getAttribute('data-profile-id')
        if (profileId) this.deleteProfile(profileId)
      })
    })
  }
  
  async deleteProfile(profileId: string) {
    if (!confirm('Are you sure you want to delete this profile?')) return
    
    try {
      const response = await browser.runtime.sendMessage({
        action: 'lsr_delete_profile',
        payload: { profile_id: profileId }
      })
      
      if (response.success) {
        this.state.profiles = this.state.profiles.filter(p => p.id !== profileId)
        this.renderProfiles()
      } else {
        console.error('Failed to delete LSR profile:', response.error)
      }
    } catch (error) {
      console.error('Error deleting LSR profile:', error)
    }
  }
  
  private renderSteps() {
    const container = this.elements.stepsList
    
    if (this.state.currentSteps.length === 0) {
      container.innerHTML = '<div class="empty-state">No steps recorded yet</div>'
      return
    }
    
    container.innerHTML = this.state.currentSteps.map((step, index) => `
      <div class="step-item">
        <div class="step-number">${index + 1}</div>
        <div class="step-content">
          <div class="step-type">${step.type.toUpperCase()}</div>
          <div class="step-description">${step.description}</div>
          ${step.selector ? `<div class="step-selector">${step.selector}</div>` : ''}
          ${step.value ? `<div class="step-value">Value: ${step.value}</div>` : ''}
        </div>
      </div>
    `).join('')
  }
  
  updateUI() {
    // Update status indicator
    this.elements.statusDot.className = `status-dot ${this.state.status}`
    this.elements.statusText.textContent = this.state.status.charAt(0).toUpperCase() + this.state.status.slice(1)
    
    // Update button states
    this.elements.recordBtn.disabled = this.state.recording
    this.elements.stopBtn.disabled = !this.state.recording
    this.elements.replayBtn.disabled = this.state.recording || this.state.profiles.length === 0
    
    // Update profile name input
    this.elements.profileName.disabled = this.state.recording
    
    // Render steps
    this.renderSteps()
  }
  
  // Method to update state from background messages
  updateState(updates: Partial<LSRState>) {
    Object.assign(this.state, updates)
    this.updateUI()
  }
  
  // Method to add a step during recording
  addStep(step: LSRStep) {
    this.state.currentSteps.push(step)
    this.renderSteps()
  }
}