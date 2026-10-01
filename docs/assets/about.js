/*
 * TBaguette’s Atelier — about.js
 *
 * The live layer of the About page (/about/). The page is a whole résumé
 * without this file: scripts/about_page.py writes every fact as plain HTML
 * and about.css styles it. This adds what a document cannot do, and nothing
 * the reader needs in order to read it:
 *
 *   the hero    letters that rise, a loaf that bakes and opens its scores, a
 *               pointer that lights it, flour that drifts, crumbs when it is
 *               poked, and numbers that count up
 *   the title   three words that open onto the work that backs them
 *   the pantry  ingredients as bubbles that settle, flinch from the pointer
 *               and, when picked, filter the bakery
 *   the bakery  filters and search that re-seat the loaves with a FLIP, and
 *               a recipe card (a native <dialog>) for each project
 *   the rise    a timeline you can scrub or play, which fills the shelf
 *   the frame   a progress bar, a chapter rail, and a baking loaf that rides it
 *
 * Vanilla JS, no dependencies. Motion is decoration only in the sense that
 * nothing depends on it: with prefers-reduced-motion every animation is
 * skipped, the dust never starts, the bubbles stay as chips and the counters
 * show their final numbers. Print never sees any of it (about.css reads the
 * plain layer).
 */
(function () {
  'use strict';

  if (window.TBaguetteAbout) { return; }
  var root = document.querySelector('[data-about]');
  if (!root) { return; }
  window.TBaguetteAbout = true;

  var doc = document.documentElement;
  var mq = window.matchMedia ? window.matchMedia('(prefers-reduced-motion: reduce)') : null;

  function reduced() { return !!(mq && mq.matches); }
  function $(sel, from) { return (from || root).querySelector(sel); }
  function $$(sel, from) { return Array.prototype.slice.call((from || root).querySelectorAll(sel)); }
  function clamp(v, lo, hi) { return v < lo ? lo : (v > hi ? hi : v); }
  function lerp(a, b, t) { return a + (b - a) * t; }
  function easeOutExpo(t) { return t >= 1 ? 1 : 1 - Math.pow(2, -10 * t); }
  function isFlour() { return doc.getAttribute('data-theme') === 'flour'; }
  function finePointer(e) { return e.pointerType === 'mouse' || e.pointerType === 'pen'; }

  // Throttle a function to one call per animation frame.
  function frame(fn) {
    var queued = false;
    return function () {
      if (queued) { return; }
      queued = true;
      requestAnimationFrame(function () { queued = false; fn(); });
    };
  }

  // -------------------------------------------------------------------------
  // Shared state: what the bakery is showing. The pantry sets a tag, the
  // chips set a domain, the search sets a query; the bakery draws the result.
  // -------------------------------------------------------------------------

  var state = { domain: 'all', tag: null, q: '' };
  var listeners = [];
  function onState(fn) { listeners.push(fn); }
  function setState(patch) {
    for (var k in patch) { if (Object.prototype.hasOwnProperty.call(patch, k)) { state[k] = patch[k]; } }
    for (var i = 0; i < listeners.length; i++) { listeners[i](); }
  }

  // The tag labels, from the pantry itself, so a tag is named once.
  var tagNames = {};
  var tagKinds = {};
  $$('.ingredient').forEach(function (b) {
    var name = $('.ingredient__name', b);
    tagNames[b.getAttribute('data-tag')] = name ? name.textContent : b.getAttribute('data-tag');
    tagKinds[b.getAttribute('data-tag')] = b.getAttribute('data-kind');
  });

  // -------------------------------------------------------------------------
  // Going live
  // -------------------------------------------------------------------------

  var hasDialog = typeof HTMLDialogElement === 'function' && !!document.createElement('dialog').showModal;

  function goLive() {
    root.classList.add('about-live');
    if (!hasDialog) { root.classList.add('about-no-dialog'); }
    // One frame later, so the hidden starting states are painted before the
    // entrance runs; otherwise the first frame would show the finished page.
    requestAnimationFrame(function () {
      requestAnimationFrame(function () { root.classList.add('is-ready'); });
    });
  }

  // -------------------------------------------------------------------------
  // Progress, rail and the baking loaf
  // -------------------------------------------------------------------------

  function initFrame() {
    var hero = $('[data-about-hero]');
    var stops = $$('[data-rail-stop]');
    var chapters = $$('[data-chapter]');

    function update() {
      var max = Math.max(1, doc.scrollHeight - window.innerHeight);
      var p = clamp(window.pageYOffset / max, 0, 1);
      root.style.setProperty('--progress', p.toFixed(4));
      root.style.setProperty('--bake', p.toFixed(4));
      if (hero) {
        var r = hero.getBoundingClientRect();
        root.classList.toggle('is-past-hero', r.bottom < window.innerHeight * 0.5);
      }
      var line = window.innerHeight * 0.4;
      var here = -1;
      for (var i = 0; i < chapters.length; i++) {
        if (chapters[i].getBoundingClientRect().top <= line) { here = i; }
      }
      stops.forEach(function (s, i) {
        s.classList.toggle('is-here', i === here);
        s.classList.toggle('is-done', i < here);
        if (i === here) { s.setAttribute('aria-current', 'location'); } else { s.removeAttribute('aria-current'); }
      });
    }

    var queued = frame(update);
    window.addEventListener('scroll', queued, { passive: true });
    window.addEventListener('resize', queued, { passive: true });
    update();
  }

  // -------------------------------------------------------------------------
  // The hero
  // -------------------------------------------------------------------------

  var dust = null; // the flour canvas, once built (see initDust)

  function initHero() {
    var hero = $('[data-about-hero]');
    var stage = $('[data-about-stage]');
    var name = $('.about-hero__name');
    var loaf = $('.about-hero__loaf');
    if (!hero) { return; }

    // After the entrance, the name stops being an animation and starts being
    // a type specimen: the weight of each letter swells as the pointer nears.
    var letters = $$('.about-hero__ch', name);
    if (name && letters.length) {
      var last = letters[letters.length - 1];
      var settled = false;
      var settle = function () {
        if (settled) { return; }
        settled = true;
        name.classList.add('is-set');
        letters.forEach(function (l) { l.style.fontWeight = '800'; });
      };
      if (reduced()) { settle(); } else {
        last.addEventListener('animationend', settle);
        window.setTimeout(settle, 3200); // if the animation never reports
      }
      hero.addEventListener('pointermove', function (e) {
        if (!settled || !finePointer(e) || reduced()) { return; }
        letters.forEach(function (l) {
          var r = l.getBoundingClientRect();
          var d = Math.hypot(e.clientX - (r.left + r.width / 2), (e.clientY - (r.top + r.height / 2)) * 0.6);
          l.style.fontWeight = String(Math.round(lerp(900, 800, clamp(d / 240, 0, 1))));
        });
      }, { passive: true });
      hero.addEventListener('pointerleave', function () {
        if (!settled) { return; }
        letters.forEach(function (l) { l.style.fontWeight = '800'; });
      });
    }

    // The loaf follows the pointer a little, and its highlight a lot.
    if (stage) {
      var tx = 0, ty = 0, cx = 0, cy = 0, raf = 0;
      var step = function () {
        cx = lerp(cx, tx, 0.08);
        cy = lerp(cy, ty, 0.08);
        stage.style.setProperty('--px', cx.toFixed(3));
        stage.style.setProperty('--py', cy.toFixed(3));
        raf = (Math.abs(cx - tx) + Math.abs(cy - ty) > 0.002) ? requestAnimationFrame(step) : 0;
      };
      hero.addEventListener('pointermove', function (e) {
        if (!finePointer(e) || reduced()) { return; }
        var r = hero.getBoundingClientRect();
        tx = clamp(((e.clientX - r.left) / r.width) * 2 - 1, -1, 1);
        ty = clamp(((e.clientY - r.top) / r.height) * 2 - 1, -1, 1);
        if (!raf) { raf = requestAnimationFrame(step); }
      }, { passive: true });
      hero.addEventListener('pointerleave', function () { tx = 0; ty = 0; if (!raf) { raf = requestAnimationFrame(step); } });
    }

    // The loaf goes in the oven when it is first seen: at once on a wide
    // screen, and on a phone, where it sits below the buttons, when it
    // scrolls into view rather than offscreen where nobody would watch it.
    if (stage && 'IntersectionObserver' in window) {
      var seen = new IntersectionObserver(function (entries) {
        if (!entries[0].isIntersecting) { return; }
        seen.disconnect();
        root.classList.add('loaf-seen');
      }, { threshold: 0.2 });
      seen.observe(stage);
    } else {
      root.classList.add('loaf-seen');
    }

    // Poke the loaf: it gives, and crumbs fly.
    if (loaf) {
      loaf.addEventListener('click', function (e) {
        if (reduced()) { return; }
        loaf.classList.remove('is-poked');
        void loaf.offsetWidth;
        loaf.classList.add('is-poked');
        if (dust) { dust.burst(e.clientX, e.clientY); }
      });
    }
  }

  // -------------------------------------------------------------------------
  // Flour dust: slow motes, a pointer that stirs them, and crumbs on a poke.
  // -------------------------------------------------------------------------

  function initDust() {
    var canvas = $('[data-about-dust]');
    var hero = $('[data-about-hero]');
    if (!canvas || !canvas.getContext || !hero || reduced()) { return; }
    var ctx = canvas.getContext('2d');
    var w = 0, h = 0, dpr = 1;
    var motes = [];
    var crumbs = [];
    var pointer = { x: -999, y: -999 };
    var running = false, visible = true, last = 0;

    function resize() {
      var r = hero.getBoundingClientRect();
      dpr = Math.min(2, window.devicePixelRatio || 1);
      w = Math.max(1, Math.round(r.width));
      h = Math.max(1, Math.round(r.height));
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      var want = clamp(Math.round((w * h) / 16000), 24, 90);
      while (motes.length < want) { motes.push(spawn(true)); }
      motes.length = want;
    }

    function spawn(anywhere) {
      return {
        x: Math.random() * w,
        y: anywhere ? Math.random() * h : h + 10,
        r: 0.6 + Math.random() * 1.9,
        a: 0.12 + Math.random() * 0.4,
        vy: -(6 + Math.random() * 16),
        ph: Math.random() * 6.28,
        sp: 0.3 + Math.random() * 0.8,
        vx: 0
      };
    }

    function tone() { return isFlour() ? '168,111,52' : '245,234,217'; }

    function tick(now) {
      if (!running) { return; }
      var dt = Math.min(0.05, (now - (last || now)) / 1000);
      last = now;
      ctx.clearRect(0, 0, w, h);
      var base = tone();
      var i, m;
      for (i = 0; i < motes.length; i++) {
        m = motes[i];
        m.ph += dt * m.sp;
        var dx = m.x - pointer.x, dy = m.y - pointer.y, d2 = dx * dx + dy * dy;
        if (d2 < 14400) {
          var d = Math.sqrt(d2) || 1, f = (1 - d / 120) * 260;
          m.vx += (dx / d) * f * dt;
          m.vy += (dy / d) * f * dt * 0.6;
        }
        m.vx *= 0.96;
        m.x += (Math.sin(m.ph) * 9 + m.vx) * dt;
        m.y += m.vy * dt;
        m.vy = lerp(m.vy, -(6 + m.r * 5), 0.01);
        if (m.y < -12 || m.x < -20 || m.x > w + 20) { motes[i] = spawn(false); continue; }
        ctx.fillStyle = 'rgba(' + base + ',' + (m.a * (0.6 + 0.4 * Math.sin(m.ph * 1.7))).toFixed(3) + ')';
        ctx.beginPath();
        ctx.arc(m.x, m.y, m.r, 0, 6.2832);
        ctx.fill();
      }
      for (i = crumbs.length - 1; i >= 0; i--) {
        var c = crumbs[i];
        c.life -= dt;
        if (c.life <= 0) { crumbs.splice(i, 1); continue; }
        c.vy += 520 * dt;
        c.x += c.vx * dt;
        c.y += c.vy * dt;
        c.rot += c.vr * dt;
        ctx.save();
        ctx.translate(c.x, c.y);
        ctx.rotate(c.rot);
        ctx.fillStyle = 'rgba(' + c.col + ',' + clamp(c.life * 1.6, 0, 1).toFixed(3) + ')';
        ctx.fillRect(-c.s, -c.s * 0.6, c.s * 2, c.s * 1.2);
        ctx.restore();
      }
      requestAnimationFrame(tick);
    }

    function start() { if (!running && visible && !document.hidden) { running = true; last = 0; requestAnimationFrame(tick); } }
    function stop() { running = false; }

    hero.addEventListener('pointermove', function (e) {
      if (!finePointer(e)) { return; }
      var r = hero.getBoundingClientRect();
      pointer.x = e.clientX - r.left;
      pointer.y = e.clientY - r.top;
    }, { passive: true });
    hero.addEventListener('pointerleave', function () { pointer.x = pointer.y = -999; });
    document.addEventListener('visibilitychange', function () { if (document.hidden) { stop(); } else { start(); } });
    window.addEventListener('resize', frame(resize), { passive: true });
    if (mq && mq.addEventListener) { mq.addEventListener('change', function () { if (reduced()) { stop(); ctx.clearRect(0, 0, w, h); } else { start(); } }); }

    if ('IntersectionObserver' in window) {
      new IntersectionObserver(function (entries) {
        visible = entries[0].isIntersecting;
        if (visible) { start(); } else { stop(); }
      }).observe(hero);
    }

    resize();
    start();

    dust = {
      burst: function (px, py) {
        var r = hero.getBoundingClientRect();
        var cols = ['232,184,118', '221,162,92', '185,119,47', '255,243,214'];
        for (var i = 0; i < 26; i++) {
          var ang = -Math.PI / 2 + (Math.random() - 0.5) * 2.4;
          var sp = 120 + Math.random() * 300;
          crumbs.push({
            x: px - r.left, y: py - r.top,
            vx: Math.cos(ang) * sp, vy: Math.sin(ang) * sp,
            s: 1.4 + Math.random() * 2.6, rot: Math.random() * 6, vr: (Math.random() - 0.5) * 12,
            life: 0.7 + Math.random() * 0.6, col: cols[i % cols.length]
          });
        }
        start();
      }
    };
  }

  // -------------------------------------------------------------------------
  // Counters
  // -------------------------------------------------------------------------

  function initCounters() {
    var els = $$('[data-count]');
    if (!els.length || reduced() || !('IntersectionObserver' in window)) { return; }
    els.forEach(function (el) { el.textContent = '0'; });
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) { return; }
        io.unobserve(entry.target);
        var el = entry.target;
        var to = parseInt(el.getAttribute('data-count'), 10) || 0;
        var t0 = null;
        var run = function (t) {
          if (t0 === null) { t0 = t; }
          var k = clamp((t - t0) / 1500, 0, 1);
          el.textContent = String(Math.round(to * easeOutExpo(k)));
          if (k < 1) { requestAnimationFrame(run); }
        };
        requestAnimationFrame(run);
      });
    }, { threshold: 0.6 });
    els.forEach(function (el) { io.observe(el); });
  }

  // -------------------------------------------------------------------------
  // Scroll reveal
  // -------------------------------------------------------------------------

  function initReveal() {
    var els = $$('[data-reveal]');
    if (!('IntersectionObserver' in window)) {
      els.forEach(function (el) { el.classList.add('is-in'); });
      return;
    }
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) { entry.target.classList.add('is-in'); io.unobserve(entry.target); }
      });
    // threshold 0, not a fraction: a block taller than the screen (the whole
    // bakery is, on a phone) can never have a tenth of itself in view, and
    // would be waited for forever.
    }, { rootMargin: '0px 0px -8% 0px', threshold: 0 });
    els.forEach(function (el) { io.observe(el); });
  }

  // -------------------------------------------------------------------------
  // The title's anatomy
  // -------------------------------------------------------------------------

  function initAnatomy() {
    var box = $('[data-anatomy]');
    if (!box) { return; }
    var words = $$('[data-claim]', box);
    var panels = words.map(function (w) { return document.getElementById(w.getAttribute('aria-controls')); });
    function open(i) {
      words.forEach(function (w, j) {
        w.setAttribute('aria-expanded', j === i ? 'true' : 'false');
        if (panels[j]) { panels[j].hidden = j !== i; }
      });
    }
    words.forEach(function (w, i) { w.addEventListener('click', function () { open(i); }); });
    open(0);
  }

  // -------------------------------------------------------------------------
  // The pantry: bubbles
  // -------------------------------------------------------------------------

  function initPantry() {
    var host = $('[data-pantry]');
    var field = $('[data-pantry-field]');
    var status = $('[data-pantry-status]');
    var buttons = $$('.ingredient');
    if (!host || !field || !buttons.length) { return; }
    var total = $$('.loaf').length;

    buttons.forEach(function (b, i) {
      b.style.setProperty('--i', String(i));
      b.addEventListener('click', function () {
        var tag = b.getAttribute('data-tag');
        setState({ tag: state.tag === tag ? null : tag });
      });
    });

    function describe() {
      buttons.forEach(function (b) { b.setAttribute('aria-pressed', b.getAttribute('data-tag') === state.tag ? 'true' : 'false'); });
      if (!status) { return; }
      if (!state.tag) { status.textContent = 'Nothing picked: all of the shelf is showing.'; return; }
      var n = $$('.loaf').filter(function (l) { return (' ' + l.getAttribute('data-tags') + ' ').indexOf(' ' + state.tag + ' ') > -1; }).length;
      status.textContent = tagNames[state.tag] + ' is in ' + n + ' of ' + total + ' loaves. ';
      var go = document.createElement('button');
      go.type = 'button';
      go.textContent = 'See them';
      go.addEventListener('click', function () {
        var target = document.getElementById('bakery');
        if (target) { target.scrollIntoView({ behavior: reduced() ? 'auto' : 'smooth', block: 'start' }); }
      });
      status.appendChild(go);
      var clear = document.createElement('button');
      clear.type = 'button';
      clear.textContent = 'Put it back';
      clear.addEventListener('click', function () { setState({ tag: null }); });
      status.appendChild(clear);
    }
    onState(describe);

    // The bubbles. Wide screens, and only if motion is welcome: otherwise the
    // chips stay chips and the shelf is complete.
    function physicsWanted() { return window.innerWidth >= 720 && !reduced(); }
    var bodies = [];
    var W = 0, H = 0;
    var pointer = { x: -999, y: -999, on: false };
    var running = false, onScreen = false, built = false, last = 0, sleepFor = 0;

    function size() {
      var r = field.getBoundingClientRect();
      W = r.width; H = r.height;
    }

    function build() {
      size();
      var maxW = 1;
      buttons.forEach(function (b) { maxW = Math.max(maxW, parseInt(b.getAttribute('data-weight'), 10) || 1); });
      var k = clamp(W / 1100, 0.72, 1.1);
      var order = buttons.slice().sort(function (a, b) { return (b.getAttribute('data-weight') | 0) - (a.getAttribute('data-weight') | 0); });
      bodies = order.map(function (b, i) {
        var wgt = parseInt(b.getAttribute('data-weight'), 10) || 1;
        var r = (40 + 46 * Math.sqrt(wgt / maxW)) * k;
        // Phyllotaxis from the centre: biggest in the middle, the rest in a spiral.
        var ang = i * 2.39996, rad = 14 * Math.sqrt(i + 1) * 7 * k;
        b.style.width = b.style.height = (r * 2).toFixed(1) + 'px';
        b.style.setProperty('--d', (r * 2).toFixed(1) + 'px');
        return { el: b, r: r, x: W / 2 + Math.cos(ang) * rad, y: H / 2 + Math.sin(ang) * rad * 0.7, vx: 0, vy: 0, ph: i * 1.3 };
      });
      for (var n = 0; n < 260; n++) { step(0.016, true); }
      built = true;
      paint(0);
    }

    function step(dt, silent) {
      var i, j, a, b, dx, dy, d, min, push;
      var cx = W / 2, cy = H / 2;
      var energy = 0;
      for (i = 0; i < bodies.length; i++) {
        a = bodies[i];
        a.vx += (cx - a.x) * 0.9 * dt * 0.02 * (a.r / 60);
        a.vy += (cy - a.y) * 0.9 * dt * 0.034 * (a.r / 60);
        if (pointer.on && !silent) {
          dx = a.x - pointer.x; dy = a.y - pointer.y;
          d = Math.sqrt(dx * dx + dy * dy) || 1;
          var reach = a.r + 90;
          if (d < reach) { push = (1 - d / reach) * 700 * dt; a.vx += (dx / d) * push; a.vy += (dy / d) * push; }
        }
      }
      for (i = 0; i < bodies.length; i++) {
        a = bodies[i];
        for (j = i + 1; j < bodies.length; j++) {
          b = bodies[j];
          dx = b.x - a.x; dy = b.y - a.y;
          d = Math.sqrt(dx * dx + dy * dy) || 0.01;
          min = a.r + b.r + 5;
          if (d < min) {
            var o = (min - d) / 2, nx = dx / d, ny = dy / d;
            var wa = b.r / (a.r + b.r), wb = a.r / (a.r + b.r);
            a.x -= nx * o * 2 * wa; a.y -= ny * o * 2 * wa;
            b.x += nx * o * 2 * wb; b.y += ny * o * 2 * wb;
            a.vx -= nx * o * 0.4; a.vy -= ny * o * 0.4;
            b.vx += nx * o * 0.4; b.vy += ny * o * 0.4;
          }
        }
      }
      for (i = 0; i < bodies.length; i++) {
        a = bodies[i];
        a.vx *= 0.9; a.vy *= 0.9;
        a.x += a.vx * dt * 60; a.y += a.vy * dt * 60;
        if (a.x < a.r + 4) { a.x = a.r + 4; a.vx = Math.abs(a.vx) * 0.5; }
        if (a.x > W - a.r - 4) { a.x = W - a.r - 4; a.vx = -Math.abs(a.vx) * 0.5; }
        if (a.y < a.r + 4) { a.y = a.r + 4; a.vy = Math.abs(a.vy) * 0.5; }
        if (a.y > H - a.r - 4) { a.y = H - a.r - 4; a.vy = -Math.abs(a.vy) * 0.5; }
        energy += Math.abs(a.vx) + Math.abs(a.vy);
      }
      return energy;
    }

    function paint(t) {
      for (var i = 0; i < bodies.length; i++) {
        var a = bodies[i];
        // A breath: the field is never quite still, even when it has settled.
        var bx = Math.sin(t * 0.0009 + a.ph) * 2.2, by = Math.cos(t * 0.0011 + a.ph * 1.4) * 2.2;
        a.el.style.transform = 'translate3d(' + (a.x - a.r + bx).toFixed(1) + 'px,' + (a.y - a.r + by).toFixed(1) + 'px,0)';
      }
    }

    function tick(now) {
      if (!running) { return; }
      var dt = Math.min(0.04, (now - (last || now)) / 1000);
      last = now;
      var e = step(dt, false);
      sleepFor = e < 0.4 && !pointer.on ? sleepFor + dt : 0;
      paint(now);
      requestAnimationFrame(tick);
    }

    function start() {
      if (running || !built || !onScreen || !physicsWanted() || document.hidden) { return; }
      running = true; last = 0; requestAnimationFrame(tick);
    }
    function stop() { running = false; }

    function enable() {
      if (!physicsWanted()) { disable(); return; }
      host.classList.add('is-physical');
      build();
      start();
    }
    function disable() {
      stop();
      host.classList.remove('is-physical');
      buttons.forEach(function (b) { b.style.width = b.style.height = b.style.transform = ''; b.style.removeProperty('--d'); });
      built = false;
    }

    field.addEventListener('pointermove', function (e) {
      if (!finePointer(e)) { return; }
      var r = field.getBoundingClientRect();
      pointer.x = e.clientX - r.left; pointer.y = e.clientY - r.top; pointer.on = true;
      start();
    }, { passive: true });
    field.addEventListener('pointerleave', function () { pointer.on = false; });
    window.addEventListener('resize', frame(function () {
      if (physicsWanted()) { enable(); } else if (built) { disable(); }
    }), { passive: true });
    document.addEventListener('visibilitychange', function () { if (document.hidden) { stop(); } else { start(); } });
    if (mq && mq.addEventListener) { mq.addEventListener('change', function () { if (reduced()) { disable(); } else { enable(); } }); }

    if ('IntersectionObserver' in window) {
      new IntersectionObserver(function (entries) {
        onScreen = entries[0].isIntersecting;
        if (onScreen) {
          host.classList.add('is-entered');
          start();
        } else { stop(); }
      }, { threshold: 0.15 }).observe(field);
    } else {
      host.classList.add('is-entered');
    }
    // Decided now, not when the field is first seen: the chips become bubbles
    // before anyone is looking, so nothing visibly rearranges.
    if (physicsWanted()) { enable(); }
    describe();
  }

  // -------------------------------------------------------------------------
  // The bakery: filters, search, FLIP
  // -------------------------------------------------------------------------

  var slots = [];

  function initBakery() {
    var bakery = $('[data-bakery]');
    if (!bakery) { return; }
    var grid = $('[data-bakery-grid]', bakery);
    var status = $('[data-bakery-status]', bakery);
    var empty = $('[data-bakery-empty]', bakery);
    var search = $('[data-bakery-search]', bakery);
    var chips = $$('[data-domain-filter]', bakery);
    slots = $$('[data-slot]', grid).map(function (slot) {
      var card = $('.loaf', slot);
      return {
        el: slot,
        card: card,
        slug: card.getAttribute('data-project'),
        domain: card.getAttribute('data-domain'),
        tags: ' ' + card.getAttribute('data-tags') + ' ',
        hay: (card.textContent || '').toLowerCase().replace(/\s+/g, ' ')
      };
    });
    var domainNames = {};
    chips.forEach(function (c) { domainNames[c.getAttribute('data-domain-filter')] = (c.firstChild.textContent || '').trim(); });
    var total = slots.length;

    function matches(s) {
      if (state.domain !== 'all' && s.domain !== state.domain) { return false; }
      if (state.tag && s.tags.indexOf(' ' + state.tag + ' ') < 0) { return false; }
      if (state.q) {
        var words = state.q.split(' ');
        for (var i = 0; i < words.length; i++) { if (words[i] && s.hay.indexOf(words[i]) < 0) { return false; } }
      }
      return true;
    }

    function rects() {
      var map = {};
      slots.forEach(function (s) { if (!s.el.hidden) { map[s.slug] = s.el.getBoundingClientRect(); } });
      return map;
    }

    var firstPaint = true;
    function draw() {
      chips.forEach(function (c) {
        var on = c.getAttribute('data-domain-filter') === state.domain;
        c.setAttribute('aria-pressed', on ? 'true' : 'false');
        c.classList.toggle('is-on', on);
      });
      if (search && search.value.toLowerCase().trim() !== state.q) { search.value = state.q; }

      var animate = !firstPaint && !reduced() && typeof Element.prototype.animate === 'function';
      var before = animate ? rects() : null;
      var shown = 0;
      slots.forEach(function (s) {
        var ok = matches(s);
        s.was = !s.el.hidden;
        s.el.hidden = !ok;
        if (ok) { shown++; }
      });
      if (animate) {
        slots.forEach(function (s) {
          if (s.el.hidden) { return; }
          var now = s.el.getBoundingClientRect();
          var was = before[s.slug];
          if (was) {
            var dx = was.left - now.left, dy = was.top - now.top;
            if (Math.abs(dx) > 1 || Math.abs(dy) > 1) {
              s.el.animate([{ transform: 'translate(' + dx + 'px,' + dy + 'px)' }, { transform: 'none' }],
                { duration: 520, easing: 'cubic-bezier(0.16,1,0.3,1)' });
            }
          } else {
            s.el.animate([{ opacity: 0, transform: 'scale(0.92) translateY(10px)' }, { opacity: 1, transform: 'none' }],
              { duration: 420, easing: 'cubic-bezier(0.16,1,0.3,1)' });
          }
        });
      }
      firstPaint = false;

      var parts = [];
      if (state.domain !== 'all') { parts.push(domainNames[state.domain]); }
      if (state.tag) { parts.push(tagNames[state.tag]); }
      if (state.q) { parts.push('“' + state.q + '”'); }
      var text = shown === total ? total + ' projects.' : shown + ' of ' + total + ' projects';
      if (parts.length && shown !== total) { text += ' · ' + parts.join(' · '); }
      if (status) { status.textContent = text; }
      if (empty) { empty.hidden = shown !== 0; }
    }

    chips.forEach(function (c) {
      c.addEventListener('click', function () { setState({ domain: c.getAttribute('data-domain-filter') }); });
    });
    if (search) {
      var timer = 0;
      search.addEventListener('input', function () {
        window.clearTimeout(timer);
        timer = window.setTimeout(function () { setState({ q: search.value.toLowerCase().replace(/\s+/g, ' ').trim() }); }, 120);
      });
    }
    var reset = $('[data-bakery-reset]', bakery);
    if (reset) { reset.addEventListener('click', function () { setState({ domain: 'all', tag: null, q: '' }); }); }

    // "/" jumps to the search from anywhere on the page, the way it does on
    // the rest of the site's search fields.
    document.addEventListener('keydown', function (e) {
      if (e.key !== '/' || e.ctrlKey || e.metaKey || e.altKey) { return; }
      var t = e.target;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) { return; }
      if (!search || (document.querySelector('dialog[open]'))) { return; }
      e.preventDefault();
      search.focus();
      search.scrollIntoView({ behavior: reduced() ? 'auto' : 'smooth', block: 'center' });
    });

    // A spotlight on the loaf under the pointer.
    grid.addEventListener('pointermove', function (e) {
      if (!finePointer(e)) { return; }
      var card = e.target.closest ? e.target.closest('.loaf') : null;
      if (!card) { return; }
      var r = card.getBoundingClientRect();
      card.style.setProperty('--mx', (e.clientX - r.left).toFixed(0) + 'px');
      card.style.setProperty('--my', (e.clientY - r.top).toFixed(0) + 'px');
    }, { passive: true });

    onState(draw);
    draw();
  }

  // -------------------------------------------------------------------------
  // The recipe card
  // -------------------------------------------------------------------------

  function initRecipe() {
    var dlg = $('[data-recipe]');
    if (!dlg) { return; }
    var card = $('[data-recipe-card]', dlg);
    var current = null;

    function loafFor(slug) { return document.querySelector('.loaf[data-project="' + slug + '"]'); }

    function visibleSlugs() {
      var list = slots.filter(function (s) { return !s.el.hidden; }).map(function (s) { return s.slug; });
      return list.length ? list : slots.map(function (s) { return s.slug; });
    }

    function fill(slug) {
      var loaf = loafFor(slug);
      if (!loaf) { return false; }
      current = slug;
      card.style.setProperty('--dom', loaf.style.getPropertyValue('--dom'));
      $('[data-recipe-domain]', dlg).textContent = $('.loaf__domain', loaf).textContent;
      $('[data-recipe-title]', dlg).textContent = $('.loaf__open', loaf).textContent;
      $('[data-recipe-pitch]', dlg).textContent = $('.loaf__pitch', loaf).textContent;

      var stack = $('[data-recipe-stack]', dlg);
      stack.innerHTML = '';
      $$('.loaf__all li', loaf).forEach(function (li) {
        var n = document.createElement('li'); n.textContent = li.textContent; stack.appendChild(n);
      });

      var steps = $('[data-recipe-steps]', dlg);
      steps.innerHTML = '';
      $$('.loaf__method li', loaf).forEach(function (li, i) {
        var n = document.createElement('li'); n.textContent = li.textContent; n.style.setProperty('--i', String(i)); steps.appendChild(n);
      });

      var crown = $('[data-recipe-crown]', dlg);
      crown.innerHTML = '';
      var src = $('.crown', loaf);
      if (src) { crown.appendChild(src.cloneNode(true)); }

      var scale = loaf.getAttribute('data-scale') || '';
      var cuts = ['', 'a sketch', 'a small loaf', 'a good loaf', 'a big loaf', 'a flagship'][parseInt(scale, 10)] || '';
      var meta = [];
      meta.push($('.loaf__kept', loaf).textContent.replace(/^Kept:\s*/, 'Kept ').replace(/\s+/g, ' ').trim());
      meta.push($('.loaf__vis', loaf).textContent);
      if ($('.loaf__ai', loaf)) { meta.push('Built in pair with AI coding agents'); }
      if (cuts) { meta.unshift(scale + (scale === '1' ? ' cut' : ' cuts') + ' — ' + cuts); }
      $('[data-recipe-meta]', dlg).innerHTML = meta.map(function (m) { return m.replace(/&/g, '&amp;').replace(/</g, '&lt;'); }).join('<br>');

      var lineage = $('.loaf__lineage', loaf);
      $('[data-recipe-lineage]', dlg).textContent = lineage ? lineage.textContent : '';

      var link = $('.loaf__link', loaf);
      var out = $('[data-recipe-link]', dlg);
      if (link) { out.href = link.href; out.textContent = link.textContent + ' ↗'; out.hidden = false; } else { out.hidden = true; out.removeAttribute('href'); }

      card.scrollTop = 0;
      return true;
    }

    function setHash(slug) {
      try {
        var base = window.location.pathname + window.location.search;
        history.replaceState(null, '', slug ? base + '#project=' + slug : base);
      } catch (e) { /* a sandboxed frame may refuse; the card still works */ }
    }

    function open(slug) {
      if (!fill(slug)) { return; }
      if (!dlg.open) {
        dlg.classList.remove('is-closing');
        dlg.showModal();
        doc.classList.add('about-modal-open');
      }
      setHash(slug);
    }

    function close() {
      if (!dlg.open) { return; }
      var done = function () {
        dlg.classList.remove('is-closing');
        dlg.close();
        doc.classList.remove('about-modal-open');
        setHash(null);
      };
      if (reduced()) { done(); return; }
      dlg.classList.add('is-closing');
      window.setTimeout(done, 190);
    }

    function go(delta) {
      var list = visibleSlugs();
      var i = list.indexOf(current);
      var next = list[(i + delta + list.length) % list.length];
      open(next);
    }

    // Any button that names a project opens it: the loaves, the evidence
    // chips under the title and the house rules, the dots on the timeline.
    root.addEventListener('click', function (e) {
      var b = e.target.closest ? e.target.closest('[data-open-project]') : null;
      if (!b) { return; }
      var slug = b.getAttribute('data-open-project');
      if (hasDialog) { open(slug); return; }
      var loaf = loafFor(slug);
      if (loaf) { loaf.scrollIntoView({ behavior: 'auto', block: 'center' }); }
    });

    if (!hasDialog) { return; }

    $('[data-recipe-close]', dlg).addEventListener('click', close);
    $('[data-recipe-prev]', dlg).addEventListener('click', function () { go(-1); });
    $('[data-recipe-next]', dlg).addEventListener('click', function () { go(1); });
    dlg.addEventListener('click', function (e) { if (e.target === dlg) { close(); } });
    dlg.addEventListener('cancel', function (e) { e.preventDefault(); close(); });
    dlg.addEventListener('keydown', function (e) {
      if (e.key === 'ArrowLeft') { e.preventDefault(); go(-1); }
      if (e.key === 'ArrowRight') { e.preventDefault(); go(1); }
    });
    dlg.addEventListener('close', function () { doc.classList.remove('about-modal-open'); });

    function fromHash() {
      var m = /^#project=([a-z0-9-]+)$/.exec(window.location.hash);
      if (m && loafFor(m[1])) { open(m[1]); } else if (!m && dlg.open) { close(); }
    }
    window.addEventListener('hashchange', fromHash);
    if (/^#project=/.test(window.location.hash)) { fromHash(); }
  }

  // -------------------------------------------------------------------------
  // The rise: a timeline you can play
  // -------------------------------------------------------------------------

  function initRise() {
    var rise = $('[data-rise]');
    if (!rise) { return; }
    var track = $('[data-rise-track]', rise);
    var scroller = $('[data-rise-scroll]', rise);
    var range = $('[data-rise-range]', rise);
    var play = $('[data-rise-play]', rise);
    var readout = $('[data-rise-readout]', rise);
    var dotSlots = $$('[data-rise-slot]', rise).map(function (el) {
      var slug = el.getAttribute('data-slug');
      var loaf = document.querySelector('.loaf[data-project="' + slug + '"]');
      var tags = loaf ? loaf.getAttribute('data-tags').split(' ') : [];
      return { el: el, x: parseFloat(el.getAttribute('data-x')), tags: tags, born: true };
    });
    var eras = $$('[data-era]', rise).map(function (el) {
      return { el: el, from: parseFloat(el.getAttribute('data-x-from')), to: parseFloat(el.getAttribute('data-x-to')) };
    });
    var warp = (rise.getAttribute('data-warp') || '').split(',').map(parseFloat);
    var firstYear = parseInt(rise.getAttribute('data-first-year'), 10);
    var months = warp.length - 1;
    var MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    var total = dotSlots.length;
    if (!track || !range || !play || months < 1) { return; }

    // The playhead.
    var now = document.createElement('div');
    now.className = 'rise__now';
    now.setAttribute('aria-hidden', 'true');
    track.appendChild(now);

    function monthAt(pct) {
      var m = 0;
      for (var i = 0; i < months; i++) { if (warp[i] <= pct) { m = i; } }
      return m;
    }

    function render(frac, quiet) {
      var pct = frac * 100;
      track.style.setProperty('--now', pct.toFixed(2) + '%');
      range.style.setProperty('--p', pct.toFixed(1) + '%');
      var out = 0, langs = {};
      dotSlots.forEach(function (d) {
        var born = d.x <= pct + 0.0001;
        if (born !== d.born) {
          d.el.classList.toggle('is-unborn', !born);
          d.el.classList.toggle('is-born', born && !quiet);
          if (born && !quiet) {
            window.setTimeout(function () { d.el.classList.remove('is-born'); }, 560);
          }
          d.born = born;
        }
        if (born) {
          out++;
          d.tags.forEach(function (t) { if (tagKinds[t] === 'language') { langs[t] = true; } });
        }
      });
      eras.forEach(function (e) {
        e.el.classList.toggle('is-now', pct >= e.from && pct < e.to + 0.0001);
        e.el.classList.toggle('is-past', pct >= e.to + 0.0001);
      });
      var nl = Object.keys(langs).length;
      if (out === total) {
        readout.textContent = 'All ' + total + ' loaves are out.';
      } else {
        var m = monthAt(pct);
        var label = MONTHS[m % 12] + ' ' + (firstYear + Math.floor(m / 12));
        readout.innerHTML = '';
        var b = document.createElement('b');
        b.textContent = label;
        readout.appendChild(b);
        readout.appendChild(document.createTextNode(out === 0
          ? ' · the oven is still cold.'
          : ' · ' + out + (out === 1 ? ' loaf is' : ' loaves are') + ' out, in ' + nl + (nl === 1 ? ' language.' : ' languages.')));
      }
    }

    function frac() { return parseInt(range.value, 10) / 1000; }
    range.addEventListener('input', function () { stop(); render(frac(), false); });

    var raf = 0, t0 = 0, from = 0, duration = 0;
    function stop() {
      if (raf) { cancelAnimationFrame(raf); raf = 0; }
      play.textContent = frac() >= 1 ? 'Replay the years' : 'Play the years';
    }

    function run(startFrac, ms) {
      stop();
      from = startFrac; duration = ms; t0 = 0;
      play.textContent = 'Pause';
      var tick = function (t) {
        if (!t0) { t0 = t; }
        var k = clamp((t - t0) / duration, 0, 1);
        var f = lerp(from, 1, k);
        range.value = String(Math.round(f * 1000));
        render(f, false);
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
      if (f >= 1) { f = 0; range.value = '0'; render(0, true); }
      run(f, Math.max(2500, (1 - f) * 11000));
    });

    // Start on the finished shelf; the first time it is seen, empty it and
    // fill it once, so the point of the thing is shown rather than described.
    render(1, true);
    if (scroller) { scroller.scrollLeft = scroller.scrollWidth; }
    if (!reduced() && 'IntersectionObserver' in window) {
      var seen = new IntersectionObserver(function (entries) {
        if (!entries[0].isIntersecting) { return; }
        seen.disconnect();
        range.value = '0';
        render(0, true);
        if (scroller) { scroller.scrollLeft = 0; }
        window.setTimeout(function () { run(0, 9000); }, 500);
      }, { threshold: 0.55 });
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
  }

  // -------------------------------------------------------------------------

  function boot() {
    goLive();
    initFrame();
    initHero();
    initDust();
    initCounters();
    initReveal();
    initAnatomy();
    initBakery();
    initPantry();
    initRecipe();
    initRise();
    initPrint();
  }

  if (document.readyState === 'loading') { document.addEventListener('DOMContentLoaded', boot); } else { boot(); }
})();
