/*
 * La Boulangerie TBaguette — site.js
 * Vanilla JS, no dependencies. Seven independent features, each a no-op if
 * its markup isn't on the current page:
 *   - theme toggle   (every page)
 *   - search/filter  (landing page only)
 *   - tabs           (formidable's skill page's Stacks/Commands; the
 *                      landing page's install-command platform picker)
 *   - copy install command (landing page only; one button per platform tab)
 *   - header "Updated" time (every page — formats the baked-in UTC instant
 *                             as the visitor's local time)
 *   - site update check (every page — polls docs/version.txt every 10-12s;
 *                         on a mismatch, shows a reload-only modal)
 *   - post-reload scroll restore (every page — companion to the update
 *                                  check; restores scroll position after
 *                                  the reload it triggered)
 * Loaded with `defer`, so the DOM is fully parsed before any of this runs —
 * no DOMContentLoaded wrapper needed.
 */
(function () {
  'use strict';

  var THEME_STORAGE_KEY = 'tbaguette-theme';

  function prefersReducedMotion() {
    return !!(window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches);
  }

  function toArray(nodeList) {
    return Array.prototype.slice.call(nodeList);
  }

  function formatTemplate(template, values) {
    return template.replace(/\{(\w+)\}/g, function (match, key) {
      return Object.prototype.hasOwnProperty.call(values, key) ? String(values[key]) : match;
    });
  }

  // Best-effort OS sniff, used only to pick a sane default platform tab in
  // the install frame — never to gate functionality. navigator.platform is
  // deprecated but still broadly supported; userAgentData.platform is its
  // modern, more honest replacement where a browser has it. Worst case on a
  // browser with neither (or a misleading one), the visitor sees the POSIX
  // command first and clicks the other tab themselves — never a dead end.
  function isWindowsPlatform() {
    var uaData = window.navigator.userAgentData;
    if (uaData && typeof uaData.platform === 'string') {
      return /win/i.test(uaData.platform);
    }
    var legacy = window.navigator.platform || window.navigator.userAgent || '';
    return /win/i.test(legacy);
  }

  // -------------------------------------------------------------------
  // Header "Updated" time — the server bakes in one UTC instant; this
  // renders it in both the visitor's own local time and UTC, since the
  // visitor's timezone is only knowable in the browser, never at build
  // time. Left as the server-rendered plain-UTC fallback if Intl isn't
  // available or the timestamp fails to parse — never blanked out.
  // -------------------------------------------------------------------

  // The page's own language, for Intl — NOT the browser's. Passing
  // undefined (the old behaviour) formats against whatever the visitor's
  // browser is set to, so a reader on /ja/ got an English month name in a
  // Japanese header. Every page sets <html lang> and all 16 values are
  // valid BCP-47 tags, but a malformed one would make Intl throw
  // RangeError and cost us the whole formatted timestamp, so an
  // unparseable tag degrades to the browser default rather than to
  // nothing. supportedLocalesOf is the cheapest way to ask "would Intl
  // reject this tag?" without building a formatter: it throws on a
  // structurally invalid tag, and merely returns [] for a well-formed tag
  // the browser has no data for — which is fine, Intl falls back on its
  // own for that case.
  function pageLocale() {
    var lang = document.documentElement.getAttribute('lang');
    if (!lang) return undefined;
    try {
      Intl.DateTimeFormat.supportedLocalesOf([lang]);
      return lang;
    } catch (error) {
      return undefined;
    }
  }

  function initUpdatedTime() {
    var els = toArray(document.querySelectorAll('[data-format-updated]'));
    if (!els.length) return;
    if (!window.Intl || !window.Intl.DateTimeFormat) return;

    els.forEach(function (el) {
      var iso = el.getAttribute('datetime');
      if (!iso) return;
      var when = new Date(iso);
      if (isNaN(when.getTime())) return;

      try {
        var forLocale = pageLocale();
        var local = new Intl.DateTimeFormat(forLocale, {
          dateStyle: 'medium', timeStyle: 'short'
        }).format(when);
        var utc = new Intl.DateTimeFormat(forLocale, {
          hour: '2-digit', minute: '2-digit', hour12: false, timeZone: 'UTC'
        }).format(when);
        // Glue text comes from the page's catalog; the English default
        // matches ENGLISH_STRINGS.header_updated_value_template so a page
        // rendered before that field existed still reads correctly.
        var template = el.getAttribute('data-i18n-updated-template')
          || '{local} your time · {utc} UTC';
        el.textContent = formatTemplate(template, { local: local, utc: utc });
      } catch (error) {
        // Unsupported options or timeZone in this browser: leave the
        // server-rendered plain-UTC fallback text in place.
      }
    });
  }

  // -------------------------------------------------------------------
  // Theme toggle
  // -------------------------------------------------------------------

  function currentTheme() {
    return document.documentElement.getAttribute('data-theme') === 'flour' ? 'flour' : 'crust';
  }

  function describeToggleTarget(theme) {
    var toggle = document.querySelector('[data-theme-toggle]');
    if (!toggle) return;
    var label = theme === 'flour'
      ? toggle.getAttribute('data-i18n-theme-dark')
      : toggle.getAttribute('data-i18n-theme-light');
    toggle.setAttribute('aria-label', label);
  }

  function setTheme(theme) {
    if (theme === 'flour') {
      document.documentElement.setAttribute('data-theme', 'flour');
    } else {
      document.documentElement.removeAttribute('data-theme');
    }
    try {
      localStorage.setItem(THEME_STORAGE_KEY, theme);
    } catch (error) {
      // Private browsing / storage disabled: theme still applies for this
      // load, it just won't persist. Not worth surfacing to the user.
    }
    describeToggleTarget(theme);
  }

  function initThemeToggle() {
    var toggle = document.querySelector('[data-theme-toggle]');
    if (!toggle) return;
    describeToggleTarget(currentTheme());
    toggle.addEventListener('click', function () {
      setTheme(currentTheme() === 'flour' ? 'crust' : 'flour');
    });
  }

  // -------------------------------------------------------------------
  // Search / filter (landing page)
  // -------------------------------------------------------------------

  function initSearch() {
    var root = document.querySelector('[data-search-root]');
    if (!root) return;

    var input = root.querySelector('[data-search-input]');
    var clearButton = root.querySelector('[data-search-clear]');
    var status = root.querySelector('[data-search-status]');
    var emptyState = document.querySelector('[data-search-empty]');
    var emptyQuery = emptyState ? emptyState.querySelector('[data-search-empty-query]') : null;
    var resetButton = emptyState ? emptyState.querySelector('[data-search-reset]') : null;
    var cards = toArray(document.querySelectorAll('[data-search-card]'));
    var sections = toArray(document.querySelectorAll('[data-category-section]'));
    var totalCount = cards.length;

    function applyFilter() {
      var query = input.value.trim().toLowerCase();
      var visibleCount = 0;

      cards.forEach(function (card) {
        var haystack = card.getAttribute('data-search-terms') || '';
        var matches = query === '' || haystack.indexOf(query) !== -1;
        card.hidden = !matches;
        if (matches) visibleCount += 1;
      });

      sections.forEach(function (section) {
        var stillVisible = section.querySelectorAll('[data-search-card]:not([hidden])').length;
        section.hidden = query !== '' && stillVisible === 0;

        var badge = section.querySelector('[data-category-count]');
        if (badge) {
          var total = parseInt(badge.getAttribute('data-category-count'), 10);
          var shown = query === '' ? total : stillVisible;
          var noun = shown === 1
            ? badge.getAttribute('data-i18n-singular')
            : badge.getAttribute('data-i18n-plural');
          badge.textContent = shown + ' ' + noun;
        }
      });

      clearButton.hidden = query === '';

      var showEmptyState = query !== '' && visibleCount === 0;
      if (emptyState) {
        emptyState.hidden = !showEmptyState;
        if (showEmptyState && emptyQuery) {
          var quoteOpen = document.body.getAttribute('data-i18n-quote-open') || '“';
          var quoteClose = document.body.getAttribute('data-i18n-quote-close') || '”';
          emptyQuery.textContent = quoteOpen + input.value.trim() + quoteClose;
        }
      }

      if (query === '') {
        status.textContent = formatTemplate(
          status.getAttribute('data-i18n-showing-all-template'), { count: totalCount }
        );
      } else if (visibleCount === 0) {
        status.textContent = document.body.getAttribute('data-i18n-no-match');
      } else {
        status.textContent = formatTemplate(
          status.getAttribute('data-i18n-partial-template'), { shown: visibleCount, total: totalCount }
        );
      }
    }

    function resetSearch() {
      input.value = '';
      applyFilter();
      input.focus();
    }

    input.addEventListener('input', applyFilter);
    clearButton.addEventListener('click', resetSearch);
    if (resetButton) resetButton.addEventListener('click', resetSearch);

    applyFilter();
  }

  // -------------------------------------------------------------------
  // Copy install command
  // -------------------------------------------------------------------

  function fallbackCopy(text) {
    var scratch = document.createElement('textarea');
    scratch.value = text;
    scratch.setAttribute('readonly', '');
    scratch.style.position = 'fixed';
    scratch.style.opacity = '0';
    document.body.appendChild(scratch);
    scratch.select();
    try { document.execCommand('copy'); } catch (err) { /* nothing left to try */ }
    document.body.removeChild(scratch);
  }

  // One page can carry more than one copy button now — the install frame
  // has a separate command (and a separate button) per platform tab. Each
  // button gets its own independent target/label/timer via its own
  // [data-copy-target] id, so copying one platform's command never
  // clobbers another's "Copied!" state.
  function initInstallCopy() {
    var buttons = toArray(document.querySelectorAll('[data-copy-target]'));
    if (!buttons.length) return;

    buttons.forEach(function (button) {
      var target = document.getElementById(button.getAttribute('data-copy-target'));
      var label = button.querySelector('[data-copy-label]');
      if (!target || !label) return;

      var defaultLabel = label.textContent;
      var resetTimer = null;

      function showCopied() {
        button.classList.add('is-copied');
        label.textContent = document.body.getAttribute('data-i18n-copied');
        clearTimeout(resetTimer);
        resetTimer = setTimeout(function () {
          button.classList.remove('is-copied');
          label.textContent = defaultLabel;
        }, 1600);
      }

      button.addEventListener('click', function () {
        var text = target.textContent;
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(text).then(showCopied, function () {
            fallbackCopy(text);
            showCopied();
          });
        } else {
          fallbackCopy(text);
          showCopied();
        }
      });
    });
  }

  // -------------------------------------------------------------------
  // Tabs (formidable's stacks / commands groups; the install frame's
  // platform picker)
  // -------------------------------------------------------------------

  function scrollToPanel(panel) {
    if (!panel) return;
    panel.scrollIntoView({ behavior: prefersReducedMotion() ? 'auto' : 'smooth', block: 'start' });
  }

  function initTabs() {
    var groups = toArray(document.querySelectorAll('[data-tabs]'));
    if (!groups.length) return;

    var everyTab = [];

    groups.forEach(function (group) {
      var tabs = toArray(group.querySelectorAll('[role="tab"]'));
      var panels = toArray(group.querySelectorAll('[role="tabpanel"]'));

      function activate(tab, options) {
        options = options || {};
        tabs.forEach(function (candidate) {
          var selected = candidate === tab;
          candidate.setAttribute('aria-selected', selected ? 'true' : 'false');
          candidate.tabIndex = selected ? 0 : -1;
        });
        panels.forEach(function (panel) {
          panel.hidden = panel.id !== tab.getAttribute('aria-controls');
        });
        if (options.focus) tab.focus();
        if (options.scroll) scrollToPanel(document.getElementById(tab.getAttribute('aria-controls')));
      }

      // Opt-in only (the install frame sets this; formidable's Stacks/
      // Commands tabs don't and are unaffected): pre-select whichever tab
      // is tagged data-platform="windows" for a visitor who looks like
      // they're on Windows. Still just a default — every tab stays
      // reachable by click either way, for WSL/Git Bash users on Windows
      // or anyone detection got wrong.
      if (group.getAttribute('data-autoselect-platform') === 'true' && isWindowsPlatform()) {
        var windowsTab = tabs.filter(function (candidate) {
          return candidate.getAttribute('data-platform') === 'windows';
        })[0];
        if (windowsTab) activate(windowsTab);
      }

      tabs.forEach(function (tab, index) {
        tab.addEventListener('click', function () {
          activate(tab);
        });

        tab.addEventListener('keydown', function (event) {
          var targetIndex = null;
          switch (event.key) {
            case 'ArrowRight':
            case 'ArrowDown':
              targetIndex = (index + 1) % tabs.length;
              break;
            case 'ArrowLeft':
            case 'ArrowUp':
              targetIndex = (index - 1 + tabs.length) % tabs.length;
              break;
            case 'Home':
              targetIndex = 0;
              break;
            case 'End':
              targetIndex = tabs.length - 1;
              break;
            default:
              return;
          }
          event.preventDefault();
          activate(tabs[targetIndex], { focus: true });
        });

        everyTab.push({ tab: tab, activate: activate });
      });
    });

    // A link inside body_html (or an external link) may point at
    // "#stack-web"-style panel ids. Those panels start hidden unless their
    // tab is first in the group, so a plain anchor jump would land on
    // invisible content — activate the owning tab before/while scrolling.
    function focusPanelFromHash() {
      var hash = window.location.hash.slice(1);
      if (!hash) return;
      var entry = everyTab.filter(function (candidate) {
        return candidate.tab.getAttribute('aria-controls') === hash;
      })[0];
      if (entry) entry.activate(entry.tab, { scroll: true });
    }

    window.addEventListener('hashchange', focusPanelFromHash);
    focusPanelFromHash();
  }

  // -------------------------------------------------------------------
  // Site update check — polls a tiny generated file for a newer build,
  // then prompts a reload. Reload-only by design: no dismiss/snooze/Esc/
  // backdrop-click, since the point is to not leave a visitor on a stale
  // page. Scroll position is saved before reloading and restored after,
  // via initScrollRestore() below.
  // -------------------------------------------------------------------

  var SCROLL_RESTORE_KEY = 'tbaguette-reload-scroll-y';
  var UPDATE_POLL_MIN_MS = 10000;
  var UPDATE_POLL_MAX_MS = 12000;

  function showUpdateModal() {
    if (document.querySelector('.update-modal-overlay')) return;

    var title = document.body.getAttribute('data-i18n-modal-title');
    var body = document.body.getAttribute('data-i18n-modal-body');
    var reloadLabel = document.body.getAttribute('data-i18n-modal-reload');

    var overlay = document.createElement('div');
    overlay.className = 'update-modal-overlay';

    var modal = document.createElement('div');
    modal.className = 'update-modal';
    modal.setAttribute('role', 'alertdialog');
    modal.setAttribute('aria-modal', 'true');
    modal.setAttribute('aria-labelledby', 'update-modal-title');
    modal.setAttribute('aria-describedby', 'update-modal-body');

    var titleEl = document.createElement('p');
    titleEl.className = 'update-modal__title';
    titleEl.id = 'update-modal-title';
    titleEl.textContent = title;

    var bodyEl = document.createElement('p');
    bodyEl.className = 'update-modal__body';
    bodyEl.id = 'update-modal-body';
    bodyEl.textContent = body;

    var reloadButton = document.createElement('button');
    reloadButton.className = 'update-modal__reload';
    reloadButton.type = 'button';
    reloadButton.textContent = reloadLabel;

    modal.appendChild(titleEl);
    modal.appendChild(bodyEl);
    modal.appendChild(reloadButton);
    overlay.appendChild(modal);

    // Genuinely modal, not just visually on top: everything already in
    // <body> (skip link, header, main, footer) becomes untabbable and
    // unclickable. The overlay itself is appended after, so it's never
    // included in this pass.
    toArray(document.body.children).forEach(function (el) {
      el.setAttribute('inert', '');
    });
    document.body.appendChild(overlay);

    reloadButton.addEventListener('click', function () {
      try {
        sessionStorage.setItem(SCROLL_RESTORE_KEY, String(window.scrollY));
      } catch (error) {
        // Private browsing / storage disabled: reload still happens, it
        // just won't restore scroll position.
      }
      window.location.reload();
    });
    reloadButton.focus();
  }

  function initVersionCheck() {
    var timeEl = document.querySelector('[data-format-updated]');
    if (!timeEl) return;
    var versionUrl = timeEl.getAttribute('data-version-url');
    var initialVersion = timeEl.getAttribute('datetime');
    if (!versionUrl || !initialVersion || !window.fetch) return;

    function scheduleNext() {
      var delay = UPDATE_POLL_MIN_MS + Math.random() * (UPDATE_POLL_MAX_MS - UPDATE_POLL_MIN_MS);
      setTimeout(poll, delay);
    }

    function poll() {
      // Skip the network call while backgrounded; the timer chain keeps
      // running, so a refocused tab notices within one interval.
      if (document.hidden) {
        scheduleNext();
        return;
      }
      // cache: 'no-store' bypasses the browser's own HTTP cache; the ?t=
      // query param is belt-and-suspenders against any intermediate cache
      // that doesn't honor that — GitHub Pages' CDN headers aren't
      // something this project controls.
      var bustedUrl = versionUrl + (versionUrl.indexOf('?') === -1 ? '?' : '&') + 't=' + Date.now();
      fetch(bustedUrl, { cache: 'no-store' }).then(function (response) {
        if (!response.ok) throw new Error('version check failed: ' + response.status);
        return response.text();
      }).then(function (text) {
        var latest = text.trim();
        if (latest && latest !== initialVersion) {
          showUpdateModal();
        } else {
          scheduleNext();
        }
      }).catch(function () {
        // Offline / transient failure: try again next cycle, no error
        // surfaced to the visitor.
        scheduleNext();
      });
    }

    scheduleNext();
  }

  function initScrollRestore() {
    var saved;
    try {
      saved = sessionStorage.getItem(SCROLL_RESTORE_KEY);
      if (saved !== null) sessionStorage.removeItem(SCROLL_RESTORE_KEY);
    } catch (error) {
      return;
    }
    if (saved === null) return;
    var y = parseInt(saved, 10);
    if (!isNaN(y)) window.scrollTo(0, y);
  }

  initThemeToggle();
  initSearch();
  initInstallCopy();
  initTabs();
  initUpdatedTime();
  initScrollRestore();
  initVersionCheck();
})();
