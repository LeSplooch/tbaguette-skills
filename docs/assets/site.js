/*
 * La Boulangerie TBaguette — site.js
 * Vanilla JS, no dependencies. Ten independent features, each a no-op if
 * its markup isn't on the current page:
 *   - theme toggle   (every page)
 *   - search/filter  (landing page only)
 *   - tabs           (formidable's skill page's Stacks/Commands; the
 *                      landing page's install-command platform picker)
 *   - copy install command (landing page only; one button per platform tab)
 *   - header "Updated" time (every page — formats the baked-in UTC instant
 *                             as the visitor's local time)
 *   - freshness      (every page — re-checks each New/Updated badge's
 *                      48h window against the visitor's own clock)
 *   - fresh coverflow (landing page only — steps which rail tile is centred
 *                       on a timer; pauses on hover/focus; click-and-drag
 *                       spins it manually, clicking a side tile recentres
 *                       it instead of navigating)
 *   - language switcher dismissal (every page — Escape / outside click on
 *                                   the native <details>; opening and
 *                                   closing it needs no JS)
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

  // -------------------------------------------------------------------
  // Fresh coverflow — the browser half of the "Fresh from the oven" rail.
  //
  // templates.py renders every tile with a --cf-offset custom property
  // (its signed distance from the rail's centre) and styles.css turns that
  // into position, tilt, scale, and opacity — but only one offset layout
  // is ever "current", and which one that is changes on a clock, the one
  // thing CSS alone can't keep on a static site. This is that clock: every
  // FRESH_COVERFLOW_STEP_MS it advances which tile is centred and rewrites
  // every tile's --cf-offset to match. freshSignedOffset() restates
  // templates.py's _fresh_signed_offset() rather than sharing it — there is
  // no build step joining this file to that one — so the two are kept
  // deliberately parallel; a change to one's wraparound rule needs the
  // same change made here.
  //
  // A no-JS visitor never runs this function and simply keeps the offsets
  // the server rendered — a real, if motionless, coverflow, not a fallback.
  //
  // The prev/next buttons (markup rendered hidden by templates.py; see
  // below for where this un-hides them) are the manual override: same
  // activeIndex, same layout(), just a step the visitor chose instead of
  // the clock.
  // -------------------------------------------------------------------

  var FRESH_COVERFLOW_STEP_MS = 3000;

  function freshSignedOffset(index, activeIndex, count) {
    if (count <= 1) return 0;
    var diff = ((index - activeIndex) % count + count) % count;
    return diff > count / 2 ? diff - count : diff;
  }

  function initFreshCoverflow() {
    var rail = document.querySelector('[data-fresh-coverflow]');
    if (!rail) return;
    var tiles = toArray(rail.querySelectorAll('.fresh__tile'));
    if (tiles.length < 2) return;

    // The same three conditions styles.css gates the coverflow rule
    // behind: this function only ever drives a property that rule reads,
    // so running it where that rule never applies would just rewrite
    // --cf-offset every few seconds for nothing to consume.
    if (prefersReducedMotion()) return;
    if (window.matchMedia && window.matchMedia('(max-width: 639px)').matches) return;
    if (window.matchMedia && window.matchMedia('(forced-colors: active)').matches) return;

    var count = tiles.length;
    var activeIndex = 0;

    // Two independent reasons the clock might not be driving right now,
    // not one: "hovering" is temporary and self-undoes (mouseleave/
    // focusout), for the WCAG 2.2.2 pause-while-the-visitor-is-right-there
    // case. "manualOverride" is permanent once set — prev/next, a drag, or
    // picking a side tile all mean the visitor has taken the wheel, and
    // the clock resuming a few seconds after they let go of the mouse
    // would just undo their choice. A single `paused` flag used to stand
    // in for both, which meant mouseleave after a prev/next click quietly
    // resumed autoplay the moment the pointer left the rail — the opposite
    // of what its own comment claimed.
    var hovering = false;
    var manualOverride = false;
    function isPaused() { return hovering || manualOverride; }

    // Setting only the custom property here would be simpler, and
    // styles.css's --cf-transform / opacity calc()s would pick it up. This
    // writes plain transform/opacity values too, deliberately redundant
    // with that calc(): transitioning a property whose value derives from
    // an *unregistered* custom property is a real, documented animation
    // edge case in the platform (it's the motivating case for @property),
    // and an ordinary transition between two ordinary values is the one
    // form guaranteed to animate correctly everywhere, no browser-specific
    // custom-property-interpolation behaviour to depend on. --cf-offset is
    // still set alongside it, since it (and styles.css's calc() rule) is
    // what a no-JS visitor's static, correct-but-motionless coverflow
    // relies on. The clamped magnitudes mirror styles.css's clamp()s
    // exactly — see that rule for why they're this steep.
    //
    // index is a plain parameter, not always activeIndex itself: a drag
    // in progress renders a live, fractional index every pointermove
    // without ever writing that half-settled value into activeIndex,
    // which only ever holds a real resting tile position.
    function layout(index) {
      tiles.forEach(function (tile, i) {
        var offset = freshSignedOffset(i, index, count);
        tile.style.setProperty('--cf-offset', offset);
        var abs = Math.abs(offset);
        var x = Math.max(-280, Math.min(280, offset * 140));
        var rotate = Math.max(-38, Math.min(38, offset * -34));
        var scale = Math.max(0.55, 1 - abs * 0.19);
        tile.style.transform =
          'translateX(' + x + 'px) ' +
          'rotateY(' + rotate + 'deg) ' +
          'scale(' + scale + ')';
        tile.style.opacity = Math.max(0, 1 - abs * 0.32);
      });
    }

    function step() {
      if (isPaused()) return;
      activeIndex = (activeIndex + 1) % count;
      layout(activeIndex);
    }

    // WCAG 2.2.2: anything that moves on its own for more than five seconds
    // needs a way to stop it. Hovering or tabbing into the rail is that way
    // — the interval keeps firing but simply stops rewriting offsets, so
    // resuming afterwards continues from the same tile rather than jumping.
    rail.addEventListener('mouseenter', function () { hovering = true; });
    rail.addEventListener('mouseleave', function () { hovering = false; });
    rail.addEventListener('focusin', function (event) {
      hovering = true;
      // Recentre on whichever tile actually received focus, rather than
      // leaving it frozen wherever the step timer last put it — a keyboard
      // visitor's own tile should be the flat, full-size, opaque one, not
      // whichever one the clock happened to choose. Left as a temporary
      // (hovering) override, not a permanent one: passing through on the
      // way to something else with Tab isn't the same declaration of
      // intent as reaching for a drag or a button.
      var index = tiles.indexOf(event.target);
      if (index !== -1 && index !== activeIndex) {
        activeIndex = index;
        layout(activeIndex);
      }
    });
    rail.addEventListener('focusout', function () { hovering = false; });

    // Manual prev/next. Once a visitor has reached for one of these, the
    // clock has been overruled for good — manualOverride, not hovering.
    var nav = document.querySelector('[data-fresh-nav]');
    if (nav) {
      var prevBtn = nav.querySelector('[data-fresh-prev]');
      var nextBtn = nav.querySelector('[data-fresh-next]');
      var goTo = function (newIndex) {
        manualOverride = true;
        activeIndex = ((newIndex % count) + count) % count;
        layout(activeIndex);
      };
      if (prevBtn) prevBtn.addEventListener('click', function () { goTo(activeIndex - 1); });
      if (nextBtn) nextBtn.addEventListener('click', function () { goTo(activeIndex + 1); });
      nav.hidden = false;
    }

    // Click-and-drag. DRAG_STEP_PX (how many px of drag equals one tile of
    // rotation) intentionally matches layout()'s own 140px-per-step
    // translateX above: dragging a tile by roughly its own visual travel
    // distance advances exactly one slot, so the ring tracks the pointer
    // 1:1 rather than at some unrelated, unpredictable rate.
    //
    // A pointer press alone doesn't mean a drag is happening — most
    // presses on a tile are the first half of an ordinary click.
    // dragMoved only flips true past DRAG_CLICK_THRESHOLD_PX of real
    // movement, and only then are transitions disabled (so the live
    // preview tracks the pointer with no easing lag) and manualOverride
    // set. The click handler below reads dragMoved to tell a real drag's
    // trailing click apart from an ordinary one.
    var DRAG_STEP_PX = 140;
    var DRAG_CLICK_THRESHOLD_PX = 6;
    var pointerDown = false;
    var dragMoved = false;
    var dragPointerId = null;
    var dragStartX = 0;
    var dragStartIndex = 0;

    rail.addEventListener('pointerdown', function (event) {
      if (event.button !== undefined && event.button !== 0) return;
      pointerDown = true;
      dragMoved = false;
      dragPointerId = event.pointerId;
      dragStartX = event.clientX;
      dragStartIndex = activeIndex;
      if (rail.setPointerCapture) rail.setPointerCapture(event.pointerId);
    });

    rail.addEventListener('pointermove', function (event) {
      if (!pointerDown || event.pointerId !== dragPointerId) return;
      var deltaX = event.clientX - dragStartX;
      if (!dragMoved) {
        if (Math.abs(deltaX) < DRAG_CLICK_THRESHOLD_PX) return;
        dragMoved = true;
        manualOverride = true;
        tiles.forEach(function (tile) { tile.style.setProperty('transition', 'none', 'important'); });
      }
      // Dragging left (negative deltaX) steps forward, the same direction
      // as the next button and the auto-advance timer — see
      // freshSignedOffset's own sign convention.
      layout(dragStartIndex - deltaX / DRAG_STEP_PX);
    });

    function endDrag(event) {
      if (!pointerDown || event.pointerId !== dragPointerId) return;
      pointerDown = false;
      if (rail.releasePointerCapture) {
        try { rail.releasePointerCapture(event.pointerId); } catch (error) { /* already released */ }
      }
      if (!dragMoved) return;
      var deltaX = event.clientX - dragStartX;
      var liveIndex = dragStartIndex - deltaX / DRAG_STEP_PX;
      activeIndex = ((Math.round(liveIndex) % count) + count) % count;
      tiles.forEach(function (tile) { tile.style.removeProperty('transition'); });
      layout(activeIndex);
    }
    rail.addEventListener('pointerup', endDrag);
    rail.addEventListener('pointercancel', endDrag);

    // Click. A tile that's already centred just navigates — plain <a>
    // behaviour, nothing to intercept. Any other tile recentres instead of
    // navigating: the visitor picked which one they want a closer look at,
    // not necessarily to leave the page yet. The click a real drag leaves
    // behind is suppressed outright rather than read as either — dragMoved
    // is consumed (reset) here so a single drag can only ever suppress the
    // one click it actually caused.
    rail.addEventListener('click', function (event) {
      if (dragMoved) {
        dragMoved = false;
        event.preventDefault();
        return;
      }
      var tile = event.target.closest ? event.target.closest('.fresh__tile') : null;
      if (!tile) return;
      var index = tiles.indexOf(tile);
      if (index === -1 || index === activeIndex) return;
      event.preventDefault();
      manualOverride = true;
      activeIndex = index;
      layout(activeIndex);
    });

    layout(activeIndex);
    window.setInterval(step, FRESH_COVERFLOW_STEP_MS);
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

    // The seal is the loaf mark and nothing else, so with no sprite to draw
    // from it is dropped whole rather than left as an empty gold ring. Its
    // markup comes from this file's own hardcoded icon sprite reference,
    // not from any localized or otherwise untrusted string, so building it
    // via innerHTML here (unlike the translated text below) carries no
    // injection risk.
    var sealMarkup = iconMarkup('icon-crust');
    if (sealMarkup) {
      var seal = document.createElement('span');
      seal.className = 'update-modal__seal';
      seal.setAttribute('aria-hidden', 'true');
      seal.innerHTML = sealMarkup;
      modal.appendChild(seal);
    }

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
    var reloadIconMarkup = iconMarkup('icon-rotate');
    if (reloadIconMarkup) {
      var reloadIconWrap = document.createElement('span');
      reloadIconWrap.innerHTML = reloadIconMarkup;
      reloadButton.appendChild(reloadIconWrap.firstChild);
    }
    var reloadLabelEl = document.createElement('span');
    reloadLabelEl.textContent = reloadLabel;
    reloadButton.appendChild(reloadLabelEl);

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

  // -------------------------------------------------------------------
  // Language switcher — the panel is a native <details>, so it opens,
  // closes, and is keyboard-reachable with no JS at all, and the CSS
  // takes it out of flow so it overlays rather than pushing the header.
  // The one behaviour a bare <details> lacks is dismissal: it stays open
  // until its own summary is clicked again, which for a popover anchored
  // in site chrome reads as stuck. This adds only that.
  // -------------------------------------------------------------------

  function initLanguageSwitcher() {
    var details = document.querySelector('.language-switcher');
    if (!details) return;

    document.addEventListener('click', function (event) {
      // contains() covers the summary itself, so clicking it still
      // toggles normally instead of being closed out from under the
      // browser's own default handling.
      if (details.open && !details.contains(event.target)) details.open = false;
    });

    document.addEventListener('keydown', function (event) {
      if (event.key !== 'Escape' || !details.open) return;
      details.open = false;
      // Escape moved focus nowhere on its own; without this it would be
      // left on a link inside a panel that no longer exists.
      var summary = details.querySelector('summary');
      if (summary) summary.focus();
    });
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
  // initSearch() counts cards to build its "showing all N" status. It also
  // has to run before initFreshCoverflow(), for the same shape of reason —
  // the coverflow's tile count needs to be whatever survived expiry, not
  // whatever the server happened to render before this visitor's clock had
  // a say.
  initFreshness();
  initFreshCoverflow();
  initSearch();
  initInstallCopy();
  initTabs();
  initUpdatedTime();
  initLanguageSwitcher();
  initScrollRestore();
  initVersionCheck();
})();
