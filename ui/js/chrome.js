// Browser Chrome UI Controller - Phase 4 (Persistence Support)
document.addEventListener('DOMContentLoaded', () => {
  const tabList = document.getElementById('tab-list');
  const btnNewTab = document.getElementById('btn-new-tab');
  const addressForm = document.getElementById('address-bar-form');
  const addressInput = document.getElementById('address-input');
  const btnBookmark = document.getElementById('btn-bookmark');
  const btnBack = document.getElementById('btn-back');
  const btnForward = document.getElementById('btn-forward');
  const btnReload = document.getElementById('btn-reload');
  const btnHome = document.getElementById('btn-home');
  const btnDevtools = document.getElementById('btn-devtools');

  let currentTabs = [];

  // Safely invoke Tauri IPC commands
  async function tauriInvoke(cmd, args = {}) {
    if (window.__TAURI__ && window.__TAURI__.core) {
      try {
        return await window.__TAURI__.core.invoke(cmd, args);
      } catch (err) {
        console.error(`Tauri IPC Error [${cmd}]:`, err);
      }
    } else {
      console.log(`[IPC Mock] ${cmd}`, args);
    }
  }

  // Check and update bookmark star state
  async function updateBookmarkState(url) {
    if (!url) return;
    const bookmarked = await tauriInvoke('is_bookmarked', { url });
    if (bookmarked) {
      btnBookmark.classList.add('bookmarked');
      btnBookmark.title = 'Remove Bookmark (Ctrl+D)';
    } else {
      btnBookmark.classList.remove('bookmarked');
      btnBookmark.title = 'Add Bookmark (Ctrl+D)';
    }
  }

  // Toggle Bookmark
  async function toggleBookmark() {
    const activeTab = currentTabs.find(t => t.active);
    if (!activeTab || !activeTab.url) return;

    const isBookmarked = btnBookmark.classList.contains('bookmarked');
    if (isBookmarked) {
      await tauriInvoke('remove_bookmark', { url: activeTab.url });
    } else {
      await tauriInvoke('add_bookmark', { url: activeTab.url, title: activeTab.title || activeTab.url });
    }
    updateBookmarkState(activeTab.url);
  }

  // Render Tabs DOM
  function renderTabs(tabs) {
    currentTabs = tabs || [];
    tabList.innerHTML = '';

    currentTabs.forEach((tab) => {
      const tabEl = document.createElement('div');
      tabEl.className = `tab-item ${tab.active ? 'active' : ''}`;
      tabEl.dataset.tabId = tab.id;

      const titleEl = document.createElement('span');
      titleEl.className = 'tab-title';
      titleEl.textContent = tab.title || tab.url || 'New Tab';
      titleEl.title = tab.url || 'New Tab';

      const closeBtn = document.createElement('button');
      closeBtn.className = 'tab-close-btn';
      closeBtn.innerHTML = '&times;';
      closeBtn.title = 'Close tab (Ctrl+W)';

      closeBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        tauriInvoke('close_tab', { tabId: tab.id });
      });

      tabEl.appendChild(titleEl);
      tabEl.appendChild(closeBtn);

      tabEl.addEventListener('click', () => {
        if (!tab.active) {
          tauriInvoke('switch_tab', { tabId: tab.id });
        }
      });

      tabList.appendChild(tabEl);

      // Update address input & bookmark star if active tab
      if (tab.active) {
        addressInput.value = tab.url;
        updateBookmarkState(tab.url);
      }
    });
  }

  // Address Bar Submission
  addressForm.addEventListener('submit', (e) => {
    e.preventDefault();
    const inputVal = addressInput.value.trim();
    if (inputVal) {
      tauriInvoke('navigate', { input: inputVal });
    }
  });

  addressInput.addEventListener('focus', () => {
    addressInput.select();
  });

  // Action Handlers
  btnNewTab.addEventListener('click', () => tauriInvoke('create_tab', { url: 'https://www.google.com' }));
  btnBookmark.addEventListener('click', () => toggleBookmark());
  btnBack.addEventListener('click', () => tauriInvoke('go_back'));
  btnForward.addEventListener('click', () => tauriInvoke('go_forward'));
  btnReload.addEventListener('click', () => tauriInvoke('reload'));
  btnHome.addEventListener('click', () => tauriInvoke('navigate', { input: 'https://www.google.com' }));
  btnDevtools.addEventListener('click', () => tauriInvoke('open_devtools'));

  // Keyboard Shortcuts
  window.addEventListener('keydown', (e) => {
    // Ctrl+D Bookmark Toggle
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'd') {
      e.preventDefault();
      toggleBookmark();
    }
    // Ctrl+T New Tab
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key.toLowerCase() === 't') {
      e.preventDefault();
      tauriInvoke('create_tab', { url: 'https://www.google.com' });
    }
    // Ctrl+Shift+T Reopen Tab
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 't') {
      e.preventDefault();
      tauriInvoke('reopen_tab');
    }
    // Ctrl+W Close Active Tab
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'w') {
      e.preventDefault();
      const activeTab = currentTabs.find(t => t.active);
      if (activeTab) {
        tauriInvoke('close_tab', { tabId: activeTab.id });
      }
    }
    // Ctrl+Tab Next Tab
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'Tab') {
      e.preventDefault();
      const activeIdx = currentTabs.findIndex(t => t.active);
      if (activeIdx !== -1 && currentTabs.length > 1) {
        const nextIdx = (activeIdx + 1) % currentTabs.length;
        tauriInvoke('switch_tab', { tabId: currentTabs[nextIdx].id });
      }
    }
    // Ctrl+Shift+Tab Previous Tab
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'Tab') {
      e.preventDefault();
      const activeIdx = currentTabs.findIndex(t => t.active);
      if (activeIdx !== -1 && currentTabs.length > 1) {
        const prevIdx = (activeIdx - 1 + currentTabs.length) % currentTabs.length;
        tauriInvoke('switch_tab', { tabId: currentTabs[prevIdx].id });
      }
    }
    // Ctrl+L focus address bar
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l') {
      e.preventDefault();
      addressInput.focus();
      addressInput.select();
    }
    // Ctrl+R reload
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'r') {
      e.preventDefault();
      tauriInvoke('reload');
    }
  });

  // Listen for Rust events
  if (window.__TAURI__ && window.__TAURI__.event) {
    window.__TAURI__.event.listen('tab_state_changed', (event) => {
      if (event.payload) {
        renderTabs(event.payload);
      }
    });

    window.__TAURI__.event.listen('url_changed', (event) => {
      if (event.payload && typeof event.payload === 'string') {
        addressInput.value = event.payload;
        updateBookmarkState(event.payload);
      }
    });
  }
});
