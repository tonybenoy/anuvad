// Open Anuvad side panel when the extension icon is clicked
if (chrome.sidePanel) {
  chrome.sidePanel.setPanelBehavior({ openPanelOnActionClick: true });
}
