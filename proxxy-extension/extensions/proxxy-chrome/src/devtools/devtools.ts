import browser from 'webextension-polyfill'

browser.devtools.panels.create(
  'Proxxy',
  'assets/icon-48.png',
  'panel.html'
)