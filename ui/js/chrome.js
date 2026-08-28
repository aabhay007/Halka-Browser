// Browser Chrome UI Controller - Phase 5 (AI Preview Mode V1)
document.addEventListener('DOMContentLoaded', () => {
  // Navigation & Chrome Elements
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

  // AI Sidebar Views & Controls
  const btnAiPreview = document.getElementById('btn-ai-preview');
  const aiSidebar = document.getElementById('ai-sidebar');
  const btnAiClose = document.getElementById('btn-ai-close');
  const btnAiSettings = document.getElementById('btn-ai-settings');
  const btnPickElement = document.getElementById('btn-pick-element');
  const pickerBtnText = document.getElementById('picker-btn-text');

  const aiViewMain = document.getElementById('ai-view-main');
  const aiViewSettings = document.getElementById('ai-view-settings');
  const aiViewExport = document.getElementById('ai-view-export');
  const btnBackFromSettings = document.getElementById('btn-back-from-settings');
  const btnBackFromExport = document.getElementById('btn-back-from-export');
  const btnToggleKeyVisibility = document.getElementById('btn-toggle-key-visibility');

  const selectedCard = document.getElementById('selected-card');
  const elTag = document.getElementById('el-tag');
  const elId = document.getElementById('el-id');
  const elSelector = document.getElementById('el-selector');
  const elText = document.getElementById('el-text');
  const elPills = document.getElementById('el-pills');

  const aiPromptInput = document.getElementById('ai-prompt-input');
  const promptChips = document.querySelectorAll('.chip');
  const btnApplyPreview = document.getElementById('btn-apply-preview');

  const previewStatusBanner = document.getElementById('preview-status-banner');
  const previewSummaryText = document.getElementById('preview-summary-text');
  const aiErrorBanner = document.getElementById('ai-error-banner');
  const aiErrorText = document.getElementById('ai-error-text');

  const btnUndoPreview = document.getElementById('btn-undo-preview');
  const btnResetPreview = document.getElementById('btn-reset-preview');
  const btnExportPrompt = document.getElementById('btn-export-prompt');

  // Export View Elements
  const exportPromptText = document.getElementById('export-prompt-text');
  const btnCopyPrompt = document.getElementById('btn-copy-prompt');
  const copyBtnText = document.getElementById('copy-btn-text');

  // Settings View Elements
  const settingProvider = document.getElementById('setting-provider');
  const settingModel = document.getElementById('setting-model');
  const settingApiKey = document.getElementById('setting-api-key');
  const btnSaveSettings = document.getElementById('btn-save-settings');
  const settingsSaveFeedback = document.getElementById('settings-save-feedback');

  // State
  let currentTabs = [];
  let isPickingElement = false;
  let selectedElementContext = null;
  let currentAppliedRules = [];
  let latestAppliedCss = '';

  // Helper to get Tauri invoke function
  function getInvoke() {
    if (window.__TAURI__?.core?.invoke) {
      return window.__TAURI__.core.invoke;
    }
    if (window.__TAURI__?.invoke) {
      return window.__TAURI__.invoke;
    }
    return null;
  }

  // Safely invoke Tauri IPC commands
  async function tauriInvoke(cmd, args = {}) {
    const invoke = getInvoke();
    if (invoke) {
      try {
        return await invoke(cmd, args);
      } catch (err) {
        console.error(`Tauri IPC Error [${cmd}]:`, err);
        throw err;
      }
    } else {
      console.warn(`[IPC Unavailable] ${cmd}`, args);
    }
  }

  // Show inline notification/error inside the AI sidebar
  function showAiError(msg) {
    if (aiErrorBanner && aiErrorText) {
      aiErrorText.textContent = msg;
      aiErrorBanner.style.display = 'block';
      aiErrorBanner.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      setTimeout(() => {
        aiErrorBanner.style.display = 'none';
      }, 8000);
    } else {
      console.error(msg);
    }
  }

  // Check and update bookmark star state
  async function updateBookmarkState(url) {
    if (!url) return;
    try {
      const bookmarked = await tauriInvoke('is_bookmarked', { url });
      if (bookmarked) {
        btnBookmark.classList.add('bookmarked');
        btnBookmark.title = 'Remove Bookmark (Ctrl+D)';
      } else {
        btnBookmark.classList.remove('bookmarked');
        btnBookmark.title = 'Add Bookmark (Ctrl+D)';
      }
    } catch (e) {
      // Ignored if IPC unavailable
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

  // ========================================================
  // AI PREVIEW MODE CONTROLLER
  // ========================================================

  function switchAiView(viewName) {
    if (aiViewMain) aiViewMain.style.display = viewName === 'main' ? 'flex' : 'none';
    if (aiViewSettings) aiViewSettings.style.display = viewName === 'settings' ? 'flex' : 'none';
    if (aiViewExport) aiViewExport.style.display = viewName === 'export' ? 'flex' : 'none';
  }

  function updateSidebarState(isOpen) {
    if (isOpen) {
      aiSidebar.classList.add('open');
      btnAiPreview.classList.add('active');
      switchAiView('main');
    } else {
      aiSidebar.classList.remove('open');
      btnAiPreview.classList.remove('active');
      if (isPickingElement) {
        cancelElementPicker();
      }
    }
  }

  async function toggleAiSidebar(open) {
    const isOpened = await tauriInvoke('toggle_ai_sidebar', { open });
    updateSidebarState(isOpened);
  }

  btnAiPreview.addEventListener('click', () => toggleAiSidebar());
  btnAiClose.addEventListener('click', () => toggleAiSidebar(false));

  document.querySelectorAll('.btn-close-any-view').forEach(btn => {
    btn.addEventListener('click', () => toggleAiSidebar(false));
  });

  // Settings View Handlers
  btnAiSettings.addEventListener('click', async () => {
    switchAiView('settings');
    if (settingsSaveFeedback) settingsSaveFeedback.style.display = 'none';
    try {
      const settings = await tauriInvoke('get_ai_settings');
      if (settings) {
        if (settingProvider) settingProvider.value = settings.provider || 'groq';
        if (settingModel) settingModel.value = settings.model || 'openai/gpt-oss-120b';
        if (settingApiKey) settingApiKey.value = settings.api_key || '';
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  });

  btnBackFromSettings.addEventListener('click', () => switchAiView('main'));
  btnBackFromExport.addEventListener('click', () => switchAiView('main'));

  if (btnToggleKeyVisibility) {
    btnToggleKeyVisibility.addEventListener('click', () => {
      if (settingApiKey) {
        settingApiKey.type = settingApiKey.type === 'password' ? 'text' : 'password';
      }
    });
  }

  btnSaveSettings.addEventListener('click', async () => {
    try {
      await tauriInvoke('save_ai_settings', {
        settings: {
          provider: settingProvider.value,
          model: settingModel.value,
          api_key: settingApiKey.value.trim()
        }
      });
      if (settingsSaveFeedback) settingsSaveFeedback.style.display = 'block';
      setTimeout(() => {
        if (settingsSaveFeedback) settingsSaveFeedback.style.display = 'none';
        switchAiView('main');
      }, 900);
    } catch (err) {
      showAiError(`Failed to save settings: ${err}`);
    }
  });

  // Element Picker Trigger
  async function startElementPicker() {
    isPickingElement = true;
    btnPickElement.classList.add('active');
    pickerBtnText.textContent = 'Click an element on page (Esc to cancel)...';
    try {
      await tauriInvoke('start_element_picker');
    } catch (e) {
      console.error('Failed to start element picker:', e);
      cancelElementPicker();
    }
  }

  async function cancelElementPicker() {
    isPickingElement = false;
    btnPickElement.classList.remove('active');
    pickerBtnText.textContent = 'Select Element on Page';
    try {
      await tauriInvoke('cancel_element_picker');
    } catch (e) {
      // Ignore
    }
  }

  btnPickElement.addEventListener('click', () => {
    if (isPickingElement) {
      cancelElementPicker();
    } else {
      startElementPicker();
    }
  });

  // Display Selected Element Context
  function displaySelectedElement(data) {
    selectedElementContext = data;
    isPickingElement = false;
    btnPickElement.classList.remove('active');
    pickerBtnText.textContent = 'Select Different Element';

    selectedCard.classList.remove('empty');
    selectedCard.querySelector('.empty-msg').style.display = 'none';
    selectedCard.querySelector('.card-content').style.display = 'block';

    elTag.textContent = (data.tag || 'ELEMENT').toUpperCase();
    elId.textContent = data.id ? `#${data.id}` : (data.classes?.length ? `.${data.classes[0]}` : '');
    elSelector.textContent = data.selector || `${data.tag}`;
    elText.textContent = data.text ? `"${data.text}"` : '<No text content>';

    // Render key computed styles as pills
    elPills.innerHTML = '';
    const cs = data.computed_styles || {};
    const highlightProps = ['border-radius', 'background-color', 'color', 'display', 'padding', 'margin', 'font-size'];
    
    highlightProps.forEach(prop => {
      if (cs[prop]) {
        const pill = document.createElement('span');
        pill.className = 'style-pill';
        pill.textContent = `${prop}: ${cs[prop]}`;
        elPills.appendChild(pill);
      }
    });

    // Enable prompt & apply buttons
    btnApplyPreview.disabled = false;
    aiPromptInput.focus();
  }

  // Suggestion Chips
  promptChips.forEach(chip => {
    chip.addEventListener('click', () => {
      aiPromptInput.value = chip.dataset.prompt;
      aiPromptInput.focus();
    });
  });

  // Apply AI Preview
  btnApplyPreview.addEventListener('click', async () => {
    if (!selectedElementContext) {
      showAiError('Please select an element on the webpage first.');
      return;
    }

    const instruction = aiPromptInput.value.trim();
    if (!instruction) {
      showAiError('Please enter a description of the changes you would like to preview.');
      aiPromptInput.focus();
      return;
    }

    btnApplyPreview.disabled = true;
    btnApplyPreview.innerHTML = `
      <svg class="spinner" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" style="animation: spin 1s linear infinite;">
        <circle cx="12" cy="12" r="10" stroke-opacity="0.25"></circle>
        <path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"></path>
      </svg>
      <span>Generating Preview...</span>
    `;

    try {
      const response = await tauriInvoke('ai_generate_preview', {
        req: {
          instruction: instruction,
          element_context: selectedElementContext,
          current_preview_css: latestAppliedCss || null
        }
      });

      if (response && response.css_rules) {
        currentAppliedRules = response.css_rules;
        latestAppliedCss = await tauriInvoke('apply_preview_css', {
          rules: response.css_rules
        });

        // Show status banner
        previewStatusBanner.style.display = 'block';
        previewSummaryText.textContent = response.summary || 'AI preview applied successfully.';
        if (aiErrorBanner) aiErrorBanner.style.display = 'none';

        btnUndoPreview.disabled = false;
        btnResetPreview.disabled = false;
        btnExportPrompt.disabled = false;
      }
    } catch (err) {
      showAiError(`AI Preview Error: ${err}`);
    } finally {
      btnApplyPreview.disabled = false;
      btnApplyPreview.innerHTML = `
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"/>
        </svg>
        <span>Apply Preview</span>
      `;
    }
  });

  // Undo Preview
  btnUndoPreview.addEventListener('click', async () => {
    try {
      const prevRules = await tauriInvoke('undo_preview_css');
      if (prevRules && prevRules.length > 0) {
        currentAppliedRules = prevRules;
        previewSummaryText.textContent = 'Reverted to previous preview state.';
      } else {
        currentAppliedRules = [];
        latestAppliedCss = '';
        previewStatusBanner.style.display = 'none';
        btnUndoPreview.disabled = true;
        btnExportPrompt.disabled = true;
      }
    } catch (err) {
      console.error('Failed to undo preview:', err);
    }
  });

  // Reset Preview
  btnResetPreview.addEventListener('click', async () => {
    try {
      await tauriInvoke('reset_preview_css');
      currentAppliedRules = [];
      latestAppliedCss = '';
      previewStatusBanner.style.display = 'none';
      btnUndoPreview.disabled = true;
      btnResetPreview.disabled = true;
      btnExportPrompt.disabled = true;
    } catch (err) {
      console.error('Failed to reset preview:', err);
    }
  });

  // Export Prompt
  btnExportPrompt.addEventListener('click', async () => {
    if (!selectedElementContext || currentAppliedRules.length === 0) return;

    btnExportPrompt.disabled = true;
    btnExportPrompt.textContent = 'Generating Prompt...';

    try {
      const prompt = await tauriInvoke('export_preview_prompt', {
        req: {
          instruction: aiPromptInput.value.trim(),
          element_context: selectedElementContext,
          applied_css_rules: currentAppliedRules
        }
      });

      exportPromptText.value = prompt;
      switchAiView('export');
    } catch (err) {
      showAiError(`Export Prompt Error: ${err}`);
    } finally {
      btnExportPrompt.disabled = false;
      btnExportPrompt.innerHTML = `
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
          <line x1="16" y1="17" x2="8" y2="17"></line>
          <polyline points="10 9 9 9 8 9"></polyline>
        </svg>
        <span>Export Prompt for Cursor / Claude</span>
      `;
    }
  });

  // Copy Prompt to Clipboard
  btnCopyPrompt.addEventListener('click', async () => {
    const text = exportPromptText.value;
    if (text) {
      await navigator.clipboard.writeText(text);
      btnCopyPrompt.classList.add('copied');
      copyBtnText.textContent = 'Copied to Clipboard!';
      setTimeout(() => {
        btnCopyPrompt.classList.remove('copied');
        copyBtnText.textContent = 'Copy Prompt';
      }, 2000);
    }
  });

  // Keyboard Shortcuts (Standard Browser Shortcuts)
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      if (aiViewSettings && aiViewSettings.style.display !== 'none') {
        switchAiView('main');
        return;
      }
      if (aiViewExport && aiViewExport.style.display !== 'none') {
        switchAiView('main');
        return;
      }
      if (isPickingElement) {
        cancelElementPicker();
        return;
      }
    }
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

  // Setup Tauri Listeners and initial state
  async function setupTauri() {
    const listen = window.__TAURI__?.event?.listen;
    if (listen) {
      await listen('tab_state_changed', (event) => {
        if (event.payload) {
          renderTabs(event.payload);
        }
      });

      await listen('url_changed', (event) => {
        if (event.payload && typeof event.payload === 'string') {
          addressInput.value = event.payload;
          updateBookmarkState(event.payload);
          // Clean preview on new navigation
          currentAppliedRules = [];
          latestAppliedCss = '';
          previewStatusBanner.style.display = 'none';
          btnUndoPreview.disabled = true;
          btnResetPreview.disabled = true;
          btnExportPrompt.disabled = true;
        }
      });

      await listen('ai_sidebar_toggled', (event) => {
        updateSidebarState(event.payload);
      });

      await listen('ai_element_selected', (event) => {
        if (event.payload) {
          try {
            const data = typeof event.payload === 'string' ? JSON.parse(event.payload) : event.payload;
            displaySelectedElement(data);
          } catch (e) {
            console.error('Failed to parse selected element data:', e);
          }
        }
      });
    }

    // Fetch initial tab state
    const tabs = await tauriInvoke('get_tabs');
    if (tabs && tabs.length > 0) {
      renderTabs(tabs);
    }
  }

  // Initialize
  if (window.__TAURI__) {
    setupTauri();
  } else {
    const checkInterval = setInterval(() => {
      if (window.__TAURI__) {
        clearInterval(checkInterval);
        setupTauri();
      }
    }, 50);
    setTimeout(() => clearInterval(checkInterval), 2000);
  }
});
