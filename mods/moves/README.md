# moves

A Claude Code mod for the end of a run. A run turns up things it does not act on: a second
instance of the bug it just fixed, a call that could have gone the other way, a line of the
request it gave up on. They live in the conversation and nowhere else, so by the time the work
is reported they are usually gone, and the report ends with "let me know what's next". This mod
gives them a table. The model sets a move aside at the moment it decides not to take it, the
table is handed back when the run closes so the next move is offered from it, and what you do
not pick stays one press away. It is the tool half of TBaguette's `offering-the-next-move` skill,
which covers what a real offer is made of.

## Install

At the prompt of a terminal session:

    /plugin install moves --marketplace LeSplooch/tbaguette-skills

From a shell:

    claude plugin install moves --marketplace LeSplooch/tbaguette-skills

In the desktop app: click + next to the prompt box, then Plugins, Add marketplace with
`LeSplooch/tbaguette-skills`, then Add plugin.

## Use

    /moves                 what is on the table for this project
    /moves take 2          put move #2 in the prompt, to edit and send as your next request
    /moves send 2          send move #2 as your next request, as it stands
    /moves drop 2          take move #2 off the table
    /moves add <text>      put a move of your own on the table
    /moves clear           take every move off the table
    /moves gate off        stop holding turns that end without an offer; /moves gate on resumes

The model puts moves on the table as it works, through two tools the mod registers: `set_aside`
takes a kind, the move in a few words and what it costs, and `harvest` hands the table back,
strongest first, right before the closing question. The model can clear a move it set aside in
this session once it is done. A move an earlier session left, and one you added, leave the table
only by your hand.

Only you take, send, drop or add a move, clear the table, or switch the gate. A move on the table
is a candidate, never an instruction: the model is told to put it to you, and it becomes a
request only when you pick it in a question, press it on the band, or type `/moves take` or
`/moves send`. `/moves` run by a plugin, a schedule or another session lists the table and
changes nothing.

## How it behaves

- **Setting aside.** Four kinds, strongest first: a line of the request that was given up, a
  ruling that could have gone the other way, scope pushed out on purpose, and something found on
  the way past. A move is a label of at most 80 characters with a cost of at most 300. The mod
  refuses a cost that only says the label again, a move already on the table, and a thirteenth
  move from one session. Your own moves, from `/moves add`, rank above all four.
- **The close.** When a turn you started changed a file (Edit, Write or NotebookEdit, a
  subagent's included) or set a move aside, and is about to end with no question from the model
  since, the mod holds the end once. The model is handed the table and asked to put one question
  under its report: the move it would take first, one option that stops, four options at most.
  If its last lines announce what it will do next ("Tonight I'll rebuild the runner"), the hold
  quotes them. A background task that reports back later belongs to the run that started it, so
  the turn its report starts can be the one held.
- **What is never held.** A turn that changed no file and set nothing aside. A turn that already
  asked. A turn with a background task still running or a schedule waiting to wake the session.
  A run started by a plugin, a schedule, a channel or another session, or with no app or
  terminal attached, where nobody may be there to answer. The same turn twice. Any turn while
  the gate is off: `/moves gate off` is kept for every session, and read again at each close.
- **The offer.** The question is Claude Code's own, asked by the model; the mod only watches it.
  An option whose label says a move on the table is that move. Picked, it leaves the table, and
  the model is reminded that a chosen move is a new request. Passed over, it stays and counts
  one more offer. An option that says no move, like the one that stops, is not kept.
- **Taking a move.** Your prompt carries the move's number and its label, which you saw. What a
  model wrote about the move, its cost, never goes out as your words: it rides beside the
  request as a note marked as a model's. Where the session has no prompt box to fill, the mod
  sends the move under its own name, saying that you picked it.
- **The band.** After a turn, while the table holds a move you have not waved off, a band above
  the prompt shows up to three of them as buttons. Not now hides the moves it showed; a move it
  had no room for, or a new one, brings the band back after a later turn. Your next prompt
  lowers the band.
- **Across sessions.** The table belongs to the project: the repository's top folder, or the
  session's own folder outside one. It is kept in the mod's store, so a later session in the same
  project starts with what earlier ones left, each marked with its age, and the band is up before
  the first turn. A move is forgotten after 30 days, and its number is never given to another.
- **A full table.** The table holds 24 moves. A twenty-fifth pushes off the oldest one a model
  set aside, another session's before this one's, and a toast tells you which. A move you added
  is never pushed off: a table full of your own moves takes no more until you drop one.
- **Compaction and `/clear`.** The table is not in the conversation, so neither one touches it.
- **Not yet confirmed live.** The mod's tests run against the engine's test kit, not a live
  session. Two things in particular have not been watched: the end of a turn being held in a real
  session, and a running session seeing what another running session wrote, a move or the gate,
  which depends on how the engine shares a mod's store. A session started later does read it.
- **What it does not see.** A turn that changed things only through shell commands owes nothing
  unless a move was set aside. The match between an option and a move is by their words, so an
  option worded very differently from its move is not recognised: the move stays on the table.
- The status line shows how many moves are on the table. The band draws on the terminal and in
  the desktop app; `/moves` and the two tools work wherever the mod loads. Other harnesses
  ignore this mod.

The mod API is early access, and nothing in the skills depends on this mod.
