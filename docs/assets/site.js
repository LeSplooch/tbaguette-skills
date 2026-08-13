/*
 * La Boulangerie TBaguette — site.js
 * Vanilla JS, no dependencies. Three independent features, each a no-op if
 * its markup isn't on the current page:
 *   - theme toggle   (every page)
 *   - search/filter  (landing page only)
 *   - tabs           (formidable's skill page only)
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
  // Tabs (formidable's stacks / commands groups)
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

  initThemeToggle();
  initSearch();
  initTabs();
})();
