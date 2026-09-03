// charis Desktop Application Controller
(function () {
  // 安全な invoke 取得ラッパー（Tauri の初期化待ちリトライ付き）
  async function invokeTauri(cmd, args = {}) {
    let retries = 10;
    while (retries > 0) {
      const inv = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke || window.__TAURI__?.tauri?.invoke;
      if (typeof inv === 'function') {
        return await inv(cmd, args);
      }
      await new Promise(r => setTimeout(r, 100));
      retries--;
    }
    const err = "Tauri API (window.__TAURI__.core.invoke) が検出できませんでした。";
    console.error(err, window.__TAURI__);
    throw new Error(err);
  }

  // App State
  const DEFAULT_APP_SETTINGS = {
    uiFontFamily: "'M PLUS 1', 'M PLUS 1p', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Noto Color Emoji', 'Apple Color Emoji', 'Segoe UI Emoji', sans-serif",
    uiFontSize: 13,
    postFontFamily: "",
    postFontSize: 14,
    postLineHeight: 1.65,
    defaultBlurImages: true,
    scrollAmount: 120,
    theme: "dark",
    initialScrollPosition: "top",
    defaultName: "",
    defaultMail: "sage"
  };
  let appSettings = Object.assign({}, DEFAULT_APP_SETTINGS);
  let saveSettingsTimeout = null;
  let readPositions = {};
  let saveReadPosTimeout = null;
  let pendingConfirmParams = null;
  let isPosting = false;

  let categories = [];
  let favorites = [];
  let bookmarks = [];
  let history = [];
  let ngSettings = { ngWords: [], ngIds: [], ngMode: 'abone', chainAbone: true };

  let currentBoard = null;
  let currentThread = null;
  let rawThreads = [];
  let rawPosts = [];
  let currentSort = 'ikioi';
  let filterText = '';
  let autoReloadTimer = null;
  let blurImages = true;

  // Post Maps
  let postsMap = new Map();
  let idCountMap = new Map();
  let idPostsMap = new Map();
  let replyMap = new Map();
  let ngPostNumbers = new Set();

  // DOM Elements
  const favoritesList = document.getElementById('favoritesList');
  const bookmarksList = document.getElementById('bookmarksList');
  const historyList = document.getElementById('historyList');
  const allBoardsList = document.getElementById('allBoardsList');
  const favCountBadge = document.getElementById('favCountBadge');
  const bmCountBadge = document.getElementById('bmCountBadge');
  const historyCountBadge = document.getElementById('historyCountBadge');
  const allBoardsCountBadge = document.getElementById('allBoardsCountBadge');

  const currentBoardName = document.getElementById('currentBoardName');
  const btnToggleFavBoard = document.getElementById('btnToggleFavBoard');
  const threadSearchInput = document.getElementById('threadSearchInput');
  const threadTableBody = document.getElementById('threadTableBody');
  const threadCountBadge = document.getElementById('threadCountBadge');
  const emptyThreadsMessage = document.getElementById('emptyThreadsMessage');

  const threadTitle = document.getElementById('threadTitle');
  const threadBoardLabel = document.getElementById('threadBoardLabel');
  const threadResCountLabel = document.getElementById('threadResCountLabel');
  const autoReloadIndicator = document.getElementById('autoReloadIndicator');
  const btnRefreshContent = document.getElementById('btnRefreshContent');
  const selectAutoReload = document.getElementById('selectAutoReload');
  const btnBookmarkThread = document.getElementById('btnBookmarkThread');
  const bmStarIcon = document.getElementById('bmStarIcon');
  const bmLabel = document.getElementById('bmLabel');
  const postsContainer = document.getElementById('postsContainer');
  const postsList = document.getElementById('postsList');
  const anchorPopup = document.getElementById('anchorPopup');
  const toastNotification = document.getElementById('toastNotification');
  const toastText = document.getElementById('toastText');

  // Modals
  const idModalOverlay = document.getElementById('idModalOverlay');
  const idModalTitle = document.getElementById('idModalTitle');
  const idModalBody = document.getElementById('idModalBody');
  const btnCloseIdModal = document.getElementById('btnCloseIdModal');

  const ngModalOverlay = document.getElementById('ngModalOverlay');
  const btnCloseNGModal = document.getElementById('btnCloseNGModal');
  const btnOpenNGModal = document.getElementById('btnOpenNGModal');
  const ngModeSelect = document.getElementById('ngModeSelect');
  const chainAboneCheck = document.getElementById('chainAboneCheck');
  const inputNewNGId = document.getElementById('inputNewNGId');
  const btnAddNGId = document.getElementById('btnAddNGId');
  const ngIdChips = document.getElementById('ngIdChips');
  const inputNewNGWord = document.getElementById('inputNewNGWord');
  const btnAddNGWord = document.getElementById('btnAddNGWord');
  const ngWordChips = document.getElementById('ngWordChips');

  // Settings Modal Elements
  const settingsModalOverlay = document.getElementById('settingsModalOverlay');
  const btnCloseSettingsModal = document.getElementById('btnCloseSettingsModal');
  const btnOpenSettingsModal = document.getElementById('btnOpenSettingsModal');
  const btnOpenSettingsModalToolbar = document.getElementById('btnOpenSettingsModalToolbar');
  const btnResetSettings = document.getElementById('btnResetSettings');
  const btnSaveSettings = document.getElementById('btnSaveSettings');
  const selectUiFont = document.getElementById('selectUiFont');
  const groupCustomUiFont = document.getElementById('groupCustomUiFont');
  const inputCustomUiFont = document.getElementById('inputCustomUiFont');
  const rangeUiFontSize = document.getElementById('rangeUiFontSize');
  const valUiFontSize = document.getElementById('valUiFontSize');
  const selectPostFont = document.getElementById('selectPostFont');
  const groupCustomPostFont = document.getElementById('groupCustomPostFont');
  const inputCustomPostFont = document.getElementById('inputCustomPostFont');
  const rangePostFontSize = document.getElementById('rangePostFontSize');
  const valPostFontSize = document.getElementById('valPostFontSize');
  const rangePostLineHeight = document.getElementById('rangePostLineHeight');
  const valPostLineHeight = document.getElementById('valPostLineHeight');
  const checkDefaultBlurImages = document.getElementById('checkDefaultBlurImages');
  const selectInitialScroll = document.getElementById('selectInitialScroll');
  const rangeScrollAmount = document.getElementById('rangeScrollAmount');
  const valScrollAmount = document.getElementById('valScrollAmount');
  const inputDefaultName = document.getElementById('inputDefaultName');
  const inputDefaultMail = document.getElementById('inputDefaultMail');
  const settingsPreviewBox = document.getElementById('settingsPreviewBox');
  const previewPostBody = document.getElementById('previewPostBody');

  // Post Modal Elements
  const btnOpenPostModal = document.getElementById('btnOpenPostModal');
  const postModalOverlay = document.getElementById('postModalOverlay');
  const postModalTitle = document.getElementById('postModalTitle');
  const btnClosePostModal = document.getElementById('btnClosePostModal');
  const postConfirmNotice = document.getElementById('postConfirmNotice');
  const postConfirmMessage = document.getElementById('postConfirmMessage');
  const postErrorNotice = document.getElementById('postErrorNotice');
  const inputPostName = document.getElementById('inputPostName');
  const inputPostMail = document.getElementById('inputPostMail');
  const btnToggleSage = document.getElementById('btnToggleSage');
  const textareaPostBody = document.getElementById('textareaPostBody');
  const btnCancelPost = document.getElementById('btnCancelPost');
  const btnSubmitPost = document.getElementById('btnSubmitPost');
  const btnSubmitPostText = document.getElementById('btnSubmitPostText');
  const postStatusText = document.getElementById('postStatusText');

  // Splitters
  const splitterSidebar = document.getElementById('splitterSidebar');
  const sidebar = document.getElementById('sidebar');
  const splitterMain = document.getElementById('splitterMain');
  const paneThreads = document.getElementById('paneThreads');

  // Utility: Escape HTML
  function escapeHtml(str) {
    if (!str) return '';
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  // Utility: Decode Numeric Character References & Restore Corrupted 16-bit Emojis
  function decodeHtmlEntities(str) {
    if (!str) return '';
    return String(str)
      .replace(/&amp;#(x[0-9a-fA-F]+|\d+);|&#(x[0-9a-fA-F]+|\d+);/gi, (match, p1, p2) => {
        const raw = p1 || p2;
        const isHex = raw.toLowerCase().startsWith('x');
        let code = isHex ? parseInt(raw.slice(1), 16) : parseInt(raw, 10);
        if (isNaN(code)) return match;

        // XSS防止: 危険文字 (<, >, &, ", ', /) や制御文字はエスケープ状態を維持
        if (code === 60 || code === 62 || code === 38 || code === 34 || code === 39 || code === 47 || code < 32) {
          return match;
        }

        // 5ch 特有の 16bit 桁落ち絵文字の救済復元 (0xF000〜0xFFFF ➔ 0x1F000〜0x1FFFF)
        // 例: &#62978; (0xF602) + 0x10000 = 0x1F602 (😂)
        if (code >= 0xF000 && code <= 0xFFFF) {
          const shifted = code + 0x10000;
          if (shifted >= 0x1F000 && shifted <= 0x1FAFF) {
            try {
              return String.fromCodePoint(shifted);
            } catch (e) {
              // fallback
            }
          }
        }

        try {
          return String.fromCodePoint(code);
        } catch (e) {
          return match;
        }
      })
      .replace(/&amp;(nbsp|amp|lt|gt|quot);/gi, (match, entity) => {
        if (entity === 'nbsp') return ' ';
        return match;
      });
  }

  function formatSafeText(str) {
    if (!str) return '';
    return decodeHtmlEntities(escapeHtml(str));
  }

  function getCleanDate(dateStr) {
    if (!dateStr) return '';
    return String(dateStr).replace(/\s*ID:[^\s]+/g, '').trim();
  }

  // --- Initial Load ---
  async function init() {
    setupSplitters();
    setupKeybindings();
    setupTreeToggles();

    // 1. 板一覧を最優先で取得開始
    loadBBSMenu();

    // 2. ストレージ情報（お気に入り・履歴・NG設定・アプリ設定）を並行ロード
    try {
      let loadedSettings = null;
      let loadedReadPositions = {};
      [favorites, bookmarks, history, ngSettings, loadedSettings, loadedReadPositions] = await Promise.all([
        invokeTauri('get_favorites').catch(e => { console.warn("get_favorites failed:", e); return []; }),
        invokeTauri('get_bookmarks').catch(e => { console.warn("get_bookmarks failed:", e); return []; }),
        invokeTauri('get_history').catch(e => { console.warn("get_history failed:", e); return []; }),
        invokeTauri('get_ng_settings').catch(e => { console.warn("get_ng_settings failed:", e); return { ngWords: [], ngIds: [], ngMode: 'abone', chainAbone: true }; }),
        invokeTauri('get_app_settings').catch(e => { console.warn("get_app_settings failed:", e); return null; }),
        invokeTauri('get_read_positions').catch(e => { console.warn("get_read_positions failed:", e); return {}; }),
      ]);

      if (loadedReadPositions && typeof loadedReadPositions === 'object') {
        readPositions = loadedReadPositions;
      }

      if (loadedSettings) {
        appSettings = Object.assign({}, DEFAULT_APP_SETTINGS, loadedSettings);
      }
      applySettings(appSettings, false);
      setupSettingsUI();
      setupScrollPositionTracker();
      setupPostModalUI();

      renderFavorites();
      renderBookmarks();
      renderHistory();
      renderNGChips();
    } catch (e) {
      console.error("Storage loading error:", e);
    }
  }

  // --- BBSMenu & Boards ---
  async function loadBBSMenu() {
    allBoardsList.innerHTML = '<div class="loading-item">板一覧を取得中...</div>';
    try {
      categories = await invokeTauri('get_bbsmenu');
      let totalBoards = 0;
      categories.forEach(c => totalBoards += c.boards.length);
      allBoardsCountBadge.textContent = `${totalBoards} 板`;

      renderAllBoards();
    } catch (e) {
      console.error("loadBBSMenu failed:", e);
      allBoardsList.innerHTML = `
        <div class="loading-item" style="color:#ff6b6b; padding:12px;">
          <div>板一覧の取得に失敗しました:</div>
          <div style="font-size:11px; margin-top:4px; opacity:0.8;">${escapeHtml(String(e))}</div>
          <button id="btnRetryBbs" class="btn" style="margin-top:8px;">再試行</button>
        </div>`;
      document.getElementById('btnRetryBbs')?.addEventListener('click', loadBBSMenu);
    }
  }


  function renderAllBoards() {
    allBoardsList.innerHTML = categories.map((cat, idx) => `
      <div class="tree-category">
        <div class="tree-sub-header" data-idx="${idx}">
          <span class="tree-toggle">▶</span>
          <span>${escapeHtml(cat.name)}</span>
          <span class="tree-badge">${cat.boards.length}</span>
        </div>
        <div class="tree-sub-list collapsed" id="cat-list-${idx}">
          ${cat.boards.map(b => `
            <div class="tree-item" data-server="${escapeHtml(b.server)}" data-board="${escapeHtml(b.board)}" data-name="${escapeHtml(b.name)}" data-url="${escapeHtml(b.url)}">
              <span>💬</span> <span>${escapeHtml(b.name)}</span>
            </div>
          `).join('')}
        </div>
      </div>
    `).join('');

    allBoardsList.querySelectorAll('.tree-sub-header').forEach(header => {
      header.onclick = () => {
        const idx = header.getAttribute('data-idx');
        const list = document.getElementById(`cat-list-${idx}`);
        const toggle = header.querySelector('.tree-toggle');
        const isCollapsed = list.classList.toggle('collapsed');
        toggle.textContent = isCollapsed ? '▶' : '▼';
      };
    });

    allBoardsList.querySelectorAll('.tree-item').forEach(item => {
      item.onclick = () => {
        const board = {
          name: item.getAttribute('data-name'),
          server: item.getAttribute('data-server'),
          board: item.getAttribute('data-board'),
          url: item.getAttribute('data-url'),
        };
        selectBoard(board);
      };
    });
  }

  function renderFavorites() {
    favCountBadge.textContent = favorites.length;
    if (favorites.length === 0) {
      favoritesList.innerHTML = '<div class="loading-item">(お気に入りは空です)</div>';
      return;
    }
    favoritesList.innerHTML = favorites.map(b => `
      <div class="tree-item" data-server="${escapeHtml(b.server)}" data-board="${escapeHtml(b.board)}" data-name="${escapeHtml(b.name)}" data-url="${escapeHtml(b.url)}">
        <span>★</span> <span>${escapeHtml(b.name)}</span>
      </div>
    `).join('');

    favoritesList.querySelectorAll('.tree-item').forEach(item => {
      item.onclick = () => {
        const board = {
          name: item.getAttribute('data-name'),
          server: item.getAttribute('data-server'),
          board: item.getAttribute('data-board'),
          url: item.getAttribute('data-url'),
        };
        selectBoard(board);
      };
    });
  }

  function renderBookmarks() {
    bmCountBadge.textContent = bookmarks.length;
    if (bookmarks.length === 0) {
      bookmarksList.innerHTML = '<div class="loading-item">(ブックマークは空です)</div>';
      return;
    }
    bookmarksList.innerHTML = bookmarks.map(bm => `
      <div class="tree-item" data-server="${escapeHtml(bm.board.server)}" data-board="${escapeHtml(bm.board.board)}" data-id="${escapeHtml(bm.thread.id)}" title="${escapeHtml(bm.thread.title)}">
        <span>📌</span> <span style="overflow:hidden; text-overflow:ellipsis;">${formatSafeText(bm.thread.title)}</span>
      </div>
    `).join('');

    bookmarksList.querySelectorAll('.tree-item').forEach(item => {
      item.onclick = async () => {
        const server = item.getAttribute('data-server');
        const boardCode = item.getAttribute('data-board');
        const threadId = item.getAttribute('data-id');
        const bm = bookmarks.find(b => b.board.server === server && b.board.board === boardCode && b.thread.id === threadId);
        if (bm) {
          if (!currentBoard || currentBoard.server !== server || currentBoard.board !== boardCode) {
            await selectBoard(bm.board, false);
          }
          selectThread(bm.thread);
        }
      };
    });
  }

  function renderHistory() {
    historyCountBadge.textContent = history.length;
    if (history.length === 0) {
      historyList.innerHTML = '<div class="loading-item">(履歴は空です)</div>';
      return;
    }
    historyList.innerHTML = history.map(h => `
      <div class="tree-item" data-server="${escapeHtml(h.board.server)}" data-board="${escapeHtml(h.board.board)}" data-id="${escapeHtml(h.thread.id)}" title="${escapeHtml(h.thread.title)}">
        <span>🕒</span> <span style="overflow:hidden; text-overflow:ellipsis;">${formatSafeText(h.thread.title)}</span>
      </div>
    `).join('');

    historyList.querySelectorAll('.tree-item').forEach(item => {
      item.onclick = async () => {
        const server = item.getAttribute('data-server');
        const boardCode = item.getAttribute('data-board');
        const threadId = item.getAttribute('data-id');
        const h = history.find(b => b.board.server === server && b.board.board === boardCode && b.thread.id === threadId);
        if (h) {
          if (!currentBoard || currentBoard.server !== server || currentBoard.board !== boardCode) {
            await selectBoard(h.board, false);
          }
          selectThread(h.thread);
        }
      };
    });
  }

  // --- Select Board & Load Threads ---
  async function selectBoard(board, loadFirstThread = false) {
    currentBoard = board;
    currentBoardName.textContent = board.name;
    updateFavStarStatus();

    // Highlight active in sidebar
    document.querySelectorAll('.sidebar .tree-item').forEach(el => {
      const match = el.getAttribute('data-server') === board.server && el.getAttribute('data-board') === board.board;
      el.classList.toggle('active', match);
    });

    threadTableBody.innerHTML = `<tr><td colspan="4" class="empty-cell">「${escapeHtml(board.name)}」のスレッド一覧を取得中...</td></tr>`;

    try {
      rawThreads = await invokeTauri('get_thread_list', { server: board.server, board: board.board });
      renderThreadTable();
      setFocusedPane('threads');

      if (loadFirstThread && rawThreads.length > 0) {
        selectThread(rawThreads[0]);
      }
    } catch (e) {
      threadTableBody.innerHTML = `<tr><td colspan="4" class="empty-cell" style="color:#ff6b6b;">スレッド一覧の取得に失敗: ${escapeHtml(e)}</td></tr>`;
    }
  }

  function updateFavStarStatus() {
    if (!currentBoard) {
      btnToggleFavBoard.classList.remove('active');
      btnToggleFavBoard.textContent = '☆';
      return;
    }
    const isFav = favorites.some(b => b.server === currentBoard.server && b.board === currentBoard.board);
    btnToggleFavBoard.classList.toggle('active', isFav);
    btnToggleFavBoard.textContent = isFav ? '★' : '☆';
  }

  btnToggleFavBoard.onclick = async () => {
    if (!currentBoard) return;
    const isFav = favorites.some(b => b.server === currentBoard.server && b.board === currentBoard.board);
    if (isFav) {
      await invokeTauri('remove_favorite', { server: currentBoard.server, board: currentBoard.board });
    } else {
      await invokeTauri('add_favorite', { board: currentBoard });
    }
    favorites = await invokeTauri('get_favorites');
    renderFavorites();
    updateFavStarStatus();
  };

  // --- Render Threads Table ---
  function renderThreadTable() {
    let filtered = rawThreads.filter(t => {
      if (!filterText) return true;
      return t.title.toLowerCase().includes(filterText.toLowerCase());
    });

    if (currentSort === 'ikioi') {
      filtered.sort((a, b) => b.ikioi - a.ikioi);
    } else if (currentSort === 'res') {
      filtered.sort((a, b) => b.resCount - a.resCount);
    } else if (currentSort === 'new') {
      filtered.sort((a, b) => parseInt(b.id, 10) - parseInt(a.id, 10));
    } else if (currentSort === 'default') {
      const orderMap = new Map(rawThreads.map((t, idx) => [t.id, idx]));
      filtered.sort((a, b) => (orderMap.get(a.id) || 0) - (orderMap.get(b.id) || 0));
    }

    threadCountBadge.textContent = `${filtered.length} / ${rawThreads.length} スレッド`;

    if (filtered.length === 0) {
      threadTableBody.innerHTML = '';
      emptyThreadsMessage.style.display = 'block';
      return;
    }
    emptyThreadsMessage.style.display = 'none';

    threadTableBody.innerHTML = filtered.map((t, idx) => {
      const isSelected = currentThread && currentThread.id === t.id;
      let ikioiClass = '';
      if (t.ikioi >= 1000) ikioiClass = ' very-hot';
      else if (t.ikioi >= 300) ikioiClass = ' hot';

      return `
        <tr class="thread-row${isSelected ? ' selected' : ''}" data-id="${escapeHtml(t.id)}">
          <td class="thread-index">${idx + 1}</td>
          <td class="thread-title-cell">${formatSafeText(t.title)}</td>
          <td class="res-count">${t.resCount}</td>
          <td class="ikioi${ikioiClass}">${t.ikioi}</td>
        </tr>
      `;
    }).join('');

    threadTableBody.querySelectorAll('.thread-row').forEach(row => {
      row.onclick = () => {
        const id = row.getAttribute('data-id');
        const t = rawThreads.find(item => item.id === id);
        if (t) {
          selectThread(t);
          setFocusedPane('content');
        }
      };
    });
  }

  // --- Select Thread & Load Posts ---
  async function selectThread(thread) {
    currentThread = thread;
    const decodedTitle = decodeHtmlEntities(thread.title);
    threadTitle.textContent = decodedTitle;
    threadTitle.title = decodedTitle;
    threadBoardLabel.textContent = `板: ${currentBoard ? currentBoard.name : '-'}`;
    threadResCountLabel.textContent = `${thread.resCount} レス`;

    // Highlight row in table
    threadTableBody.querySelectorAll('.thread-row').forEach(r => {
      r.classList.toggle('selected', r.getAttribute('data-id') === thread.id);
    });

    updateBookmarkButtonStatus();

    // Add to History
    if (currentBoard) {
      invokeTauri('add_history', { board: currentBoard, thread }).then(async () => {
        history = await invokeTauri('get_history');
        renderHistory();
      });
    }

    postsList.innerHTML = `<div class="empty-content-message">スレッド「${formatSafeText(thread.title)}」を読み込み中...</div>`;

    try {
      const content = await invokeTauri('get_thread_posts', {
        server: currentBoard.server,
        board: currentBoard.board,
        key: thread.id,
      });

      if (content.title) {
        const decodedContentTitle = decodeHtmlEntities(content.title);
        threadTitle.textContent = decodedContentTitle;
        threadTitle.title = decodedContentTitle;
      }
      rawPosts = content.posts;
      renderAllPosts();
      if (currentBoard && currentThread) {
        const threadKey = `${currentBoard.server}_${currentBoard.board}_${currentThread.id}`;
        applyInitialScroll(threadKey);
      }
    } catch (e) {
      postsList.innerHTML = `<div class="empty-content-message" style="color:#ff6b6b;">スレッドの読み込みに失敗: ${escapeHtml(e)}</div>`;
    }
  }


  // --- Bookmark Thread ---
  function updateBookmarkButtonStatus() {
    if (!currentBoard || !currentThread) {
      btnBookmarkThread.classList.remove('active');
      bmStarIcon.textContent = '☆';
      bmLabel.textContent = '保存';
      return;
    }
    const isBm = bookmarks.some(b => b.board.server === currentBoard.server && b.board.board === currentBoard.board && b.thread.id === currentThread.id);
    btnBookmarkThread.classList.toggle('active', isBm);
    bmStarIcon.textContent = isBm ? '★' : '☆';
    bmLabel.textContent = isBm ? '保存済み' : '保存';
  }

  btnBookmarkThread.onclick = async () => {
    if (!currentBoard || !currentThread) return;
    const isBm = bookmarks.some(b => b.board.server === currentBoard.server && b.board.board === currentBoard.board && b.thread.id === currentThread.id);
    if (isBm) {
      await invokeTauri('remove_bookmark', { server: currentBoard.server, board: currentBoard.board, threadId: currentThread.id });
    } else {
      await invokeTauri('add_bookmark', { board: currentBoard, thread: currentThread });
    }
    bookmarks = await invokeTauri('get_bookmarks');
    renderBookmarks();
    updateBookmarkButtonStatus();
  };

  // --- Posts Rendering & Calculation ---
  function extractAnchorTargets(bodyText) {
    const targets = new Set();
    if (!bodyText) return [];
    const hrefRegex = /<a\s+[^>]*?href=["'][^"']*?\/(\d+)(?:-\d+)?["'][^>]*>/gi;
    let m;
    while ((m = hrefRegex.exec(bodyText)) !== null) {
      const num = parseInt(m[1], 10);
      if (!isNaN(num) && num > 0) targets.add(num);
    }
    const textRegex = /(?:&gt;|[>＞]){1,2}(\d+)(?:-\d+)?/g;
    while ((m = textRegex.exec(bodyText)) !== null) {
      const num = parseInt(m[1], 10);
      if (!isNaN(num) && num > 0) targets.add(num);
    }
    return Array.from(targets);
  }

  function recalculateMaps() {
    postsMap.clear();
    idCountMap.clear();
    idPostsMap.clear();
    replyMap.clear();
    ngPostNumbers.clear();

    rawPosts.forEach(p => {
      postsMap.set(p.number, p);

      if (p.id) {
        idCountMap.set(p.id, (idCountMap.get(p.id) || 0) + 1);
        if (!idPostsMap.has(p.id)) idPostsMap.set(p.id, []);
        idPostsMap.get(p.id).push(p);
      }

      const targets = extractAnchorTargets(p.body);
      for (const target of targets) {
        if (target !== p.number) {
          if (!replyMap.has(target)) replyMap.set(target, []);
          if (!replyMap.get(target).includes(p.number)) {
            replyMap.get(target).push(p.number);
          }
        }
      }

      let isNG = false;
      if (p.id && ngSettings.ngIds && ngSettings.ngIds.includes(p.id)) isNG = true;
      if (!isNG && ngSettings.ngWords && ngSettings.ngWords.some(w => (p.body && p.body.includes(w)) || (p.name && p.name.includes(w)))) isNG = true;

      if (isNG) ngPostNumbers.add(p.number);
    });

    // Chain Abone
    if (ngSettings.chainAbone) {
      let changed = true;
      while (changed) {
        changed = false;
        rawPosts.forEach(p => {
          if (ngPostNumbers.has(p.number)) return;
          const targets = extractAnchorTargets(p.body);
          for (const target of targets) {
            if (ngPostNumbers.has(target)) {
              ngPostNumbers.add(p.number);
              changed = true;
              break;
            }
          }
        });
      }
    }
  }

  function renderAllPosts() {
    recalculateMaps();
    threadResCountLabel.textContent = `${rawPosts.length} レス`;
    postsList.innerHTML = rawPosts.map(p => createPostHtml(p)).join('');
    attachPostEvents(postsList);
  }

  function applyInitialScroll(threadKey) {
    const mode = appSettings.initialScrollPosition || 'top';
    requestAnimationFrame(() => {
      if (mode === 'bottom') {
        postsContainer.scrollTop = postsContainer.scrollHeight;
      } else if (mode === 'lastRead') {
        const lastNum = readPositions[threadKey];
        if (lastNum && lastNum > 1) {
          const targetEl = document.getElementById(`post-${lastNum}`);
          if (targetEl) {
            targetEl.scrollIntoView({ behavior: 'auto', block: 'start' });
            return;
          }
        }
        postsContainer.scrollTop = 0;
      } else {
        postsContainer.scrollTop = 0;
      }
    });
  }

  function setupScrollPositionTracker() {
    postsContainer.addEventListener('scroll', () => {
      if (!currentBoard || !currentThread) return;
      const threadKey = `${currentBoard.server}_${currentBoard.board}_${currentThread.id}`;
      if (saveReadPosTimeout) clearTimeout(saveReadPosTimeout);
      saveReadPosTimeout = setTimeout(() => {
        const topNum = getTopVisiblePostNumber();
        if (topNum > 0) {
          readPositions[threadKey] = topNum;
          invokeTauri('save_read_position', { key: threadKey, resNumber: topNum }).catch(e => {
            console.warn("Failed to save read position:", e);
          });
        }
      }, 350);
    });
  }

  function getTopVisiblePostNumber() {
    const posts = postsContainer.querySelectorAll('.post:not(.transparent-ng)');
    const containerTop = postsContainer.getBoundingClientRect().top;
    for (const p of posts) {
      const rect = p.getBoundingClientRect();
      if (rect.bottom > containerTop + 20) {
        const num = parseInt(p.id.replace('post-', ''), 10);
        if (!isNaN(num)) return num;
      }
    }
    return 1;
  }

  function createPostHtml(p, isNew = false) {
    const isNG = ngPostNumbers.has(p.number);
    const isTransparent = isNG && ngSettings.ngMode === 'transparent';

    if (isTransparent) {
      return `<div class="post transparent-ng" id="post-${p.number}" style="display:none;"></div>`;
    }

    if (isNG) {
      return `
        <div class="post abone-item${isNew ? ' highlight' : ''}" id="post-${p.number}">
          <div class="abone-notice">
            <span>🚫 レス ${p.number} はNG指定により非表示です</span>
            <button class="abone-toggle-btn" data-action="toggle-abone" data-num="${p.number}">[表示する]</button>
          </div>
          <div class="abone-content" id="abone-content-${p.number}" style="display:none;">
            ${createNormalPostInnerHtml(p)}
          </div>
        </div>
      `;
    }

    return `
      <div class="post${isNew ? ' highlight' : ''}" id="post-${p.number}">
        ${createNormalPostInnerHtml(p)}
      </div>
    `;
  }

  function createNormalPostInnerHtml(p) {
    const idCount = p.id ? (idCountMap.get(p.id) || 1) : 0;
    let idCountClass = '';
    if (idCount >= 5) idCountClass = ' very-hot';
    else if (idCount >= 3) idCountClass = ' hot';

    const replies = replyMap.get(p.number) || [];
    const hasReplies = replies.length > 0;
    const isHotPost = replies.length >= 4;

    const formattedBody = formatBody(p.body, blurImages);

    return `
      <div class="post-header">
        <span class="num">${p.number}</span>
        <span class="name"><b>${formatSafeText(p.name)}</b></span>
        ${p.mail ? `<span class="mail">[${formatSafeText(p.mail)}]</span>` : ''}
        <span class="date">${escapeHtml(getCleanDate(p.date))}</span>

        ${p.id ? `
          <span class="id-container" data-id="${escapeHtml(p.id)}" title="クリックでこのIDの発言を抽出">
            <span class="id-text">ID:${escapeHtml(p.id)}</span>
            <span class="id-count${idCountClass}">${idCount}</span>
          </span>
        ` : ''}

        ${hasReplies ? `
          <span class="reply-tree">
            <span class="reply-tree-label">💬 返信(${replies.length}):</span>
            ${replies.map(r => `<span class="reply-anchor anchor" data-target="${r}">&gt;&gt;${r}</span>`).join(' ')}
          </span>
        ` : ''}

        ${isHotPost ? '<span class="hot-badge">🔥 注目</span>' : ''}

        <div class="post-actions">
          <button class="action-icon-btn reply-btn" data-action="reply-post" data-num="${p.number}" title=">>${p.number} に返信">💬 返信</button>
          ${p.id ? `<button class="action-icon-btn ng-btn" data-action="ng-id" data-id="${escapeHtml(p.id)}" title="このIDをNG登録">🚫 NG</button>` : ''}
          <button class="action-icon-btn" data-action="copy-res" data-num="${p.number}" title=">>${p.number} をコピー">📋 コピー</button>
        </div>
      </div>
      <div class="post-body">${formattedBody}</div>
    `;
  }

  function formatBody(rawBody, blur) {
    if (!rawBody) return '';
    const tokens = [];
    const createToken = (html) => {
      const idx = tokens.length;
      tokens.push(html);
      return `\x00TOKEN_${idx}\x00`;
    };

    const lines = rawBody.split(/<br\s*\/?>/gi);
    const formattedLines = lines.map(line => {
      let text = line;

      // 1. Existing anchor tags
      text = text.replace(/<a\s+[^>]*?href=["'][^"']*?\/(\d+)(?:-\d+)?["'][^>]*>(?:&gt;|[>＞]){1,2}(\d+(?:-\d+)?)<\/a>/gi, (m, targetNum, displayNum) => {
        return createToken(`<span class="anchor" data-target="${targetNum}">&gt;&gt;${displayNum}</span>`);
      });

      // 2. Existing image tags
      text = text.replace(/<a\s+href=["']([^"']+)["'][^>]*>(.*?)<\/a>/gi, (m, url, label) => {
        if (url.match(/\.(?:jpg|jpeg|png|gif|webp)(?:\?.*)?$/i)) {
          const blurClass = blur ? ' blurred' : '';
          return createToken(`<div class="image-wrapper"><div class="image-blur-container${blurClass}" data-action="toggle-blur" title="クリックでモザイク切替"><img class="thumbnail" src="${url}" loading="lazy" /><div class="blur-overlay"><div class="blur-badge">👁️ 表示</div></div></div><a href="${url}" target="_blank" class="image-link-btn">🔗 元画像</a></div>`);
        }
        return createToken(`<a href="${url}" target="_blank" style="color:var(--highlight-color);">${label || url}</a>`);
      });

      // 3. Strip remaining tags safely
      text = text.replace(/<[^>]*>/g, '');
      text = escapeHtml(text);
      text = decodeHtmlEntities(text);

      // 4. Plain anchors
      text = text.replace(/(?:&gt;|[>＞]){1,2}(\d+)(?:-(\d+))?/g, (m, num, endNum) => {
        const display = endNum ? `${num}-${endNum}` : num;
        return createToken(`<span class="anchor" data-target="${num}">&gt;&gt;${display}</span>`);
      });

      // 5. Plain image URLs
      text = text.replace(/(https?:\/\/[^\s<]+?\.(?:jpg|jpeg|png|gif|webp)(?:\?[^\s<]*)?)/gi, (m, url) => {
        const blurClass = blur ? ' blurred' : '';
        return createToken(`<div class="image-wrapper"><div class="image-blur-container${blurClass}" data-action="toggle-blur" title="クリックでモザイク切替"><img class="thumbnail" src="${url}" loading="lazy" /><div class="blur-overlay"><div class="blur-badge">👁️ 表示</div></div></div><a href="${url}" target="_blank" class="image-link-btn">🔗 元画像</a></div>`);
      });

      // 6. Plain general URLs
      text = text.replace(/(https?:\/\/[^\s<]+)/gi, (m, url) => {
        return createToken(`<a href="${url}" target="_blank" style="color:var(--highlight-color);">${url}</a>`);
      });

      return text;
    });

    let res = formattedLines.join('<br>');
    res = res.replace(/\x00TOKEN_(\d+)\x00/g, (m, idxStr) => tokens[parseInt(idxStr, 10)] || '');
    return res;
  }

  function attachPostEvents(container) {
    // Anchor Clicks & Hover Popup
    container.querySelectorAll('.anchor').forEach(anchor => {
      const targetNum = parseInt(anchor.getAttribute('data-target'), 10);

      anchor.onclick = (e) => {
        e.preventDefault();
        scrollToPost(targetNum);
      };

      anchor.onmouseenter = () => {
        const post = postsMap.get(targetNum);
        if (post && anchorPopup) {
          anchorPopup.innerHTML = `
            <div style="font-size:0.85em; color:var(--fg-muted); margin-bottom:6px;">
              <b>${post.number}</b> ${escapeHtml(post.name)} ${escapeHtml(getCleanDate(post.date))} ${post.id ? 'ID:' + escapeHtml(post.id) : ''}
            </div>
            <div>${formatBody(post.body, blurImages)}</div>
          `;
          anchorPopup.style.display = 'block';

          const rect = anchor.getBoundingClientRect();
          let top = rect.bottom + 6;
          let left = rect.left;
          if (left + 360 > window.innerWidth) left = Math.max(10, window.innerWidth - 370);
          if (top + anchorPopup.offsetHeight > window.innerHeight) top = Math.max(10, rect.top - anchorPopup.offsetHeight - 6);

          anchorPopup.style.top = top + 'px';
          anchorPopup.style.left = left + 'px';
        }
      };

      anchor.onmouseleave = () => {
        if (anchorPopup) anchorPopup.style.display = 'none';
      };
    });

    // Toggle Abone
    container.querySelectorAll('[data-action="toggle-abone"]').forEach(btn => {
      btn.onclick = () => {
        const num = btn.getAttribute('data-num');
        const el = document.getElementById('abone-content-' + num);
        if (el) el.style.display = el.style.display === 'none' ? 'block' : 'none';
      };
    });

    // Toggle Blur
    container.querySelectorAll('[data-action="toggle-blur"]').forEach(el => {
      el.onclick = () => el.classList.toggle('blurred');
    });

    // ID Click -> Open ID Modal
    container.querySelectorAll('.id-container').forEach(el => {
      el.onclick = () => {
        const id = el.getAttribute('data-id');
        openIdModal(id);
      };
    });

    // Quick Action: Reply Post
    container.querySelectorAll('[data-action="reply-post"]').forEach(btn => {
      btn.onclick = (e) => {
        e.stopPropagation();
        const num = btn.getAttribute('data-num');
        openPostModal(`>>${num}\n`);
      };
    });

    // Quick Action: NG ID
    container.querySelectorAll('[data-action="ng-id"]').forEach(btn => {
      btn.onclick = async (e) => {
        e.stopPropagation();
        const id = btn.getAttribute('data-id');
        if (id && !ngSettings.ngIds.includes(id)) {
          ngSettings.ngIds.push(id);
          await invokeTauri('save_ng_settings', { settings: ngSettings });
          renderNGChips();
          renderAllPosts();
        }
      };
    });

    // Quick Action: Copy Res Number
    container.querySelectorAll('[data-action="copy-res"]').forEach(btn => {
      btn.onclick = async (e) => {
        e.stopPropagation();
        const num = btn.getAttribute('data-num');
        navigator.clipboard.writeText(`>>${num}`);
      };
    });
  }

  function scrollToPost(num) {
    const el = document.getElementById('post-' + num);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' });
      el.classList.add('highlight');
      setTimeout(() => el.classList.remove('highlight'), 2000);
    }
  }

  // --- ID Modal ---
  function openIdModal(id) {
    const posts = idPostsMap.get(id) || [];
    idModalTitle.textContent = `ID: ${id} の発言一覧 (${posts.length} 件)`;
    idModalBody.innerHTML = posts.map(p => `
      <div class="post" style="border-bottom:1px solid var(--border-color); padding:10px 0;">
        <div class="post-header">
          <span class="num" data-action="modal-jump" data-num="${p.number}" style="cursor:pointer; color:var(--anchor-color); font-weight:bold;" title="スレッド内のレスへジャンプ">${p.number} ➔</span>
          <span class="name">${escapeHtml(p.name)}</span>
          <span class="date">${escapeHtml(getCleanDate(p.date))}</span>
        </div>
        <div class="post-body">${formatBody(p.body, blurImages)}</div>
      </div>
    `).join('');

    idModalBody.querySelectorAll('[data-action="modal-jump"]').forEach(btn => {
      btn.onclick = () => {
        const num = parseInt(btn.getAttribute('data-num'), 10);
        scrollToPost(num);
        idModalOverlay.classList.remove('open');
      };
    });

    idModalOverlay.classList.add('open');
  }

  btnCloseIdModal.onclick = () => idModalOverlay.classList.remove('open');
  idModalOverlay.onclick = (e) => { if (e.target === idModalOverlay) idModalOverlay.classList.remove('open'); };

  // --- NG Settings Modal ---
  function openNGModal() {
    ngModeSelect.value = ngSettings.ngMode;
    chainAboneCheck.checked = ngSettings.chainAbone;
    renderNGChips();
    ngModalOverlay.classList.add('open');
  }

  function renderNGChips() {
    ngIdChips.innerHTML = (ngSettings.ngIds || []).map(id => `
      <span class="chip">
        <span>ID:${escapeHtml(id)}</span>
        <span class="chip-remove" data-action="remove-ng-id" data-id="${escapeHtml(id)}">&times;</span>
      </span>
    `).join('') || '<span style="color:var(--fg-muted); font-size:0.85em;">(登録なし)</span>';

    ngIdChips.querySelectorAll('[data-action="remove-ng-id"]').forEach(btn => {
      btn.onclick = async () => {
        const id = btn.getAttribute('data-id');
        ngSettings.ngIds = ngSettings.ngIds.filter(item => item !== id);
        await invokeTauri('save_ng_settings', { settings: ngSettings });
        renderNGChips();
        renderAllPosts();
      };
    });

    ngWordChips.innerHTML = (ngSettings.ngWords || []).map(w => `
      <span class="chip">
        <span>${escapeHtml(w)}</span>
        <span class="chip-remove" data-action="remove-ng-word" data-word="${escapeHtml(w)}">&times;</span>
      </span>
    `).join('') || '<span style="color:var(--fg-muted); font-size:0.85em;">(登録なし)</span>';

    ngWordChips.querySelectorAll('[data-action="remove-ng-word"]').forEach(btn => {
      btn.onclick = async () => {
        const word = btn.getAttribute('data-word');
        ngSettings.ngWords = ngSettings.ngWords.filter(item => item !== word);
        await invokeTauri('save_ng_settings', { settings: ngSettings });
        renderNGChips();
        renderAllPosts();
      };
    });
  }

  btnAddNGId.onclick = async () => {
    const id = inputNewNGId.value.trim();
    if (id && !ngSettings.ngIds.includes(id)) {
      ngSettings.ngIds.push(id);
      await invokeTauri('save_ng_settings', { settings: ngSettings });
      inputNewNGId.value = '';
      renderNGChips();
      renderAllPosts();
    }
  };

  btnAddNGWord.onclick = async () => {
    const word = inputNewNGWord.value.trim();
    if (word && !ngSettings.ngWords.includes(word)) {
      ngSettings.ngWords.push(word);
      await invokeTauri('save_ng_settings', { settings: ngSettings });
      inputNewNGWord.value = '';
      renderNGChips();
      renderAllPosts();
    }
  };

  ngModeSelect.onchange = async () => {
    ngSettings.ngMode = ngModeSelect.value;
    await invokeTauri('save_ng_settings', { settings: ngSettings });
    renderAllPosts();
  };

  chainAboneCheck.onchange = async () => {
    ngSettings.chainAbone = chainAboneCheck.checked;
    await invokeTauri('save_ng_settings', { settings: ngSettings });
    renderAllPosts();
  };

  btnOpenNGModal.onclick = openNGModal;
  btnCloseNGModal.onclick = () => ngModalOverlay.classList.remove('open');
  ngModalOverlay.onclick = (e) => { if (e.target === ngModalOverlay) ngModalOverlay.classList.remove('open'); };

  // --- Settings Modal & Application Configuration ---
  function applySettings(settings, save = true) {
    if (!settings) return;
    appSettings = Object.assign({}, DEFAULT_APP_SETTINGS, settings);

    // Apply CSS Variables to Document Root
    const rootStyle = document.documentElement.style;
    rootStyle.setProperty('--ui-font-family', appSettings.uiFontFamily);
    rootStyle.setProperty('--ui-font-size', `${appSettings.uiFontSize}px`);

    const effectivePostFont = appSettings.postFontFamily && appSettings.postFontFamily.trim() !== ''
      ? appSettings.postFontFamily
      : appSettings.uiFontFamily;
    rootStyle.setProperty('--post-font-family', effectivePostFont);
    rootStyle.setProperty('--post-font-size', `${appSettings.postFontSize}px`);
    rootStyle.setProperty('--post-line-height', String(appSettings.postLineHeight));

    // Update internal state
    blurImages = appSettings.defaultBlurImages ?? true;

    // Update Live Preview box in Settings modal
    updateSettingsPreview();

    // Sync input controls with appSettings
    syncSettingsControls();

    if (save) {
      debounceSaveSettings();
    }
  }

  function updateSettingsPreview() {
    if (!settingsPreviewBox) return;
    const effectivePostFont = appSettings.postFontFamily && appSettings.postFontFamily.trim() !== ''
      ? appSettings.postFontFamily
      : appSettings.uiFontFamily;

    const uiSample = settingsPreviewBox.querySelector('.preview-ui-sample');
    if (uiSample) {
      uiSample.style.fontFamily = appSettings.uiFontFamily;
      uiSample.style.fontSize = `${appSettings.uiFontSize}px`;
    }

    if (previewPostBody) {
      previewPostBody.style.fontFamily = effectivePostFont;
      previewPostBody.style.fontSize = `${appSettings.postFontSize}px`;
      previewPostBody.style.lineHeight = String(appSettings.postLineHeight);
    }
  }

  function syncSettingsControls() {
    if (!selectUiFont) return;

    // UI Font
    const uiOptions = Array.from(selectUiFont.options).map(o => o.value);
    if (uiOptions.includes(appSettings.uiFontFamily)) {
      selectUiFont.value = appSettings.uiFontFamily;
      groupCustomUiFont.style.display = 'none';
    } else {
      selectUiFont.value = 'custom';
      groupCustomUiFont.style.display = 'block';
      inputCustomUiFont.value = appSettings.uiFontFamily;
    }

    rangeUiFontSize.value = appSettings.uiFontSize;
    valUiFontSize.textContent = `${appSettings.uiFontSize}px`;

    // Post Font
    const postOptions = Array.from(selectPostFont.options).map(o => o.value);
    if (postOptions.includes(appSettings.postFontFamily)) {
      selectPostFont.value = appSettings.postFontFamily;
      groupCustomPostFont.style.display = 'none';
    } else {
      selectPostFont.value = 'custom';
      groupCustomPostFont.style.display = 'block';
      inputCustomPostFont.value = appSettings.postFontFamily;
    }

    rangePostFontSize.value = appSettings.postFontSize;
    valPostFontSize.textContent = `${appSettings.postFontSize}px`;

    rangePostLineHeight.value = appSettings.postLineHeight;
    valPostLineHeight.textContent = Number(appSettings.postLineHeight).toFixed(2);

    // General
    checkDefaultBlurImages.checked = appSettings.defaultBlurImages;
    if (selectInitialScroll) selectInitialScroll.value = appSettings.initialScrollPosition || 'top';
    rangeScrollAmount.value = appSettings.scrollAmount;
    valScrollAmount.textContent = `${appSettings.scrollAmount}px`;
    if (inputDefaultName) inputDefaultName.value = appSettings.defaultName || '';
    if (inputDefaultMail) inputDefaultMail.value = appSettings.defaultMail !== undefined ? appSettings.defaultMail : 'sage';
  }

  function debounceSaveSettings() {
    if (saveSettingsTimeout) {
      clearTimeout(saveSettingsTimeout);
    }
    saveSettingsTimeout = setTimeout(async () => {
      try {
        await invokeTauri('save_app_settings', { settings: appSettings });
      } catch (err) {
        console.error("Failed to save app settings:", err);
      }
    }, 300);
  }

  function openSettingsModal() {
    syncSettingsControls();
    updateSettingsPreview();
    settingsModalOverlay.classList.add('open');
  }

  function closeSettingsModal() {
    settingsModalOverlay.classList.remove('open');
    if (saveSettingsTimeout) {
      clearTimeout(saveSettingsTimeout);
    }
    invokeTauri('save_app_settings', { settings: appSettings }).catch(e => console.error(e));
  }

  function setupSettingsUI() {
    // Open & Close
    if (btnOpenSettingsModal) btnOpenSettingsModal.onclick = openSettingsModal;
    if (btnOpenSettingsModalToolbar) btnOpenSettingsModalToolbar.onclick = openSettingsModal;
    if (btnCloseSettingsModal) btnCloseSettingsModal.onclick = closeSettingsModal;
    if (btnSaveSettings) btnSaveSettings.onclick = closeSettingsModal;
    if (settingsModalOverlay) {
      settingsModalOverlay.onclick = (e) => {
        if (e.target === settingsModalOverlay) closeSettingsModal();
      };
    }

    // Tabs
    const tabBtns = settingsModalOverlay.querySelectorAll('.settings-tab-btn');
    const tabContents = settingsModalOverlay.querySelectorAll('.settings-tab-content');
    tabBtns.forEach(btn => {
      btn.onclick = () => {
        const targetId = btn.getAttribute('data-tab');
        tabBtns.forEach(b => b.classList.toggle('active', b === btn));
        tabContents.forEach(c => c.classList.toggle('active', c.id === targetId));
      };
    });

    // UI Font Select & Input
    selectUiFont.onchange = () => {
      if (selectUiFont.value === 'custom') {
        groupCustomUiFont.style.display = 'block';
        if (!inputCustomUiFont.value) inputCustomUiFont.value = appSettings.uiFontFamily;
        appSettings.uiFontFamily = inputCustomUiFont.value.trim() || DEFAULT_APP_SETTINGS.uiFontFamily;
      } else {
        groupCustomUiFont.style.display = 'none';
        appSettings.uiFontFamily = selectUiFont.value;
      }
      applySettings(appSettings, true);
    };

    inputCustomUiFont.oninput = () => {
      if (selectUiFont.value === 'custom') {
        appSettings.uiFontFamily = inputCustomUiFont.value.trim() || DEFAULT_APP_SETTINGS.uiFontFamily;
        applySettings(appSettings, true);
      }
    };

    // UI Font Size Slider
    rangeUiFontSize.oninput = () => {
      appSettings.uiFontSize = parseInt(rangeUiFontSize.value, 10);
      valUiFontSize.textContent = `${appSettings.uiFontSize}px`;
      applySettings(appSettings, true);
    };

    // Post Font Select & Input
    selectPostFont.onchange = () => {
      if (selectPostFont.value === 'custom') {
        groupCustomPostFont.style.display = 'block';
        if (!inputCustomPostFont.value) inputCustomPostFont.value = appSettings.postFontFamily;
        appSettings.postFontFamily = inputCustomPostFont.value.trim();
      } else {
        groupCustomPostFont.style.display = 'none';
        appSettings.postFontFamily = selectPostFont.value;
      }
      applySettings(appSettings, true);
    };

    inputCustomPostFont.oninput = () => {
      if (selectPostFont.value === 'custom') {
        appSettings.postFontFamily = inputCustomPostFont.value.trim();
        applySettings(appSettings, true);
      }
    };

    // Post Font Size Slider
    rangePostFontSize.oninput = () => {
      appSettings.postFontSize = parseInt(rangePostFontSize.value, 10);
      valPostFontSize.textContent = `${appSettings.postFontSize}px`;
      applySettings(appSettings, true);
    };

    // Post Line Height Slider
    rangePostLineHeight.oninput = () => {
      appSettings.postLineHeight = parseFloat(rangePostLineHeight.value);
      valPostLineHeight.textContent = Number(appSettings.postLineHeight).toFixed(2);
      applySettings(appSettings, true);
    };

    // Image Blur Checkbox
    checkDefaultBlurImages.onchange = () => {
      appSettings.defaultBlurImages = checkDefaultBlurImages.checked;
      applySettings(appSettings, true);
      renderAllPosts();
    };

    // Initial Scroll Position Select
    if (selectInitialScroll) {
      selectInitialScroll.onchange = () => {
        appSettings.initialScrollPosition = selectInitialScroll.value;
        applySettings(appSettings, true);
      };
    }

    // Scroll Amount Slider
    rangeScrollAmount.oninput = () => {
      appSettings.scrollAmount = parseInt(rangeScrollAmount.value, 10);
      valScrollAmount.textContent = `${appSettings.scrollAmount}px`;
      applySettings(appSettings, true);
    };

    // Default Name & Mail
    if (inputDefaultName) {
      inputDefaultName.oninput = () => {
        appSettings.defaultName = inputDefaultName.value.trim();
        debounceSaveSettings();
      };
    }
    if (inputDefaultMail) {
      inputDefaultMail.oninput = () => {
        appSettings.defaultMail = inputDefaultMail.value.trim();
        debounceSaveSettings();
      };
    }

    // Reset to Default
    btnResetSettings.onclick = () => {
      if (confirm('すべての設定を初期値に戻しますか？')) {
        applySettings(DEFAULT_APP_SETTINGS, true);
        renderAllPosts();
      }
    };
  }

  // --- Post Compose Modal UI ---
  function openPostModal(initialBody = '') {
    if (!currentBoard || !currentThread) {
      alert('書き込むスレッドを選択してください。');
      return;
    }

    postModalTitle.textContent = `✍️ レス書き込み - ${currentThread.title}`;
    inputPostName.value = appSettings.defaultName || '';
    inputPostMail.value = appSettings.defaultMail !== undefined ? appSettings.defaultMail : 'sage';

    if (initialBody) {
      textareaPostBody.value = initialBody;
    }

    postConfirmNotice.style.display = 'none';
    postErrorNotice.style.display = 'none';
    postStatusText.textContent = '';
    btnSubmitPostText.textContent = '書き込む (Ctrl+Enter)';
    btnSubmitPost.disabled = false;
    pendingConfirmParams = null;
    isPosting = false;

    postModalOverlay.classList.add('open');
    setTimeout(() => {
      textareaPostBody.focus();
      textareaPostBody.setSelectionRange(textareaPostBody.value.length, textareaPostBody.value.length);
    }, 60);
  }

  function closePostModal() {
    postModalOverlay.classList.remove('open');
    pendingConfirmParams = null;
    isPosting = false;
    postStatusText.textContent = '';
  }

  async function submitPost(confirmParams = null) {
    if (isPosting) return;
    if (!currentBoard || !currentThread) {
      alert('スレッドが選択されていません。');
      return;
    }

    const body = textareaPostBody.value.trim();
    if (!body) {
      postErrorNotice.textContent = '本文を入力してください。';
      postErrorNotice.style.display = 'block';
      textareaPostBody.focus();
      return;
    }

    isPosting = true;
    postErrorNotice.style.display = 'none';
    btnSubmitPost.disabled = true;
    postStatusText.textContent = confirmParams ? '承諾確認を送信中...' : '書き込み中...';
    btnSubmitPostText.textContent = '送信中...';

    const payload = {
      server: currentBoard.server,
      board: currentBoard.board,
      key: currentThread.id,
      name: inputPostName.value.trim(),
      mail: inputPostMail.value.trim(),
      body: textareaPostBody.value,
      extraParams: confirmParams || {},
    };

    try {
      const result = await invokeTauri('post_comment', { payload });
      if (result.status === 'success') {
        postStatusText.textContent = '書き込み完了！';
        setTimeout(async () => {
          closePostModal();
          textareaPostBody.value = '';
          await refreshCurrentThread();
          // 最新レスへスクロール
          postsContainer.scrollTop = postsContainer.scrollHeight;
        }, 400);
      } else if (result.status === 'needConfirm') {
        // 5ch サーバーからの投稿確認・クッキー確認要求
        pendingConfirmParams = Object.assign({}, result.extraParams || {});
        pendingConfirmParams.submit = '上記全てを承諾して書き込む';
        postConfirmNotice.style.display = 'flex';
        postConfirmMessage.textContent = result.message || 'サーバーから投稿の確認が求められました。';
        btnSubmitPostText.textContent = '承諾して書き込む (Ctrl+Enter)';
        btnSubmitPost.disabled = false;
        postStatusText.textContent = '承諾が必要です';
        isPosting = false;
      } else {
        // エラー（規制、NGワード、考え中等）
        postErrorNotice.textContent = result.message || '書き込みエラーが発生しました。';
        postErrorNotice.style.display = 'block';
        btnSubmitPostText.textContent = '書き込む (Ctrl+Enter)';
        btnSubmitPost.disabled = false;
        postStatusText.textContent = 'エラー';
        isPosting = false;
      }
    } catch (err) {
      console.error("post_comment failed:", err);
      postErrorNotice.textContent = `通信エラー: ${err}`;
      postErrorNotice.style.display = 'block';
      btnSubmitPostText.textContent = '書き込む (Ctrl+Enter)';
      btnSubmitPost.disabled = false;
      postStatusText.textContent = 'エラー';
      isPosting = false;
    }
  }

  function setupPostModalUI() {
    if (btnOpenPostModal) btnOpenPostModal.onclick = () => openPostModal();
    if (btnClosePostModal) btnClosePostModal.onclick = closePostModal;
    if (btnCancelPost) btnCancelPost.onclick = closePostModal;

    if (postModalOverlay) {
      postModalOverlay.onclick = (e) => {
        if (e.target === postModalOverlay) closePostModal();
      };
    }

    if (btnToggleSage) {
      btnToggleSage.onclick = () => {
        if (inputPostMail.value.trim().toLowerCase() === 'sage') {
          inputPostMail.value = '';
        } else {
          inputPostMail.value = 'sage';
        }
      };
    }

    if (btnSubmitPost) {
      btnSubmitPost.onclick = () => {
        submitPost(pendingConfirmParams);
      };
    }

    if (textareaPostBody) {
      textareaPostBody.onkeydown = (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
          e.preventDefault();
          submitPost(pendingConfirmParams);
        }
      };
    }
  }

  // --- Refresh Threads & Posts ---
  async function refreshCurrentThread() {
    if (!currentBoard || !currentThread) return;

    try {
      const content = await invokeTauri('get_thread_posts', {
        server: currentBoard.server,
        board: currentBoard.board,
        key: currentThread.id,
      });

      const newPostsCount = content.posts.length - rawPosts.length;
      if (newPostsCount > 0) {
        const isAtBottom = postsContainer.scrollHeight - postsContainer.scrollTop - postsContainer.clientHeight < 120;
        rawPosts = content.posts;
        renderAllPosts();

        if (isAtBottom) {
          postsContainer.scrollTo({ top: postsContainer.scrollHeight, behavior: 'smooth' });
        } else {
          toastText.textContent = `新着 +${newPostsCount} 件`;
          toastNotification.style.display = 'block';
          setTimeout(() => toastNotification.style.display = 'none', 4000);
        }
      }
    } catch (e) {
      console.warn("Refresh thread failed:", e);
    }
  }

  btnRefreshContent.onclick = refreshCurrentThread;

  selectAutoReload.onchange = () => {
    const sec = parseInt(selectAutoReload.value, 10);
    if (autoReloadTimer) clearInterval(autoReloadTimer);
    autoReloadTimer = null;

    if (sec > 0) {
      autoReloadIndicator.style.display = 'inline';
      autoReloadTimer = setInterval(refreshCurrentThread, sec * 1000);
    } else {
      autoReloadIndicator.style.display = 'none';
    }
  };

  toastNotification.onclick = () => {
    postsContainer.scrollTo({ top: postsContainer.scrollHeight, behavior: 'smooth' });
    toastNotification.style.display = 'none';
  };

  document.getElementById('btnRefreshBbs').onclick = () => loadBBSMenu();
  document.getElementById('btnRefreshThreads').onclick = () => {
    if (currentBoard) selectBoard(currentBoard);
  };
  document.getElementById('btnClearHistory').onclick = async () => {
    if (confirm("閲覧履歴をすべて消去しますか？")) {
      await invokeTauri('clear_history');
      history = [];
      renderHistory();
    }
  };

  // --- Sorting & Searching ---
  function setSort(type, activeBtn) {
    currentSort = type;
    ['btnSortIkioi', 'btnSortRes', 'btnSortNew', 'btnSortDefault'].forEach(id => {
      document.getElementById(id)?.classList.remove('active');
    });
    activeBtn.classList.add('active');
    renderThreadTable();
  }

  document.getElementById('btnSortIkioi').onclick = (e) => setSort('ikioi', e.target);
  document.getElementById('btnSortRes').onclick = (e) => setSort('res', e.target);
  document.getElementById('btnSortNew').onclick = (e) => setSort('new', e.target);
  document.getElementById('btnSortDefault').onclick = (e) => setSort('default', e.target);

  threadSearchInput.addEventListener('input', (e) => {
    filterText = e.target.value.trim();
    renderThreadTable();
  });

  // --- Sidebar Sections Accordion ---
  function setupTreeToggles() {
    document.querySelectorAll('.tree-section-header').forEach(header => {
      header.onclick = (e) => {
        if (e.target.closest('.clear-history-btn')) return;
        const targetId = header.getAttribute('data-target');
        const list = document.getElementById(targetId);
        const toggle = header.querySelector('.tree-toggle');
        const isCollapsed = list.classList.toggle('collapsed');
        toggle.textContent = isCollapsed ? '▶' : '▼';
      };
    });
  }

  // --- Focus & Keybindings Management ---
  let activePane = 'sidebar';
  let sidebarSelectedIndex = 0;
  let lastKeyTime = 0;
  let lastKey = '';

  function setFocusedPane(pane) {
    activePane = pane;
    sidebar.classList.toggle('focused', pane === 'sidebar');
    paneThreads.classList.toggle('focused', pane === 'threads');
    paneContent.classList.toggle('focused', pane === 'content');
  }

  // Get visible items in sidebar
  function getVisibleSidebarItems() {
    const items = [];
    document.querySelectorAll('.sidebar .tree-section-header, .sidebar .tree-sub-header, .sidebar .tree-item').forEach(el => {
      // offsetParent is non-null if element is visible and not display:none
      if (el.offsetParent !== null) {
        items.push(el);
      }
    });
    return items;
  }

  function updateSidebarSelection() {
    const items = getVisibleSidebarItems();
    if (items.length === 0) return;
    if (sidebarSelectedIndex < 0) sidebarSelectedIndex = 0;
    if (sidebarSelectedIndex >= items.length) sidebarSelectedIndex = items.length - 1;

    items.forEach((el, idx) => {
      el.classList.toggle('keyboard-selected', idx === sidebarSelectedIndex);
    });

    const currentItem = items[sidebarSelectedIndex];
    if (currentItem) {
      currentItem.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
  }

  function setupKeybindings() {
    // Click on pane to set focus
    sidebar.addEventListener('mousedown', () => setFocusedPane('sidebar'));
    paneThreads.addEventListener('mousedown', () => setFocusedPane('threads'));
    paneContent.addEventListener('mousedown', () => setFocusedPane('content'));

    // Set initial focus
    setFocusedPane('sidebar');

    // External link handler (xdg-open via Rust)
    document.addEventListener('click', (e) => {
      const anchor = e.target.closest('a[href]');
      if (anchor) {
        const href = anchor.getAttribute('href');
        if (href && (href.startsWith('http://') || href.startsWith('https://'))) {
          e.preventDefault();
          invokeTauri('open_external_url', { url: href }).catch(err => {
            console.error("Failed to open external URL:", err);
          });
        }
      }
    });

    window.addEventListener('keydown', (e) => {
      // Don't intercept when typing in input/textarea
      if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
        if (e.key === 'Escape') {
          e.target.blur();
        }
        return;
      }

      if (e.key === 'Escape') {
        if (postModalOverlay && postModalOverlay.classList.contains('open')) {
          closePostModal();
          return;
        }
        idModalOverlay.classList.remove('open');
        ngModalOverlay.classList.remove('open');
        settingsModalOverlay.classList.remove('open');
        return;
      }

      // Tab / Shift+Tab -> Cycle Focus between panes (Supports BackTab / X11)
      const isTabKey = e.key === 'Tab' || e.key === 'BackTab' || e.code === 'Tab' || e.keyCode === 9;
      if (isTabKey) {
        e.preventDefault();
        e.stopPropagation();
        const isBackwards = e.shiftKey || e.key === 'BackTab';
        const panes = ['sidebar', 'threads', 'content'];
        let currentIndex = panes.indexOf(activePane);
        if (currentIndex < 0) currentIndex = 0;

        if (isBackwards) {
          currentIndex = (currentIndex - 1 + panes.length) % panes.length;
        } else {
          currentIndex = (currentIndex + 1) % panes.length;
        }
        setFocusedPane(panes[currentIndex]);
        if (panes[currentIndex] === 'sidebar') {
          updateSidebarSelection();
        }
        return;
      }


      // '/' -> Focus thread search box
      if (e.key === '/') {
        e.preventDefault();
        threadSearchInput.focus();
        threadSearchInput.select();
        return;
      }

      // 'r' -> Refresh current board or thread
      if (e.key === 'r') {
        e.preventDefault();
        if (activePane === 'content' && currentThread) {
          refreshCurrentThread();
        } else if (currentBoard) {
          selectBoard(currentBoard);
        } else {
          loadBBSMenu();
        }
        return;
      }

      // ==========================================
      // 1. Sidebar Focus
      // ==========================================
      if (activePane === 'sidebar') {
        const items = getVisibleSidebarItems();

        // j / ArrowDown -> Move down
        if (e.key === 'j' || e.key === 'ArrowDown') {
          e.preventDefault();
          if (items.length > 0) {
            sidebarSelectedIndex = Math.min(sidebarSelectedIndex + 1, items.length - 1);
            updateSidebarSelection();
          }
          return;
        }

        // k / ArrowUp -> Move up
        if (e.key === 'k' || e.key === 'ArrowUp') {
          e.preventDefault();
          if (items.length > 0) {
            sidebarSelectedIndex = Math.max(sidebarSelectedIndex - 1, 0);
            updateSidebarSelection();
          }
          return;
        }

        // l / ArrowRight -> Expand node or go to threads
        if (e.key === 'l' || e.key === 'ArrowRight') {
          e.preventDefault();
          const cur = items[sidebarSelectedIndex];
          if (!cur) return;

          // Section header -> Expand if collapsed
          if (cur.classList.contains('tree-section-header')) {
            const targetId = cur.getAttribute('data-target');
            const list = document.getElementById(targetId);
            if (list && list.classList.contains('collapsed')) {
              cur.click();
              sidebarSelectedIndex = Math.min(sidebarSelectedIndex + 1, getVisibleSidebarItems().length - 1);
              updateSidebarSelection();
            }
            return;
          }

          // Category header -> Expand if collapsed
          if (cur.classList.contains('tree-sub-header')) {
            const toggle = cur.querySelector('.tree-toggle');
            if (toggle && toggle.textContent === '▶') {
              cur.click();
              sidebarSelectedIndex = Math.min(sidebarSelectedIndex + 1, getVisibleSidebarItems().length - 1);
              updateSidebarSelection();
            }
            return;
          }

          // Board item -> Select board, load threads, and move focus to threads
          if (cur.classList.contains('tree-item')) {
            cur.click();
            setFocusedPane('threads');
            return;
          }
          return;
        }

        // h / ArrowLeft -> Collapse node or go to parent
        if (e.key === 'h' || e.key === 'ArrowLeft') {
          e.preventDefault();
          const cur = items[sidebarSelectedIndex];
          if (!cur) return;

          // Section header -> Collapse if open
          if (cur.classList.contains('tree-section-header')) {
            const targetId = cur.getAttribute('data-target');
            const list = document.getElementById(targetId);
            if (list && !list.classList.contains('collapsed')) {
              cur.click();
              updateSidebarSelection();
            }
            return;
          }

          // Category header -> Collapse if open
          if (cur.classList.contains('tree-sub-header')) {
            const toggle = cur.querySelector('.tree-toggle');
            if (toggle && toggle.textContent === '▼') {
              cur.click();
              updateSidebarSelection();
            }
            return;
          }

          // Board item -> Jump back to parent category header and collapse
          if (cur.classList.contains('tree-item')) {
            const parentSubList = cur.closest('.tree-sub-list');
            if (parentSubList) {
              const catHeader = parentSubList.previousElementSibling;
              if (catHeader && catHeader.classList.contains('tree-sub-header')) {
                const newItems = getVisibleSidebarItems();
                const parentIdx = newItems.indexOf(catHeader);
                if (parentIdx >= 0) {
                  sidebarSelectedIndex = parentIdx;
                  catHeader.click(); // Collapse
                  updateSidebarSelection();
                }
              }
            }
            return;
          }
          return;
        }

        // Enter -> Open/Select and move focus to threads
        if (e.key === 'Enter') {
          e.preventDefault();
          const cur = items[sidebarSelectedIndex];
          if (!cur) return;

          if (cur.classList.contains('tree-item')) {
            cur.click();
            setFocusedPane('threads');
          } else {
            // Header click toggle
            cur.click();
            updateSidebarSelection();
          }
          return;
        }
      }

      // ==========================================
      // 2. Threads List Focus
      // ==========================================
      if (activePane === 'threads') {
        const rows = Array.from(threadTableBody.querySelectorAll('.thread-row'));
        if (rows.length === 0) return;

        let currentIndex = rows.findIndex(r => r.classList.contains('selected'));
        if (currentIndex < 0) currentIndex = 0;

        // j / ArrowDown -> Move selection down (do NOT load thread yet)
        if (e.key === 'j' || e.key === 'ArrowDown') {
          e.preventDefault();
          const nextIndex = Math.min(currentIndex + 1, rows.length - 1);
          rows.forEach((r, idx) => r.classList.toggle('selected', idx === nextIndex));
          rows[nextIndex].scrollIntoView({ behavior: 'smooth', block: 'nearest' });
          return;
        }

        // k / ArrowUp -> Move selection up
        if (e.key === 'k' || e.key === 'ArrowUp') {
          e.preventDefault();
          const prevIndex = Math.max(currentIndex - 1, 0);
          rows.forEach((r, idx) => r.classList.toggle('selected', idx === prevIndex));
          rows[prevIndex].scrollIntoView({ behavior: 'smooth', block: 'nearest' });
          return;
        }

        // h / ArrowLeft -> Return focus to sidebar
        if (e.key === 'h' || e.key === 'ArrowLeft') {
          e.preventDefault();
          setFocusedPane('sidebar');
          updateSidebarSelection();
          return;
        }

        // Enter or l / ArrowRight -> Load selected thread and move focus to content
        if (e.key === 'Enter' || e.key === 'l' || e.key === 'ArrowRight') {
          e.preventDefault();
          const targetRow = rows[currentIndex];
          if (targetRow) {
            const id = targetRow.getAttribute('data-id');
            const t = rawThreads.find(item => item.id === id);
            if (t) {
              selectThread(t);
              setFocusedPane('content');
            }
          }
          return;
        }
      }

      // ==========================================
      // 3. Thread Content Focus
      // ==========================================
      if (activePane === 'content') {
        // 'w' -> Open post compose modal
        if (e.key === 'w' || e.key === 'W') {
          e.preventDefault();
          openPostModal();
          return;
        }

        // 'r' -> Refresh current thread
        if (e.key === 'r' || e.key === 'R') {
          e.preventDefault();
          refreshCurrentThread();
          return;
        }

        // Shift+j (J) -> Half page scroll down
        if (e.key === 'J' || (e.shiftKey && (e.key === 'j' || e.key === 'J'))) {
          e.preventDefault();
          postsContainer.scrollBy({ top: postsContainer.clientHeight * 0.5, behavior: 'smooth' });
          return;
        }

        // Shift+k (K) -> Half page scroll up
        if (e.key === 'K' || (e.shiftKey && (e.key === 'k' || e.key === 'K'))) {
          e.preventDefault();
          postsContainer.scrollBy({ top: -postsContainer.clientHeight * 0.5, behavior: 'smooth' });
          return;
        }

        // j / ArrowDown -> Scroll down smoothly (small step)
        if ((e.key === 'j' && !e.shiftKey) || e.key === 'ArrowDown') {
          e.preventDefault();
          const step = appSettings.scrollAmount || 120;
          postsContainer.scrollBy({ top: step, behavior: 'smooth' });
          return;
        }

        // k / ArrowUp -> Scroll up smoothly (small step)
        if ((e.key === 'k' && !e.shiftKey) || e.key === 'ArrowUp') {
          e.preventDefault();
          const step = appSettings.scrollAmount || 120;
          postsContainer.scrollBy({ top: -step, behavior: 'smooth' });
          return;
        }

        // h / ArrowLeft -> Return focus to threads list
        if (e.key === 'h' || e.key === 'ArrowLeft') {
          e.preventDefault();
          setFocusedPane('threads');
          return;
        }

        // Half page down: Ctrl+d / d / PageDown
        if (e.key === 'd' || (e.ctrlKey && e.key === 'd') || e.key === 'PageDown') {
          e.preventDefault();
          postsContainer.scrollBy({ top: postsContainer.clientHeight * 0.5, behavior: 'smooth' });
          return;
        }

        // Half page up: Ctrl+u / u / PageUp
        if (e.key === 'u' || (e.ctrlKey && e.key === 'u') || e.key === 'PageUp') {
          e.preventDefault();
          postsContainer.scrollBy({ top: -postsContainer.clientHeight * 0.5, behavior: 'smooth' });
          return;
        }


        // 'G' -> Scroll to bottom
        if (e.key === 'G') {
          e.preventDefault();
          postsContainer.scrollTo({ top: postsContainer.scrollHeight, behavior: 'smooth' });
          return;
        }

        // 'gg' -> Scroll to top
        const now = Date.now();
        if (e.key === 'g') {
          if (lastKey === 'g' && now - lastKeyTime < 400) {
            e.preventDefault();
            postsContainer.scrollTo({ top: 0, behavior: 'smooth' });
            lastKey = '';
            return;
          }
          lastKey = 'g';
          lastKeyTime = now;
          return;
        }
      }

      lastKey = e.key;
      lastKeyTime = Date.now();
    });
  }


  // --- Resizable Splitters ---
  function setupSplitters() {
    // Vertical splitter (Sidebar <-> Main)
    let isDraggingV = false;
    splitterSidebar.addEventListener('mousedown', () => {
      isDraggingV = true;
      splitterSidebar.classList.add('dragging');
      document.body.style.cursor = 'col-resize';
    });

    // Horizontal splitter (Threads <-> Content)
    let isDraggingH = false;
    splitterMain.addEventListener('mousedown', () => {
      isDraggingH = true;
      splitterMain.classList.add('dragging');
      document.body.style.cursor = 'row-resize';
    });

    window.addEventListener('mousemove', (e) => {
      if (isDraggingV) {
        const newWidth = Math.max(180, Math.min(e.clientX, 500));
        sidebar.style.width = newWidth + 'px';
      } else if (isDraggingH) {
        const mainRect = paneThreads.parentElement.getBoundingClientRect();
        const offsetY = e.clientY - mainRect.top;
        const newHeight = Math.max(120, Math.min(offsetY, mainRect.height - 150));
        paneThreads.style.height = newHeight + 'px';
      }
    });

    window.addEventListener('mouseup', () => {
      if (isDraggingV) {
        isDraggingV = false;
        splitterSidebar.classList.remove('dragging');
        document.body.style.cursor = '';
      }
      if (isDraggingH) {
        isDraggingH = false;
        splitterMain.classList.remove('dragging');
        document.body.style.cursor = '';
      }
    });
  }

  // Run
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();

