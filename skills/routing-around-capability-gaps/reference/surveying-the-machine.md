# Surveying the machine

Everything below is a **hint to test**, never an inventory to trust. The agent-CLI
population turns over monthly — well over a hundred are in circulation — so the
names here are a starting probe list, and the invocation forms are what the
vendors documented as of August 2026. The installed version's own `--help` is
the authority, always.

- [The sweep](#the-sweep)
- [Confirming a candidate](#confirming-a-candidate)
- [Non-interactive invocation forms](#non-interactive-invocation-forms)
- [Argument-passing gotchas](#argument-passing-gotchas)
- [Local inference endpoints](#local-inference-endpoints)
- [Deterministic media tools](#deterministic-media-tools)

## The sweep

Four passes, cheapest first. Run them before concluding anything about what this
machine can do.

**1. Probe names on `PATH`.** Loop a candidate list through `command -v` in one
call rather than one call per name. Treat every hit as unconfirmed: `command -v`
also resolves shell builtins, functions, and aliases, so a "found" line can name
something that is not an executable at all. A hit whose resolved value is a bare
word rather than a path is one of those.

**2. List the install directories.** Agent CLIs cluster in a small number of
places, and this pass finds the ones nobody remembered to probe for:

| Location | Holds |
|---|---|
| `~/.local/bin`, `/usr/local/bin`, `~/bin` | Most single-binary installers |
| `~/.<toolname>/bin` | Tools that install into their own dotdir |
| npm/pnpm/bun global bin, `~/.cargo/bin`, `~/go/bin`, `pipx`/`uv tool` bins | Package-manager installs |
| `/opt`, Homebrew's prefix, Flatpak and Snap bins | System and sandboxed packages |

**3. Read config and credential paths.** These reveal what is *set up*, which is
a different question from what is installed:

- `~/.config/<tool>/`, `~/.<tool>/`, `~/.local/share/<tool>/` — settings, and
  often an `auth.json`-style credential store listing which providers have a
  live login.
- Per-project config in the working tree — a tool may be configured here and
  nowhere else.
- Environment: API-key variables for the major providers, and `*_BASE_URL`
  overrides that silently point a tool at a proxy or a local endpoint.

Read credential stores for **which providers appear**, never for the secret
values — see `secrets-hygiene`.

**4. Check what is running.** Local inference is a server, and servers stop. See
[Local inference endpoints](#local-inference-endpoints).

## Confirming a candidate

For each survivor, in order, stopping at the first failure:

1. `<tool> --help` (and `--version`). Confirms it is really an agent CLI, and
   prints *this* build's flags — which is the only version of the flags that
   matters.
2. Its model-listing subcommand — commonly `models`, sometimes `list-models` or
   a `--list-models` flag. **This is a catalog, not an entitlement.** Vendors
   list every model the tool can address; the ones you hold no credential for
   look exactly like the ones you do.
3. Its credential or auth subcommand — often `auth list`, `providers`, `login
   --status`, or `whoami`. This is what separates the catalog from what you can
   actually call. A tool listing dozens of models with one credentialed provider
   is the normal case, not an anomaly.
4. A real one-turn call with a trivial prompt and a canary token, per the main
   skill. Nothing short of this proves the reachable layer.

Steps 1–3 are free and offline. Step 4 costs a token or two of the user's quota
and is still the only step that proves anything.

## Non-interactive invocation forms

Vendor-documented as of August 2026. Confirm against `--help` before use.

| Tool | Binary | One-shot form | Structured output |
|---|---|---|---|
| Claude Code | `claude` | `claude -p "prompt"` | `--output-format json` |
| Codex CLI | `codex` | `codex exec "prompt"` | `--json`, `--output-schema`, `--output-last-message` |
| Gemini CLI | `gemini` | `gemini -p "prompt"`, also reads stdin | `--output-format json` |
| Qwen Code | `qwen` | `qwen -p "prompt"`, also stdin | documented headless mode |
| Antigravity | `agy` | `agy --print="prompt"` | `--output-format json`, `--json-schema` |
| Cursor CLI | `cursor-agent` | `cursor-agent -p "prompt"` | documented headless mode |
| Amp | `amp` | `amp -x "prompt"` | — |
| Droid (Factory) | `droid` | `droid exec "prompt"` | documented exec mode |
| OpenCode | `opencode` | `opencode run "message"` | JSON output format, `-q` to drop the spinner |
| Aider | `aider` | `aider --message "..."` | — |
| Goose | `goose` | `goose run -t "text"` | — |
| Auggie | `auggie` | `--print` mode | — |
| Copilot CLI | `copilot` | documented headless automation | — |
| Codewhale / pool | `codewhale`, `pool` | `<tool> exec` | — |
| llm (Datasette) | `llm` | `llm -m <model> "prompt"` | `--schema` |
| Ollama | `ollama` | `ollama run <model> "prompt"` | `--format json` |

Common companions worth checking for in `--help`: a model selector (`--model`),
a working-directory or workspace flag, a sandbox or permission-mode flag, a
timeout, and a resume/continue flag. Many of these tools also speak MCP, and
several are registered as ACP agents — if a structured protocol path exists and
the task is more than one turn, it is steadier than parsing a CLI's stdout.

## Argument-passing gotchas

Each of these produces a *plausible reply* rather than an error, which is why
the canary check in the main skill is not optional.

- **Flag-value vs positional.** Some tools take the prompt as a positional
  argument, others as the value of a flag. Passing a positional to a tool that
  wanted a flag value leaves the prompt empty, and the agent answers the empty
  task with a greeting or a self-description.
- **Go's `flag` package** — recognizable from a `Usage of <tool>:` header with
  single-dash flags — requires `--flag=value` or `--flag value`, and **stops
  parsing flags at the first non-flag argument**. A prompt written after other
  flags can be swallowed or ignored entirely. Prefer the `=` form there.
- **Quoting and shell expansion.** Prompts contain backticks, `$`, `!`, and
  newlines. Single-quote, or pass the prompt on stdin where the tool supports
  it, which sidesteps argument parsing completely.
- **Truncation.** Very long prompts hit argument-length limits. Write the prompt
  to a file and feed it on stdin.

## Local inference endpoints

Local servers are configured persistently and running intermittently. Probe with
a short timeout before planning around one:

| Runtime | Default endpoint | Listing path |
|---|---|---|
| Ollama | `http://127.0.0.1:11434` | `/api/tags`, or `ollama list` |
| LM Studio | `http://127.0.0.1:1234` | `/v1/models` |
| llama.cpp server, vLLM, Jan, and most others | varies, OpenAI-compatible | `/v1/models` |

A connection refused means the runtime is installed and not running — usually
startable, which makes it worth reporting to the user rather than skipping.
Anything answering here is offline-capable, free per call, and sends nothing off
the machine, so it needs no data-sharing consent.

## Deterministic media tools

Probe for these in the same sweep as the agent CLIs — they sit above every model
on the ladder. `reference/capability-routing.md` maps them to capabilities.

`ffmpeg`, `ffprobe`, `whisper` / `whisper.cpp` / `faster-whisper`, `tesseract`,
`pdftotext` and the rest of poppler-utils, `qpdf`, `pandoc`, ImageMagick
(`magick`, `convert`), `exiftool`, `sox`, `yt-dlp`, `libreoffice --headless`.

Two of these fail in a way worth knowing about: they install without their data.
`tesseract` needs language data files and reports a clean error when they are
missing; local Whisper runtimes need model weights downloaded separately. Both
are installed-but-inert until that second piece is present, and both say so
plainly if you run them rather than assuming.
