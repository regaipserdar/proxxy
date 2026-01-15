import { HARPanel } from './har-panel'
import { LSRPanel } from './lsr-panel'

export class ProxxyDevToolsPanel {
  private harPanel: HARPanel
  private lsrPanel: LSRPanel
  
  constructor() {
    this.harPanel = new HARPanel()
    this.lsrPanel = new LSRPanel()
    this.init()
  }
  
  private init() {
    this.setupTabSwitching()
    this.harPanel.init()
    this.lsrPanel.init()
  }
  
  private setupTabSwitching() {
    const tabButtons = document.querySelectorAll('.tab-button')
    const tabPanels = document.querySelectorAll('.tab-panel')
    
    tabButtons.forEach(button => {
      button.addEventListener('click', () => {
        const targetTab = button.getAttribute('data-tab') as 'har' | 'lsr'
        
        // Update button states
        tabButtons.forEach(btn => btn.classList.remove('active'))
        button.classList.add('active')
        
        // Update panel visibility
        tabPanels.forEach(panel => panel.classList.remove('active'))
        document.getElementById(`${targetTab}-tab`)?.classList.add('active')
      })
    })
  }
}

// Initialize the panel when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  new ProxxyDevToolsPanel()
})