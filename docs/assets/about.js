/*
 * TBaguette’s Atelier — about.js
 *
 * The live layer of the About page (/about/). The page is a whole résumé
 * without this file: scripts/about_page.py writes every fact as plain HTML and
 * about.css styles it. This adds what a document cannot do, and nothing the
 * reader needs in order to read it:
 *
 *   the timeline  scrub it, or press play, and the shelf fills; it is also the
 *                 page's navigator, and a loaf on it opens that entry
 *   the shelf     filters and search over the ledger, and each entry's
 *                 "How it's made" disclosure, linkable as #project=<slug>
 *   the opening   starts the loaf's bake when the loaf is first seen
 *   print         opens the bigger entries so the saved PDF carries their method
 *
 * Vanilla JS, no dependencies. With prefers-reduced-motion nothing animates by
 * itself: the timeline does not play on its own, and the opening is simply
 * there. Everything works from the keyboard, and every state that matters is
 * in the URL or in the markup.
 */
(function () {
  'use strict';

  if (window.TBaguetteAbout) { return; }
  var root = document.querySelector('[data-about]');
  if (!root) { return; }
  window.TBaguetteAbout = true;

  var mq = window.matchMedia ? window.matchMedia('(prefers-reduced-motion: reduce)') : null;

  function reduced() { return !!(mq && mq.matches); }
  function $(sel, from) { return (from || root).querySelector(sel); }
  function $$(sel, from) { return Array.prototype.slice.call((from || root).querySelectorAll(sel)); }
  function clamp(v, lo, hi) { return v < lo ? lo : (v > hi ? hi : v); }
  function lerp(a, b, t) { return a + (b - a) * t; }

  // -------------------------------------------------------------------------
  // What the shelf is showing. The family chips, the ingredient chips and the
  // search set the filters; the timeline sets a cutoff, but only while the
  // reader is driving it, so the page never dims itself on its own.
  // -------------------------------------------------------------------------

  var state = { domain: 'all', tag: null, q: '' };
  var cutoff = null; // percent along the timeline, or null for "all of it"
  var entries = [];  // one per project, in ledger order
  var bySlug = {};
  var tagNames = {};
  var domainNames = {};
  var drawn = function () {};

  function goLive() {
    root.classList.add('about-live');
    // The loaf's bake starts when the loaf is first seen: at once on a wide
    // screen, and on a phone, where it sits below the buttons, when it scrolls
    // into view rather than offscreen where nobody would watch it.
    var stage = $('[data-about-stage]');
    if (stage && 'IntersectionObserver' in window) {
      var io = new IntersectionObserver(function (list) {
        if (!list[0].isIntersecting) { return; }
        io.disconnect();
        root.classList.add('loaf-seen');
      }, { threshold: 0.2 });
      io.observe(stage);
    } else {
      root.classList.add('loaf-seen');
    }
  }

  // -------------------------------------------------------------------------
  // The shelf: filters, search, and the entries' disclosures
  // -------------------------------------------------------------------------

  function initShelf() {
    var shelf = $('[data-shelf]');
    if (!shelf) { return; }
    var status = $('[data-status]', shelf);
    var empty = $('[data-empty]', shelf);
    var search = $('[data-search]', shelf);
    var domainChips = $$('[data-domain-filter]', shelf);
    var tagChips = $$('[data-tag]', shelf);
    var groups = $$('[data-era-group]', shelf);
    var total = 0;

    domainChips.forEach(function (c) {
      domainNames[c.getAttribute('data-domain-filter')] = (c.firstChild.textContent || '').trim();
    });
    tagChips.forEach(function (c) {
      tagNames[c.getAttribute('data-tag')] = (c.firstChild.textContent || '').trim();
    });

    var dotX = {};
    $$('[data-rise-slot]').forEach(function (s) { dotX[s.getAttribute('data-slug')] = parseFloat(s.getAttribute('data-x')); });

    entries = $$('[data-slot]', shelf).map(function (slot) {
      var el = $('.entry', slot);
      var slug = el.getAttribute('data-project');
      var e = {
        slot: slot, el: el, slug: slug,
        domain: el.getAttribute('data-domain'),
        tags: ' ' + el.getAttribute('data-tags') + ' ',
        hay: (el.textContent || '').toLowerCase().replace(/\s+/g, ' '),
        more: $('[data-more]', el),
        x: dotX[slug] || 0
      };
      bySlug[slug] = e;
      return e;
    });
    total = entries.length;

    function matches(e) {
      if (state.domain !== 'all' && e.domain !== state.domain) { return false; }
      if (state.tag && e.tags.indexOf(' ' + state.tag + ' ') < 0) { return false; }
      if (state.q) {
        var words = state.q.split(' ');
        for (var i = 0; i < words.length; i++) { if (words[i] && e.hay.indexOf(words[i]) < 0) { return false; } }
      }
      return true;
    }

    function filtered() { return state.domain !== 'all' || !!state.tag || !!state.q; }

    function draw() {
      domainChips.forEach(function (c) {
        c.setAttribute('aria-pressed', c.getAttribute('data-domain-filter') === state.domain ? 'true' : 'false');
      });
      tagChips.forEach(function (c) {
        c.setAttribute('aria-pressed', c.getAttribute('data-tag') === state.tag ? 'true' : 'false');
      });
      if (search && search.value.toLowerCase().replace(/\s+/g, ' ').trim() !== state.q) { search.value = state.q; }

      var shown = 0;
      entries.forEach(function (e) {
        var ok = matches(e);
        e.slot.hidden = !ok;
        if (ok) { shown++; }
        e.el.classList.toggle('is-unborn', cutoff !== null && e.x > cutoff + 0.0001);
      });
      groups.forEach(function (g) {
        g.hidden = !$$('[data-slot]', g).some(function (s) { return !s.hidden; });
      });

      var parts = [];
      if (state.domain !== 'all') { parts.push(domainNames[state.domain]); }
      if (state.tag) { parts.push(tagNames[state.tag]); }
      if (state.q) { parts.push('“' + state.q + '”'); }
      if (status) {
        status.textContent = (filtered() ? shown + ' of ' + total + ' projects · ' + parts.join(' · ') + ' ' : total + ' projects.');
        if (filtered()) {
          var clear = document.createElement('button');
          clear.type = 'button';
          clear.className = 'linkish';
          clear.textContent = 'Clear the filters';
          clear.addEventListener('click', reset);
          status.appendChild(clear);
        }
      }
      if (empty) { empty.hidden = shown !== 0; }
    }
    drawn = draw;

    function reset() { state.domain = 'all'; state.tag = null; state.q = ''; draw(); }

    domainChips.forEach(function (c) {
      c.addEventListener('click', function () { state.domain = c.getAttribute('data-domain-filter'); draw(); });
    });
    tagChips.forEach(function (c) {
      c.addEventListener('click', function () {
        var tag = c.getAttribute('data-tag');
        state.tag = state.tag === tag ? null : tag;
        draw();
      });
    });
    if (search) {
      var timer = 0;
      search.addEventListener('input', function () {
        window.clearTimeout(timer);
        timer = window.setTimeout(function () {
          state.q = search.value.toLowerCase().replace(/\s+/g, ' ').trim();
          draw();
        }, 120);
      });
    }
    var resetButton = $('[data-reset]', shelf);
    if (resetButton) { resetButton.addEventListener('click', reset); }

    // "/" jumps to the search from anywhere on the page, the way it does on
    // the rest of the site's search fields.
    document.addEventListener('keydown', function (e) {
      if (e.key !== '/' || e.ctrlKey || e.metaKey || e.altKey || !search) { return; }
      var t = e.target;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) { return; }
      e.preventDefault();
      search.focus();
      search.scrollIntoView({ behavior: reduced() ? 'auto' : 'smooth', block: 'center' });
    });

    // An entry's disclosure is linkable: opening one puts #project=<slug> in
    // the address, closing it takes it out, so a link to the open entry can
    // be sent and Back behaves.
    entries.forEach(function (e) {
      if (!e.more) { return; }
      e.more.addEventListener('toggle', function () {
        var base = window.location.pathname + window.location.search;
        try {
          if (e.more.open) { history.replaceState(null, '', base + '#project=' + e.slug); }
          else if (window.location.hash === '#project=' + e.slug) { history.replaceState(null, '', base); }
        } catch (err) { /* a sandboxed frame may refuse; the disclosure still works */ }
      });
    });

    draw();
  }

  // Go to a project: show it if a filter or the timeline is hiding it, open
  // its disclosure, bring it to the top, and mark it for a moment.
  function openEntry(slug) {
    var e = bySlug[slug];
    if (!e) { return; }
    if (e.slot.hidden) { state.domain = 'all'; state.tag = null; state.q = ''; }
    cutoff = null;
    drawn();
    if (e.more) { e.more.open = true; }
    e.el.scrollIntoView({ behavior: reduced() ? 'auto' : 'smooth', block: 'start' });
    e.el.classList.add('is-target');
    window.setTimeout(function () { e.el.classList.remove('is-target'); }, 2400);
    var summary = e.more ? $('summary', e.more) : null;
    if (summary && summary.focus) { summary.focus({ preventScroll: true }); }
  }

  function initLinks() {
    // Every link to a project goes to its entry. Followed with script off it
    // still lands on the entry (the id is the fragment); with script on it
    // also opens it. The address is updated with pushState, not by assigning
    // location.hash: that keeps Back stepping through the entries a reader
    // visited, and it still works where a sandboxed frame will not let the
    // page change its own hash.
    root.addEventListener('click', function (ev) {
      var a = ev.target.closest ? ev.target.closest('a[href^="#project="]') : null;
      if (!a || ev.metaKey || ev.ctrlKey || ev.shiftKey) { return; }
      ev.preventDefault();
      var hash = a.getAttribute('href');
      if (window.location.hash !== hash) {
        try { history.pushState(null, '', window.location.pathname + window.location.search + hash); } catch (err) { /* the entry still opens */ }
      }
      openEntry(hash.slice(9));
    });
    function fromHash() {
      var m = /^#project=([a-z0-9-]+)$/.exec(window.location.hash);
      if (m) { openEntry(m[1]); }
    }
    window.addEventListener('hashchange', fromHash);
    window.addEventListener('popstate', fromHash);
    if (/^#project=/.test(window.location.hash)) { fromHash(); }
  }

  // -------------------------------------------------------------------------
  // The timeline
  // -------------------------------------------------------------------------

  function initRise() {
    var rise = $('[data-rise]');
    if (!rise) { return; }
    var track = $('[data-rise-track]', rise);
    var scroller = $('[data-rise-scroll]', rise);
    var range = $('[data-rise-range]', rise);
    var play = $('[data-rise-play]', rise);
    var readout = $('[data-rise-readout]', rise);
    var languages = {};
    $$('[data-tag]').forEach(function (c) { languages[c.getAttribute('data-tag')] = false; });
    var dots = $$('[data-rise-slot]', rise).map(function (el) {
      var e = bySlug[el.getAttribute('data-slug')];
      return { el: el, x: parseFloat(el.getAttribute('data-x')), tags: e ? e.tags.trim().split(' ') : [], born: true };
    });
    var warp = (rise.getAttribute('data-warp') || '').split(',').map(parseFloat);
    var firstYear = parseInt(rise.getAttribute('data-first-year'), 10);
    var months = warp.length - 1;
    var MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    var total = dots.length;
    if (!track || !range || !play || months < 1) { return; }

    // Which tags are languages: the ingredient chips are grouped, and the
    // group's label says which.
    $$('.ingredients__group').forEach(function (g) {
      var label = $('.ingredients__label', g);
      if (label && label.textContent === 'Languages') {
        $$('[data-tag]', g).forEach(function (c) { languages[c.getAttribute('data-tag')] = true; });
      }
    });

    var now = document.createElement('div');
    now.className = 'rise__now';
    now.setAttribute('aria-hidden', 'true');
    track.appendChild(now);

    function monthAt(pct) {
      var m = 0;
      for (var i = 0; i < months; i++) { if (warp[i] <= pct) { m = i; } }
      return m;
    }

    // `driven` is true when the reader is doing it (scrubbing, or pressing
    // play): only then does the ledger below follow. The one automatic play
    // on first sight moves the dots alone.
    function render(frac, quiet, driven) {
      var pct = frac * 100;
      track.style.setProperty('--now', pct.toFixed(2) + '%');
      range.style.setProperty('--p', pct.toFixed(1) + '%');
      var out = 0, langs = {};
      dots.forEach(function (d) {
        var born = d.x <= pct + 0.0001;
        if (born !== d.born) {
          d.el.classList.toggle('is-unborn', !born);
          d.el.classList.toggle('is-born', born && !quiet);
          if (born && !quiet) { window.setTimeout(function () { d.el.classList.remove('is-born'); }, 520); }
          d.born = born;
        }
        if (born) {
          out++;
          d.tags.forEach(function (t) { if (languages[t]) { langs[t] = true; } });
        }
      });
      var nl = Object.keys(langs).length;
      readout.textContent = '';
      if (out === total) {
        readout.textContent = 'All ' + total + ' loaves are out.';
      } else {
        var m = monthAt(pct);
        var b = document.createElement('b');
        b.textContent = MONTHS[m % 12] + ' ' + (firstYear + Math.floor(m / 12));
        readout.appendChild(b);
        readout.appendChild(document.createTextNode(out === 0
          ? ' · the oven is still cold.'
          : ' · ' + out + (out === 1 ? ' loaf is' : ' loaves are') + ' out, in ' + nl + (nl === 1 ? ' language.' : ' languages.')));
      }
      if (driven) { cutoff = out === total ? null : pct; drawn(); }
    }

    function frac() { return parseInt(range.value, 10) / 1000; }
    var raf = 0;

    function stop() {
      if (raf) { cancelAnimationFrame(raf); raf = 0; }
      play.textContent = frac() >= 1 ? 'Replay the years' : 'Play the years';
    }

    range.addEventListener('input', function () { stop(); render(frac(), false, true); });

    function run(from, ms, driven) {
      stop();
      var t0 = 0;
      play.textContent = 'Pause';
      var tick = function (t) {
        if (!t0) { t0 = t; }
        var k = clamp((t - t0) / ms, 0, 1);
        var f = lerp(from, 1, k);
        range.value = String(Math.round(f * 1000));
        render(f, false, driven);
        // Keep the playhead in view on a phone, where the track scrolls.
        if (scroller && scroller.scrollWidth > scroller.clientWidth) {
          var x = f * track.offsetWidth;
          if (x < scroller.scrollLeft + 60 || x > scroller.scrollLeft + scroller.clientWidth - 60) {
            scroller.scrollLeft = x - scroller.clientWidth * 0.5;
          }
        }
        if (k < 1) { raf = requestAnimationFrame(tick); } else { raf = 0; stop(); }
      };
      raf = requestAnimationFrame(tick);
    }

    play.addEventListener('click', function () {
      if (raf) { stop(); return; }
      var f = frac();
      if (f >= 1) { f = 0; range.value = '0'; render(0, true, true); }
      run(f, Math.max(2500, (1 - f) * 11000), true);
    });

    // Start on the finished shelf. The first time it is seen, empty the
    // timeline and fill it once, so what it does is shown rather than
    // described; the ledger is left alone, and a reader who asked for no
    // motion gets the finished timeline and nothing else.
    render(1, true, false);
    if (scroller) { scroller.scrollLeft = scroller.scrollWidth; }
    if (!reduced() && 'IntersectionObserver' in window) {
      var seen = new IntersectionObserver(function (list) {
        if (!list[0].isIntersecting) { return; }
        seen.disconnect();
        range.value = '0';
        render(0, true, false);
        if (scroller) { scroller.scrollLeft = 0; }
        window.setTimeout(function () { run(0, 8000, false); }, 400);
      }, { threshold: 0.6 });
      seen.observe(track);
    }
  }

  // -------------------------------------------------------------------------
  // Print
  // -------------------------------------------------------------------------

  function initPrint() {
    $$('[data-about-print]').forEach(function (b) {
      b.addEventListener('click', function () { window.print(); });
    });
    // The saved PDF carries the method of the bigger entries, so they are
    // opened for it and put back after (about.css hides the smaller ones'
    // method on paper).
    var opened = [];
    window.addEventListener('beforeprint', function () {
      opened = [];
      entries.forEach(function (e) {
        var scale = parseInt(e.el.getAttribute('data-scale'), 10) || 0;
        if (e.more && scale >= 4 && !e.more.open) { e.more.open = true; opened.push(e.more); }
      });
    });
    window.addEventListener('afterprint', function () {
      opened.forEach(function (d) { d.open = false; });
      opened = [];
    });
  }

  function boot() {
    goLive();
    initShelf();
    initLinks();
    initRise();
    initPrint();
  }

  if (document.readyState === 'loading') { document.addEventListener('DOMContentLoaded', boot); } else { boot(); }
})();
