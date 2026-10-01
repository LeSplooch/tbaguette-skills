"""The About page's content: who TBaguette is, as a résumé, from the record.

Everything the page says lives here and nowhere else; about_page.py only lays
it out and docs/assets/about.js only animates what it laid out. That split is
what keeps a claim from being made twice in two places and drifting.

Where it came from. Each project below was read in its own repository, in its
code and its history, not from a description: the first and last dates are
real commit dates (an import or a relicensing commit is not counted), and a
thing is only listed under "method" if the code does it. Three rules decided
what made the shelf:

  * Original work only. A repository that is a copy of somebody else's
    open-source project, or an unmodified starter template, is not a project
    of this person's and is not listed, however it is named.
  * Nothing the code treats as private. No credentials, hostnames or
    addresses, no personal data about anyone, no trading figures, and none of
    the internals of licence, expiry or anti-cheat logic. A description may
    say what a program is for; it may not say how to get around anything.
  * Honest about size. A prototype is labelled one. `scale` runs 1 (a sketch)
    to 5 (a flagship) and is drawn on the loaf as that many cuts.

scripts/test_about_profile.py holds the rules a typo could quietly break:
unique slugs, known domains and tags, well-formed dates, and no string in this
file that looks like a secret, an address or an email.

Dates are "YYYY-MM". `vis` is "public" or "private"; only a public project
carries a `url`.
"""

from __future__ import annotations

NAME = "TBaguette"
HANDLE = "LeSplooch"
GITHUB_URL = "https://github.com/LeSplooch"
SITE_URL = "https://lesplooch.github.io/tbaguette-skills/"

# The first original repository on the record. The account is older; the
# record of what it made starts here.
SINCE = 2019

# The title, coined rather than borrowed: a master baker, in the profession's
# own French, of the software that keeps working while nobody is looking.
TITLE_FR = "Maître"
TITLE_ROLE = "Boulanger"
TITLE_OF = "of"
TITLE_FIELD = "Autonomous Software"

TAGLINE = (
    "I make software that keeps working while nobody is looking: bots, "
    "agents, watchers, phone apps, and the tools that bake them."
)

SUMMARY = (
    "Since 2019 the record is one long loop: find something that annoys, "
    "bake the first loaf quickly, then make it sturdy. It began as Rust "
    "tools for Windows and Discord, grew through C# and Kotlin into desktop "
    "companions and Android apps, and has lately turned to agents, to a "
    "Rust-to-WebAssembly image pipeline, and to the skills that steer an "
    "agent in the first place.",
    "What repeats is everything after the first version: the installer, the "
    "self-updater, the signing step, the privacy policy, the check that "
    "refuses to build when something is off. Most recent work is made in "
    "pair with AI coding agents, and the commit history says so.",
)

# The three words of the title that are claims, and the work that answers
# each. `evidence` names projects by slug; the page turns them into buttons.
TITLE_CLAIMS = (
    ("Maître",
     "Mastery in the old guild sense: not one lane but the whole shop. "
     "{Years} years on the record, {languages} languages, {surfaces} "
     "surfaces, and the same habit of finishing in all of them.",
     ("watchcord", "translatetip", "rotnomore", "tkeyboard", "photoshield", "flappy-bunny")),
    ("Boulanger",
     "A baker takes flour and water to something people can eat. Here that "
     "means the part after the code runs: the installer, the updater, the "
     "release script, the feed the app polls, the privacy page.",
     ("tbaguettemess", "hotaru-house", "deckhand", "playctl")),
    ("Autonomous Software",
     "Programs that act without being asked twice: a bot that answers a "
     "voice channel, tools that keep closing a process, a watcher that "
     "waits out maintenance, and agents that decide, always inside a fence.",
     ("badass-bot", "anti-process", "db-check", "vireo", "hermes-pocket", "playctl")),
)

