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
 *   - freshness      (every page — re-checks each New/Updated badge's
 *                      48h window against the visitor's own clock, and
 *                      formats the fresh rail's timestamps as "3 hours ago")
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
        var local = new Intl.DateTimeFormat(undefined, {
          dateStyle: 'medium', timeStyle: 'short'
        }).format(when);
        var utc = new Intl.DateTimeFormat(undefined, {
          hour: '2-digit', minute: '2-digit', hour12: false, timeZone: 'UTC'
        }).format(when);
        el.textContent = local + ' your time · ' + utc + ' UTC';
      } catch (error) {
        // Unsupported options or timeZone in this browser: leave the
        // server-rendered plain-UTC fallback text in place.
      }
    });
  }

  // -------------------------------------------------------------------
  // Freshness — the browser half of the New/Updated window.
  //
  // generate.py already dropped anything older than FRESH_WINDOW_HOURS at
  // build time, but this site is static: a page built an hour before the
  // window closes keeps claiming "New" for as long as a visitor's tab, or a
  // CDN, holds onto it. Every element carrying data-fresh-at is re-checked
  // here against the visitor's own clock, which is what makes "48 hours"
  // true rather than "48 hours as of whenever this page was built".
  //
  // Server-side stays authoritative for no-JS visitors: they see the build's
  // answer, which is correct at build time and only ever errs toward showing
  // a badge slightly too long — never toward hiding a real one.
  // -------------------------------------------------------------------

  var FRESH_WINDOW_MS = 48 * 60 * 60 * 1000;

  function initFreshness() {
    var stamped = toArray(document.querySelectorAll('[data-fresh-at]'));
    if (!stamped.length) return;
    var now = Date.now();

    stamped.forEach(function (el) {
      var when = new Date(el.getAttribute('data-fresh-at'));
      // An unparseable timestamp leaves the element exactly as the server
      // rendered it — a badge that lingers is a far smaller failure than one
      // that vanishes because a date string had a shape this didn't expect.
      if (isNaN(when.getTime())) return;
      if (now - when.getTime() > FRESH_WINDOW_MS) {
        el.parentNode.removeChild(el);
      }
    });

    // The rail's heading and gilded rule are only worth their space if at
    // least one tile survived; an empty "Fresh from the oven" is a section
    // announcing it has nothing to announce. querySelectorAll rather than
    // querySelector: the landing page only ever renders one of these, but
    // the singular form silently checks the *first* section and leaves any
    // other one standing empty, which is a trap for whoever adds the second.
    toArray(document.querySelectorAll('[data-fresh-section]')).forEach(function (section) {
      if (!section.querySelectorAll('.fresh__tile').length) {
        section.parentNode.removeChild(section);
      }
    });
  }

  // Absolute dates are the wrong unit for something that expires in 48
  // hours: "3 hours ago" answers the question the badge raises, "2026-08-14"
  // makes the reader do the subtraction. Falls back silently to the
  // server-rendered date where Intl.RelativeTimeFormat is missing.
  function initRelativeTimes() {
    var els = toArray(document.querySelectorAll('[data-format-relative]'));
    if (!els.length) return;
    if (!window.Intl || !window.Intl.RelativeTimeFormat) return;

    var formatter;
    try {
      formatter = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });
    } catch (error) {
      return;
    }

    els.forEach(function (el) {
      var iso = el.getAttribute('datetime');
      if (!iso) return;
      var when = new Date(iso);
      if (isNaN(when.getTime())) return;

      // Elapsed time is measured as a positive magnitude and floored, then
      // negated for the formatter. Rounding a signed value would round *away*
      // from zero for negatives — 47 hours becomes "2 days ago" when only one
      // day has actually passed — and flooring a negative does the same. This
      // is also why the clamp is here: clock skew, or a commit stamped a few
      // seconds ahead of a visitor whose clock runs slow, would otherwise
      // read "in 1 minute" on a panel about things that already happened.
      var elapsedMinutes = Math.max(0, Math.floor((Date.now() - when.getTime()) / 60000));

      try {
        if (elapsedMinutes < 60) {
          el.textContent = formatter.format(-elapsedMinutes, 'minute');
        } else if (elapsedMinutes < 60 * 24) {
          el.textContent = formatter.format(-Math.floor(elapsedMinutes / 60), 'hour');
        } else {
          el.textContent = formatter.format(-Math.floor(elapsedMinutes / (60 * 24)), 'day');
        }
      } catch (error) {
        // Leave the server-rendered absolute date in place.
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
    var targetTheme = theme === 'flour' ? 'dark' : 'light';
    toggle.setAttribute('aria-label', 'Switch to ' + targetTheme + ' theme');
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
          badge.textContent = shown + ' ' + (shown === 1 ? 'skill' : 'skills');
        }
      });

      clearButton.hidden = query === '';

      var showEmptyState = query !== '' && visibleCount === 0;
      if (emptyState) {
        emptyState.hidden = !showEmptyState;
        if (showEmptyState && emptyQuery) {
          emptyQuery.textContent = '“' + input.value.trim() + '”';
        }
      }

      if (query === '') {
        status.textContent = 'Showing all ' + totalCount + ' skills.';
      } else if (visibleCount === 0) {
        status.textContent = 'No skills match.';
      } else {
        status.textContent = visibleCount + ' of ' + totalCount + ' skills match.';
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
        label.textContent = 'Copied!';
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

  // Every page paints the wordmark's wheat mark from the shared sprite, so
  // the sprite's URL — base path and all — is already on the page and does
  // not need to be threaded through as its own data attribute. Returns ''
  // if no icon is present, and callers drop their icon rather than emit a
  // <use> pointing at nothing.
  function spriteHref(symbolId) {
    var existing = document.querySelector('svg.icon use');
    var href = existing ? existing.getAttribute('href') || '' : '';
    var base = href.split('#')[0];
    return base ? base + '#' + symbolId : '';
  }

  function iconMarkup(symbolId) {
    var href = spriteHref(symbolId);
    if (!href) return '';
    return '<svg class="icon" aria-hidden="true"><use href="' + href + '"></use></svg>';
  }

  function showUpdateModal() {
    if (document.querySelector('.update-modal-overlay')) return;

    // The seal is the loaf mark and nothing else, so with no sprite to draw
    // from it is dropped whole rather than left as an empty gold ring.
    var sealIcon = iconMarkup('icon-crust');
    var seal = sealIcon
      ? '<span class="update-modal__seal" aria-hidden="true">' + sealIcon + '</span>'
      : '';

    var overlay = document.createElement('div');
    overlay.className = 'update-modal-overlay';
    overlay.innerHTML =
      '<div class="update-modal" role="alertdialog" aria-modal="true" ' +
      'aria-labelledby="update-modal-title" aria-describedby="update-modal-body">' +
      seal +
      '<p class="update-modal__title" id="update-modal-title">New version available</p>' +
      '<p class="update-modal__body" id="update-modal-body">This page has been updated. Reload to see the latest.</p>' +
      '<button class="update-modal__reload" type="button">' +
      iconMarkup('icon-rotate') +
      '<span>Reload</span>' +
      '</button>' +
      '</div>';

    // Genuinely modal, not just visually on top: everything already in
    // <body> (skip link, header, main, footer) becomes untabbable and
    // unclickable. The overlay itself is appended after, so it's never
    // included in this pass.
    toArray(document.body.children).forEach(function (el) {
      el.setAttribute('inert', '');
    });
    document.body.appendChild(overlay);

    var reloadButton = overlay.querySelector('.update-modal__reload');
    reloadButton.addEventListener('click', function () {
      // A reload is not instant on a cold or slow connection, and the button
      // is the only thing on screen that can acknowledge the click. Guard on
      // the same attribute that drives the styling, so a second click during
      // a slow reload can't stack a second navigation.
      if (reloadButton.getAttribute('aria-busy') === 'true') return;
      reloadButton.setAttribute('aria-busy', 'true');

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
  // Freshness runs before search: it can remove whole rail tiles, and
  // initSearch() counts cards to build its "showing all N" status.
  initFreshness();
  initRelativeTimes();
  initSearch();
  initInstallCopy();
  initTabs();
  initUpdatedTime();
  initScrollRestore();
  initVersionCheck();
})();
