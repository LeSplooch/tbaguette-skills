# Stack: voice, chat, and conversational

**Envelope.** Linear, ephemeral, and one-dimensional. There is no layout, no glance, no scanning back — in voice, everything is heard once, in order, at the speaker's pace, often while the listener is driving or cooking. In chat, the reader skims and the interface is entirely typography and structure. There is no undo for words already spoken.

**Working memory is the constraint.** A person holds roughly three or four items from speech. Every design decision follows from that ceiling.

## Craft

- **Front-load the answer.** The first sentence carries the result; the justification comes after. A reply that reasons toward a conclusion wastes the only guaranteed attention you get.
- **Options come in threes, at most.** In voice, name them in order and repeat the chosen one back. Never read a list of eight things.
- **Confirm consequential actions, and only those.** Repeat back what will happen with the specifics that matter (amount, recipient, time), and make the confirmation cheap to refuse. Confirming trivia trains people to say yes without listening.
- **Errors carry the recovery.** "I didn't catch that" without an alternative is a dead end. Offer a narrower question, an example, or an exit on every failure, and escalate the specificity each time rather than repeating the same prompt.
- **Always offer the exit.** Cancel, back, repeat, slower, and human. In chat, a way to stop a long generation.
- **Prosody is layout.** Punctuation, sentence length, and pauses do what whitespace does elsewhere. Write for the ear: short sentences, no nested clauses, no parentheses, numbers in speakable form, no URLs read aloud character by character.
- **Chat is a typographic medium.** Structure is the design: short paragraphs, headers when there are real sections, lists only for genuinely parallel items, code in blocks, tables only when the reader will compare across columns. Bold carries emphasis; bolding half the message carries nothing.
- **Length is a design decision, not a side effect.** Match the reply to the question — a one-line question deserves a one-line answer. Padding, restating the question, and closing summaries are the conversational equivalent of chartjunk.
- **Barge-in and interruption** must work in voice. A user who starts speaking wins.
- **Silence and latency.** Past ~1s of thinking, say something honest and brief. Filler that pretends to think is worse than a plain "one moment."

## Failure modes

| Symptom | Real cause |
|---|---|
| User repeats themselves | No acknowledgement token; unclear turn boundaries. |
| Nobody hears the options | More than three; buried after preamble. |
| Accidental confirmations | Everything is confirmed, so nothing is read. |
| Skimmed and misread in chat | Wall of text; conclusion in the last paragraph. |
| Dead-end loops | Same reprompt repeated with no escalation and no exit. |

## Audit hooks

Read every reply aloud at speaking pace; measure it in seconds. Test with background noise, an accent the recognizer struggles with, an interruption mid-sentence, a refusal, an ambiguous answer, and a request to repeat. In chat: read only the first line, then only the headers, and check that both still answer the question.