# What the work has in common, each backed by projects further down.
HOUSE_RULES = (
    ("The model may only subtract",
     "In the trading terminals a language model can veto a decision or "
     "shrink it, never enlarge it, and a deterministic gate has the last "
     "word. The Play Store tool lets an agent act unattended on the "
     "lowest-stakes track only, in code, with tests that say so.",
     ("vireo", "kairos", "soldream", "playctl")),
    ("Ship the installer too",
     "An MSI and an updater that finds its own releases, a release script "
     "that refuses a bad build, a feed the app polls. A program is not done "
     "when it runs; it is done when it can arrive and leave.",
     ("tbaguettemess", "hotaru-house", "deckhand", "playctl")),
    ("Procedural before pre-made",
     "One app draws every sprite and synthesises every sound at runtime, "
     "and its release script fails if an image or an audio file sneaks in. "
     "A browser game scores twenty worlds with synthesised music. Maths "
     "first, assets last.",
     ("hotaru-house", "flappy-bunny")),
    ("Say it in a tiny language",
     "A clan-tag animator with its own four-word script language; a "
     "terminal message player with eighteen directives. When a tool needs "
     "scripting, give it a grammar.",
     ("clan-tag-gif", "animessage")),
    ("Fail loudly",
     "A build that stops on a malformed changelog, a site checked by script "
     "before it ships, a screen that fails closed on a missing number, a "
     "dry-run flag on anything that touches the filesystem. Quiet failure "
     "is the expensive kind.",
     ("tbaguette-atelier", "deckhand", "vireo", "animessage")),
)

# language / platform / craft: the three kinds of ingredient on the pantry shelf.
TAGS = {
    "rust": ("Rust", "language"),
    "kotlin": ("Kotlin", "language"),
    "csharp": ("C#", "language"),
    "python": ("Python", "language"),
    "typescript": ("TypeScript", "language"),
    "javascript": ("JavaScript", "language"),
    "android": ("Android", "platform"),
    "windows": ("Windows", "platform"),
    "browser": ("Browser", "platform"),
    "discord": ("Discord", "platform"),
    "terminal": ("Terminal", "platform"),
    "compose": ("Jetpack Compose", "craft"),
    "tauri": ("Tauri", "craft"),
    "react": ("React", "craft"),
    "wasm": ("WebAssembly", "craft"),
    "crypto": ("Encryption & signing", "craft"),
    "procedural": ("Procedural audio & art", "craft"),
    "dsl": ("Little languages", "craft"),
    "updaters": ("Installers & updaters", "craft"),
    "automation": ("Automation", "craft"),
    "agents": ("AI agents", "craft"),
    "guardrails": ("Guardrails for models", "craft"),
    "markets": ("Market data", "craft"),
    "scroll": ("Scroll-driven motion", "craft"),
}

# `color` indexes the library's twelve family colours (--graph-cat-N in
# styles.css), which are validated in both themes; reusing them keeps this
# page's data colours as trustworthy as the Crumb's.
DOMAINS = (
    ("agents", "Agents & AI", 4,
     "Software that decides and acts, and the fences and instructions around it."),
    ("phone", "Phones", 2,
     "Android apps, from a keyboard to a private world for two."),
    ("desktop", "Desktop", 3,
     "Windows companions and watchers that run beside something else."),
    ("bots", "Bots", 1,
     "Programs that live in a chat and answer for it."),
    ("web", "Web", 8,
     "Pages that move, and the feeds behind them."),
    ("terminal", "Terminal", 5,
     "Games, tools and little languages that live in a console."),
)

