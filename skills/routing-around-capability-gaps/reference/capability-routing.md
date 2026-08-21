# Capability routing

One row per capability gap. Read the row you need. The **first** column that has
an entry is the one to use — deterministic tools sit above models because they
are free, offline, repeatable, and fail loudly instead of plausibly.

- [Reading things](#reading-things)
- [Producing things](#producing-things)
- [Scale, cost, and environment](#scale-cost-and-environment)
- [When a model is genuinely required](#when-a-model-is-genuinely-required)
- [Model facts expire](#model-facts-expire)

## Reading things

| Gap | Deterministic tool first | Model only if |
|---|---|---|
| Speech in an audio file | Local Whisper (`whisper.cpp`, `faster-whisper`); `ffmpeg` to convert or segment first | You need reasoning *about* the audio — tone, overlapping speakers, non-speech events — rather than its words |
| Audio that isn't speech | `ffprobe` for format and duration, `sox` for analysis | The question is semantic: what is this sound, what is happening |
| Text in an image | `tesseract` for printed text | Handwriting, layout-heavy scans, screenshots where meaning depends on what's around the text |
| Understanding an image | — | Always a model: charts, diagrams, UI screenshots, photos, "does this look right" |
| A PDF | `pdftotext` (poppler), `qpdf` for structure, `pandoc` to convert | The PDF is scanned images, or the layout carries the meaning |
| Office documents | `pandoc`, `libreoffice --headless --convert-to` | Only after conversion, on the resulting text |
| A video | `ffmpeg` to pull frames, `ffprobe` for metadata, extract the audio track and take the audio path | Continuity across frames matters — action, motion, ordering — not just sampled stills |
| A remote video or stream | `yt-dlp` to fetch, then the video path | Same as video |
| Image metadata | `exiftool` | Never |

The pattern in every row: **decompose the media with a tool, then route only the
irreducible semantic question to a model.** A 40-minute recording becomes a
transcript locally, and the transcript is plain text this model reads natively —
no delegation, no data leaving, no unverifiable claim.

## Producing things

| Gap | Deterministic tool first | Model |
|---|---|---|
| An image from a description | — | An image-generation model |
| An image from data or a spec | ImageMagick, a plotting library, SVG written directly | Never — deterministic beats generative for anything with a correct answer |
| Speech from text | A local TTS engine such as `piper`, or the platform's built-in `say`/`espeak` | A speech-capable model, when voice quality is the point |
| Video | `ffmpeg` for assembly, concatenation, subtitles, format | A video-generation model, for synthesized footage only |
| A converted file format | `pandoc`, `ffmpeg`, ImageMagick, `libreoffice` | Never |

## Scale, cost, and environment

| Gap | Approach |
|---|---|
| Input exceeds the context window | Chunk and map-reduce here first — it is usually cheaper and always auditable. Route to a long-context model when the task genuinely needs whole-input attention, such as cross-references spanning the entire document |
| Thousands of mechanical items | A small or local model, in bulk. This is the one cost-driven route that is not benchmark-shopping — it changes the bill by an order of magnitude |
| Must not touch the network | A local runtime, or nothing. No cloud harness qualifies, whatever its privacy tier |
| Sensitive data that cannot leave | Same: local runtime or deterministic tool only. The decision is the user's, not an implementation detail |
| Needs a real browser | A browser-automation tool, not a model that describes what a page probably says |
| Needs a GPU, a specific runtime, or a sandbox | Check what is installed before assuming a hosted agent is the answer |
| Current provider down, rate-limited, or out of quota | Route to any credentialed equivalent. Observable, not a preference |
| Embeddings or semantic search | A local embedding model — small, fast, offline, and the standard tool for the job |

## When a model is genuinely required

Once the ladder reaches a model, pick on the capability actually needed, in this
order:

1. **A different model in the harness already running.** Cheapest possible route
   — no new process, no new credential, usually no new provider boundary. Check
   the harness's own model list first.
2. **A local model, if one is running and can do it.** No consent, no cost, no
   data leaving the machine.
3. **A credentialed cloud model in another harness, whose modality matches.**
   Verified credentialed, per the three layers, and gated on the user's consent.

If none of the three exists, report the gap. There is no fourth option, and in
particular "try the request again and hope" is not one.

Within a tier, prefer the cheapest model that has the capability. A cheap model
that can take audio beats an expensive one that cannot, and "the better model"
is not a tiebreaker worth another provider boundary.

## Model facts expire

Everything in this section was accurate in August 2026 and is the fastest-rotting
content in this skill. Model lineups change monthly. Use it to form a hypothesis,
then confirm against what the installed tool actually lists and what a real call
actually accepts.

- **Native audio and video input** has been Gemini's distinguishing capability —
  text, image, audio, video, and PDF into one model, with million-token-plus
  context.
- **Frontier Claude and GPT models take text and images**, and reach audio or
  video through separate models or a preprocessing pipeline rather than natively.
- **Local Whisper matches or beats the paid cloud transcription baseline** on
  accuracy, at zero marginal cost, fully offline. Choosing a cloud model for
  plain transcription is a loss on every axis including quality.
- **Long context is not free attention.** A million-token window accepts the
  input; it does not guarantee the model attends evenly across it. Chunking with
  explicit retrieval often beats dumping everything in.
- **A model's advertised modality is not the harness's.** A harness can front a
  multimodal model and still offer no way to attach a file. What matters is
  whether *this CLI* can get the bytes to the model — confirm with a real call
  on a file whose contents you already know, so a wrong answer is recognizable.

That last check is worth doing once per harness and recording in the capability
spec: a file with known contents, routed through the full invocation, is the only
evidence that the media path works end to end.