# One entry per project: slug, name, domain, pitch, tags, method, first, last,
# scale, vis, and optionally url (public only), ai (built in pair with AI
# coding agents, per its own commit history), lineage (a note shown on its
# card), repos (what it spans, shown on its recipe).
PROJECTS = (
    {
        "slug": "tbaguette-atelier",
        "name": "TBaguette’s Atelier",
        "domain": "agents",
        "pitch": "A hundred and one skills that tell an AI coding agent how "
                 "to work, and the site, graph and build that publish them.",
        "tags": ("python", "javascript", "agents", "automation"),
        "method": (
            "101 skills in 12 families, each project-, stack- and "
            "language-agnostic, installable across a dozen agent harnesses.",
            "The showcase site is generated: a build reads every skill and "
            "writes the pages, the search and the cross-reference graph.",
            "The graph is a canvas app that draws every place one skill "
            "names another, and a check refuses to publish it if it no "
            "longer matches the skills.",
            "A changelog parser stops the build when its shape or order "
            "breaks, and a test suite gates every change.",
        ),
        "first": "2026-09", "last": "2026-10", "scale": 5, "vis": "public",
        "url": "https://github.com/LeSplooch/tbaguette-skills",
        "ai": True,
        "lineage": "This site is part of it, and so is the page you are on.",
    },
    {
        "slug": "vireo",
        "name": "Vireo",
        "domain": "agents",
        "pitch": "A desktop terminal that paper-trades on-chain tokens, where "
                 "a language model may veto a trade but never make one.",
        "tags": ("rust", "typescript", "tauri", "agents", "guardrails", "markets"),
        "method": (
            "A Rust workspace of eight crates under a Tauri desktop shell "
            "and a plain TypeScript interface.",
            "Candidates pass a fourteen-rule screen that fails closed when "
            "a metric is missing or not a number, then a scoring ensemble, "
            "an optional model review, sizing and portfolio limits.",
            "Real trading is structurally impossible: only a paper executor "
            "is wired in, a dependency policy and a test ban the signing "
            "libraries, and arming anything takes four separate gates.",
            "It brings its own market simulator and a calibration harness "
            "that compares runs on paired seeds.",
        ),
        "first": "2026-09", "last": "2026-09", "scale": 5, "vis": "private",
        "ai": True,
    },
    {
        "slug": "soldream",
        "name": "Soldream",
        "domain": "agents",
        "pitch": "A desktop trading terminal where a council of model "
                 "personas proposes and a deterministic risk module decides.",
        "tags": ("rust", "typescript", "tauri", "react", "agents", "guardrails",
                 "markets", "crypto"),
        "method": (
            "A Rust and Tauri core with a React interface; every entry a "
            "model proposes must pass the risk module before anything is "
            "sent.",
            "Designed so a sent trade is never sent twice, and so that keys "
            "live in an encrypted vault backed by the operating system’s "
            "keyring.",
            "Strategies are drafted by a model and promoted from paper to a "
            "trial to live only on a statistical lower bound; so far all of "
            "them are still in dry-run.",
            "Knowledge can sync, opt-in and signed, through a small relay "
            "written in Rust and hosted at home.",
        ),
        "first": "2026-07", "last": "2026-09", "scale": 5, "vis": "private",
        "ai": True,
    },
    {
        "slug": "kairos",
        "name": "Kairos",
        "domain": "agents",
        "pitch": "A desktop agent that scans a market-data feed and decides "
                 "with a local model, inside hard risk gates, paper first.",
        "tags": ("kotlin", "compose", "agents", "guardrails", "markets", "crypto"),
        "method": (
            "Kotlin and Compose Desktop, in five Gradle modules.",
            "A hand-written API client signs every request and was checked "
            "against the vendor’s own reference signer; a leaky bucket and "
            "a global cooldown keep it polite.",
            "A local model answers in a fixed schema and can only subtract: "
            "veto, or shrink the size.",
            "Going live needs three independent keys; there is a kill "
            "switch and a daily-loss breaker.",
        ),
        "first": "2026-08", "last": "2026-08", "scale": 3, "vis": "private",
        "ai": True,
    },
    {
        "slug": "nexus-agent",
        "name": "Nexus",
        "domain": "agents",
        "pitch": "A multi-user AI chat and agent web app, streaming from a "
                 "Rust server.",
        "tags": ("rust", "typescript", "react", "agents", "browser"),
        "method": (
            "An Axum and SQLite backend streams answers over server-sent "
            "events, with three intelligence modes mapped to reasoning "
            "effort.",
            "A React 19 front end of more than sixty components, in more "
            "than fifteen languages.",
            "Accounts with email and social sign-in, role-based access, "
            "per-address rate limits and credits billing.",
        ),
        "first": "2026-06", "last": "2026-07", "scale": 4, "vis": "private",
        "ai": True,
    },
    {
        "slug": "hermes-pocket",
        "name": "Hermes Pocket",
        "domain": "agents",
        "pitch": "An Android app that watches and steers an AI agent running "
                 "on your computer, over an encrypted relay.",
        "tags": ("kotlin", "rust", "python", "android", "compose", "agents", "crypto"),
        "method": (
            "Chat with streamed answers, tool traces and approvals; a run "
            "inbox; a workspace with files and a terminal; and screens for "
            "automations and skills.",
            "Records are sealed with AES-256-GCM across a two-party relay "
            "that was reused from another of these projects, with its "
            "provenance checksummed.",
            "The host-side bridge adds the agent’s API key itself, so the "
            "phone never holds it; pairing material sits in the Android "
            "keystore.",
        ),
        "first": "2026-09", "last": "2026-09", "scale": 3, "vis": "private",
        "lineage": "Its relay is Hotaru House’s.",
    },
    {
        "slug": "playctl",
        "name": "playctl",
        "domain": "agents",
        "pitch": "A command-line tool that publishes Android apps to Google "
                 "Play, with limits on what an agent may do unattended "
                 "enforced in code.",
        "tags": ("python", "android", "automation", "agents", "guardrails", "updaters"),
        "method": (
            "Four tiers of action: automatic, confirm, refresh and blocked. "
            "Only the internal track runs unattended; unknown actions ask; "
            "blocked ones cannot be confirmed, and tests assert it.",
            "Each edit is a transaction: validate before commit, discard on "
            "error, never retry a commit.",
            "Version codes derive from the clock, so build age can be "
            "computed, and an unattended refresh may only re-upload the "
            "commit already live.",
        ),
        "first": "2026-08", "last": "2026-08", "scale": 3, "vis": "private",
        "ai": True,
    },
    {
        "slug": "tkeyboard",
        "name": "TKeyboard",
        "domain": "phone",
        "pitch": "An Android keyboard with eleven animated skins, glide "
                 "typing, and a trick for typing on a computer whose layout "
                 "isn’t your phone’s.",
        "tags": ("kotlin", "android", "compose"),
        "method": (
            "Glide typing resamples the finger’s path and every candidate "
            "word’s key path to forty points, then ranks by mean distance, "
            "endpoint error, length mismatch and word frequency.",
            "A layout transmitter turns each character into the key presses "
            "that type it on the host’s layout, handling AltGr and dead "
            "keys, so a remote desktop receives what was meant.",
            "Eleven skins, twenty-four bundled dictionaries and an emoji "
            "set ship inside the app, built with Jetpack Compose.",
        ),
        "first": "2026-09", "last": "2026-09", "scale": 4, "vis": "private",
        "ai": True,
    },
    {
        "slug": "hotaru-house",
        "name": "Hotaru House",
        "domain": "phone",
        "pitch": "A private, two-person pixel-art world for Android, with "
                 "end-to-end encrypted sync over a relay written and hosted "
                 "by hand.",
        "tags": ("kotlin", "rust", "android", "crypto", "procedural", "updaters"),
        "method": (
            "Every sprite and every sound is generated at runtime; the "
            "release script fails if the APK contains an image, an audio "
            "file or a font.",
            "The synthesiser is written from scratch: band-limited "
            "oscillators, plucked strings, bells, envelopes, a resonant "
            "filter and a stereo room.",
            "Records are sealed with AES-256-GCM under a fresh nonce each; "
            "a Rust server builds four binaries: a relay, an update server, "
            "an append-only archive that only ever sees ciphertext, and an "
            "admin tool.",
            "Sync is offline-first with an outbox, and the two phones find "
            "each other on the local network.",
        ),
        "first": "2026-08", "last": "2026-09", "scale": 5, "vis": "private",
        "ai": True,
        "repos": "the app and its release repository",
    },
    {
        "slug": "photoshield",
        "name": "PhotoShield",
        "domain": "phone",
        "pitch": "A mobile app that stamps photos with invisible marks, with "
                 "the pixel work done in Rust compiled to WebAssembly.",
        "tags": ("typescript", "rust", "wasm", "android"),
        "method": (
            "A Rust module compiled to WebAssembly runs inside a hidden "
            "WebView and embeds invisible marks: a rotation-tolerant "
            "luminance pattern, a transform-domain mark, low-bit hiding, "
            "and a metadata notice asking not to be scraped.",
            "The app is React Native on Expo with a credits economy, "
            "in-app purchases, rewarded ads, Firebase and thirty-two "
            "languages.",
            "A background monitor watches the media library and notifies "
            "when something new needs marking.",
        ),
        "first": "2025-12", "last": "2026-06", "scale": 5, "vis": "private",
        "ai": True,
        "lineage": "Its privacy policy names Day Dream Corp as publisher.",
        "repos": "the app, and its privacy-policy page in a second repository",
    },
    {
        "slug": "audiogem",
        "name": "AudioGem",
        "domain": "phone",
        "pitch": "An Android utility that makes Bluetooth headphones use the "
                 "audio codec and quality you picked, and estimates how "
                 "loud a day of it was.",
        "tags": ("kotlin", "android", "compose"),
        "method": (
            "Reaches Android’s hidden Bluetooth codec API through "
            "reflection, with fallbacks for how the framework’s "
            "constructors changed across versions.",
            "Sets codec, sample rate, bit depth and LDAC quality, and "
            "keeps them set from a foreground service.",
            "A listening-dose estimate compares volume and time against a "
            "safe-listening table.",
        ),
        "first": "2026-07", "last": "2026-07", "scale": 2, "vis": "private",
    },
    {
        "slug": "rotnomore",
        "name": "Rot No More",
        "domain": "phone",
        "pitch": "An Android app that scans a barcode and keeps track of "
                 "when the food behind it will go off.",
        "tags": ("csharp", "android"),
        "method": (
            "Built with .NET MAUI and ReactiveUI, in the MVVM style.",
            "A camera scan reads the barcode and looks the product up; "
            "each product is kept as a small file of its own.",
        ),
        "first": "2023-02", "last": "2023-05", "scale": 2, "vis": "private",
    },
    {
        "slug": "tbaguettemess",
        "name": "TBaguette’s Mess",
        "domain": "desktop",
        "pitch": "A Windows companion for a multiplayer shooter: a live "
                 "server browser, an overlay, tray notifications and its own "
                 "installer.",
        "tags": ("kotlin", "windows", "compose", "updaters", "discord"),
        "method": (
            "A Compose Multiplatform desktop app, packaged as an MSI.",
            "The server browser polls a community stats feed about every ten "
            "seconds, pings every server at once, and sorts them into "
            "tiers.",
            "Windows integration through JNA: a click-through overlay "
            "window, game-focus detection every hundred milliseconds, and "
            "tray notifications.",
            "Updates arrive through a Discord bot that finds the newest "
            "installer, compares semantic versions, downloads it and runs it.",
        ),
        "first": "2024-10", "last": "2026-07", "scale": 4, "vis": "private",
        "lineage": "A rewrite of the 2023 C# sketch below it.",
    },
    {
        "slug": "mess-prototypes",
        "name": "Mess prototypes",
        "domain": "desktop",
        "pitch": "Two 2023 C# sketches: the first draft of the companion app, "
                 "and a small installer wizard to go with it.",
        "tags": ("csharp", "windows", "updaters"),
        "method": (
            "A WinForms installer with a folder picker that switches "
            "between install and update, plus a confirmed uninstall.",
            "The first draft of the companion app, before the rewrite in "
            "Kotlin.",
        ),
        "first": "2023-06", "last": "2023-06", "scale": 1, "vis": "private",
        "lineage": "Sketches, honestly: the Kotlin rewrite is the real thing.",
        "repos": "two small repositories",
    },
    {
        "slug": "clan-tag-gif",
        "name": "Clan Tag GIF",
        "domain": "desktop",
        "pitch": "A Windows tool that animates a player’s clan tag in a "
                 "game, driven by a script language it parses itself.",
        "tags": ("rust", "windows", "dsl", "automation"),
        "method": (
            "The script language has four words: Show, Wait, Loop and "
            "KeyboardLayout. The parser compiles a script in a few "
            "milliseconds and warns about code after an endless loop.",
            "It types into the game’s console through synthesised "
            "keystrokes, with QWERTY and AZERTY layouts, and pauses "
            "whenever the game loses focus.",
            "The console-driving code was split into a small library of "
            "its own, which also finds the game and launches it.",
        ),
        "first": "2021-07", "last": "2022-03", "scale": 3, "vis": "private",
        "repos": "the tool and the helper library split out of it",
    },
    {
        "slug": "watchcord",
        "name": "Watchcord",
        "domain": "desktop",
        "pitch": "A Windows tool, with a French console, that makes Discord "
                 "use less of the processor so the game gets more.",
        "tags": ("rust", "windows", "automation"),
        "method": (
            "Every two and a half seconds it finds Discord’s processes and "
            "lowers their priority.",
            "It limits them to about a third of the logical processors, "
            "building the affinity mask itself.",
            "A release exists, and a self-updater was written, then set "
            "aside.",
        ),
        "first": "2019-07", "last": "2019-09", "scale": 2, "vis": "private",
    },
    {
        "slug": "anti-process",
        "name": "Anti Process",
        "domain": "desktop",
        "pitch": "A Windows background tool that keeps closing the programs "
                 "listed in a text file.",
        "tags": ("rust", "windows", "automation"),
        "method": (
            "Rereads its list every second, so an edit takes effect "
            "without a restart.",
            "Reports each closure in a native dialog from a worker thread "
            "and in a log.",
        ),
        "first": "2021-12", "last": "2021-12", "scale": 1, "vis": "private",
    },
    {
        "slug": "fragmov",
        "name": "FragMov",
        "domain": "desktop",
        "pitch": "A prototype that was meant to cut gaming highlights into "
                 "a montage on its own.",
        "tags": ("rust", "windows"),
        "method": (
            "A five-screen wizard in the iced GUI toolkit with drag-and-drop "
            "input; two screens are built and three are stubs.",
            "Reads MP4 samples into frames, and validates a long list of "
            "encoding options on the command line.",
        ),
        "first": "2022-02", "last": "2022-04", "scale": 1, "vis": "private",
        "lineage": "A prototype, honestly: the montage logic was never written.",
    },
    {
        "slug": "translatetip",
        "name": "TranslateTip",
        "domain": "desktop",
        "pitch": "A pop-up translator for the desktop, opened by a global "
                 "hotkey.",
        "tags": ("python", "windows"),
        "method": (
            "A background thread listens for a global hotkey and opens a "
            "small box for what you want translated.",
            "Hotkeys live in a settings file that reloads while it runs.",
        ),
        "first": "2020-05", "last": "2020-05", "scale": 1, "vis": "private",
        "lineage": "The oldest loaf on the shelf, and a prototype.",
    },
    {
        "slug": "badass-bot",
        "name": "Badass Bot",
        "domain": "bots",
        "pitch": "A Discord bot, in Rust, that plays each member’s chosen "
                 "six-second clip when they join or leave a voice channel.",
        "tags": ("rust", "discord"),
        "method": (
            "Voice-state events feed a per-server queue that a background "
            "loop drains, streaming each clip and seeking to a saved start.",
            "A member sets their clip in a direct-message dialogue with "
            "timeouts and cancel; only a link and an offset are stored.",
            "State is saved every eight seconds by writing a new file and "
            "renaming it over the old, and the process restarts itself "
            "after an error.",
            "Two hundred and fifty commits across eleven months.",
        ),
        "first": "2021-05", "last": "2022-04", "scale": 4, "vis": "private",
    },
    {
        "slug": "moderation-suite",
        "name": "Moderation suite",
        "domain": "bots",
        "pitch": "A desktop admin app, Discord bot and member portal for "
                 "running a gaming community’s moderation by council vote.",
        "tags": ("rust", "typescript", "tauri", "react", "discord", "wasm"),
        "method": (
            "A Rust and Tauri backend of some forty modules with a React "
            "interface, and a member site written in Rust compiled to "
            "WebAssembly.",
            "A hand-written Discord gateway and REST client, including "
            "tunnelling, rather than a library.",
            "Votes survive restarts and are closed by a sweeper at their "
            "deadlines; members carry an Elo-style rating.",
            "On the member site only one open tab refreshes the dashboard, "
            "coordinated with Web Locks and a broadcast channel.",
        ),
        "first": "2026-08", "last": "2026-09", "scale": 5, "vis": "private",
        "ai": True,
        "repos": "the app and the built site it publishes",
    },
    {
        "slug": "flappy-bunny",
        "name": "Flappy Bunny",
        "domain": "web",
        "pitch": "A one-file browser game with twenty worlds, eight power-up "
                 "fruits and a leaderboard, and not one image or audio file.",
        "tags": ("javascript", "browser", "procedural"),
        "method": (
            "The bunny is drawn procedurally with squash and stretch; there "
            "are no sprites.",
            "Each of the twenty worlds has its own palette and its own "
            "synthesised Web Audio soundtrack, in its own tempo.",
            "The loop is fixed-timestep at sixty hertz, so physics doesn’t "
            "care how fast the screen refreshes.",
            "Scores are kept in the browser and on a dependency-free Node "
            "server’s top-ten board.",
        ),
        "first": "2026-07", "last": "2026-07", "scale": 3, "vis": "private",
    },
    {
        "slug": "deckhand",
        "name": "Deckhand site & feeds",
        "domain": "web",
        "pitch": "The animated launch page for a phone-as-control-surface "
                 "app, with the update feed and notice feed it serves.",
        "tags": ("javascript", "browser", "scroll", "updaters"),
        "method": (
            "A chapter engine pins and scrubs scroll timelines, and falls "
            "back to calm static layouts for readers who ask for less "
            "motion and for short screens.",
            "The page renders the app’s own control decks, fourteen kinds "
            "of control you can operate right there.",
            "It doubles as the desktop updater’s manifest, with hashes and "
            "release links, and as the feed the phone polls for notices.",
            "A script gates every publish: anchors, external hosts, inline "
            "handlers, and files that must not change.",
        ),
        "first": "2026-08", "last": "2026-09", "scale": 4, "vis": "public",
        "url": "https://daydreamcorp.github.io/deckhand-site/",
        "lineage": "Built for Day Dream Corp’s Deckhand; the app is in closed testing.",
        "repos": "the site and its privacy-policy mirror",
    },
    {
        "slug": "animessage",
        "name": "Animessage",
        "domain": "terminal",
        "pitch": "A terminal player for animated, scripted messages, with "
                 "its own small language for timing, sound and pictures.",
        "tags": ("rust", "terminal", "dsl"),
        "method": (
            "Eighteen directives, from typing a line one character at a "
            "time to jumping, replacing text, playing audio and opening a "
            "link, with a confirmation before anything is opened.",
            "Images are drawn in the terminal using the Kitty and iTerm "
            "protocols.",
            "A dry-run flag and a debug flag make a script safe to "
            "rehearse, and loops are bounded.",
        ),
        "first": "2021-11", "last": "2022-05", "scale": 3, "vis": "private",
    },
    {
        "slug": "db-check",
        "name": "DB Check",
        "domain": "terminal",
        "pitch": "A console tool that waits out a game’s maintenance and "
                 "launches the game the moment it is back.",
        "tags": ("rust", "terminal", "automation"),
        "method": (
            "Polls a public page every ten seconds with retries, and "
            "decides with four separate checks whether the game is up.",
            "Finds Steam on any drive and launches the game, and refuses "
            "to run from the Desktop.",
        ),
        "first": "2021-10", "last": "2021-11", "scale": 2, "vis": "private",
    },
    {
        "slug": "calendart",
        "name": "Calendart",
        "domain": "terminal",
        "pitch": "A terminal speed game: a date appears and you pick it on a "
                 "calendar as fast as you can.",
        "tags": ("rust", "terminal"),
        "method": (
            "Twenty-five fixed dates from easy to extreme, or a random mode "
            "of five to ten billion.",
            "Every level is timed; a wrong date repeats it.",
        ),
        "first": "2021-11", "last": "2022-01", "scale": 1, "vis": "private",
    },
)

# The chapters of the long view, each a stretch of time named for what was
# being baked in it. `from` and `to` are inclusive months.
ERAS = (
    ("2019-07", "2020-12", "First proofs",
     "Rust tools for Windows and a desktop translator: small, practical, "
     "and finished."),
    ("2021-01", "2022-12", "The Rust workshop",
     "A Discord bot of two hundred and fifty commits, a script language for "
     "a game tool, a terminal player for animated messages."),
    ("2023-01", "2023-12", "Across the platforms",
     "C# on Android and on Windows: a barcode food tracker, and the first "
     "sketches of a companion app."),
    ("2024-01", "2025-11", "Kotlin takes over",
     "The companion app, rewritten in Kotlin and Compose, shipped with an "
     "installer and an updater of its own."),
    ("2025-12", "2026-12", "Agents in the kitchen",
     "Phone apps, a Rust-to-WebAssembly pipeline, a procedural world, a "
     "keyboard, trading terminals with fences, and a library of skills for "
     "the agents that now bake alongside."),
)


def domain_by_slug() -> dict[str, tuple]:
    return {d[0]: d for d in DOMAINS}


def project_by_slug() -> dict[str, dict]:
    return {p["slug"]: p for p in PROJECTS}


def tag_counts() -> dict[str, int]:
    """How many projects use each tag: the weight of its bubble."""
    counts: dict[str, int] = {}
    for project in PROJECTS:
        for tag in project["tags"]:
            counts[tag] = counts.get(tag, 0) + 1
    return counts


def stats() -> dict[str, int]:
    """What the counters count: derived from the shelf, never typed, so
    adding a project cannot leave the numbers behind. Years are whole years
    between the earliest first commit and the latest last one."""
    kinds = {}
    for tag in {t for p in PROJECTS for t in p["tags"]}:
        kinds.setdefault(TAGS[tag][1], set()).add(tag)
    first = min(p["first"] for p in PROJECTS)
    latest = max(p["last"] for p in PROJECTS)
    months = (int(latest[:4]) - int(first[:4])) * 12 + int(latest[5:]) - int(first[5:])
    return {
        "projects": len(PROJECTS),
        "languages": len(kinds.get("language", ())),
        "surfaces": len(kinds.get("platform", ())),
        "years": months // 12,
    }
