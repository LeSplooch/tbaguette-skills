# The express lane

Eight phases on a two-line change is a defect, not diligence. It burns the
budget that the next real decision needed, and it teaches everyone watching
that the process is theater — which is how the gates get skipped on the change
that actually needed them.

Express is the answer to that, and it is a **different process, not an absent
one**. Four beats, each with a gate that something outside your own confidence
closes. The floor is lower than the spine's. It is not zero, and the reason is
the same one that governs a sixteen-color terminal: a tight envelope still owes
the guarantee, just by a cheaper mechanism.

## Entry conditions — all of them, or it is not express

Check these before the first edit, not after. Any one false and the run is
`standard`.

1. **The target is already known.** You can name the file and the change. No
   diagnosis is pending, no design is open, no "let me look around first."
2. **It fits one reviewable diff.** One logical change, in one place, that a
   reviewer holds in their head at once.
3. **Proof is already available or one line away.** A test covers this, or a
   test can be written in the same breath as the change, or a command exists
   whose output settles it.
4. **The blast radius is `sandbox` or `repo`.** Nothing live, nothing
   irreversible, nothing that leaves the machine.
5. **Nothing about it is new.** No new dependency, no new interface, no new
   category of input, no new file that other code will have to know about.

Condition 5 catches the case the other four miss: a genuinely tiny change that
introduces a decision the rest of the codebase inherits. A three-line function
with a new public name is not express work — `naming-things` and
`designing-apis` both have a seat, and the change is small only in characters.

## The four beats

| # | Beat | What it is | Gate |
|---|---|---|---|
| 1 | **Frame** | One line: what will be true when this is done, in the requester's terms | The line is written down and does not merely restate the request. If it cannot be written in one line, this is not express |
| 2 | **Change** | The edit, with its test — `writing-the-failing-test-first` still governs the order | The test was watched failing for the right reason before the change existed |
| 3 | **Prove** | Run the thing that settles it, now, fresh | The command ran, its output was read, and the frame line is true. Not "it should work" |
| 4 | **Land** | Commit with a message that says why, and say what was done | `atomic-commits` if the tree grew more than one change; `writing-commit-messages` for the message |

No design gate, no plan, no isolation, no separate review phase. Each of those
was skipped for a reason recorded by the entry conditions, and that is what
makes it a decision rather than an omission.

## The floor express still owes

These do not scale down. They are the same non-negotiables the spine carries,
and their cost at this size is seconds.

- **The frame line exists before the edit.** Written, not held.
- **The register is still named.** Express is a smaller *run*, not a licence to
  re-read a file it already has. `crouton` binds the reads here exactly as it does
  in the full spine.
- **Something outside your own confidence closes it.** A run command, a test, a
  build. Reading the diff again is not proof.
- **A one-way door still gets a human.** Express does not change the door bound;
  see `bounding-autonomous-work`.
- **What you did not do is named.** "Fixed the off-by-one; the same pattern
  appears in two other callers and I left them" is a complete report. "Fixed"
  is not.

## Promotion, and why it only goes one way

The moment any entry condition turns out false, the run is `standard` from that
point — pick up the gates express skipped, starting with the design gate if
condition 1 or 5 is what broke.

| What you notice | Promote because |
|---|---|
| The change needs a second file to make sense | Condition 2 is false; there is a shape decision here |
| The test is hard to write | Condition 3 is false, and difficulty writing it is usually the design talking |
| "While I'm in here…" | Condition 2 is false. This is scope drift with an alibi — `managing-scope-drift` |
| The fix does not hold and you are on the second attempt | Condition 1 was false: the target was not actually known. This is a `diagnose` run |
| It touches config, a credential, a migration, or a deploy | Condition 4 is false, and it was probably false from the start |

Promotion is not a penalty and not an admission. It is the mechanism working:
express is a bet that the work is small, and the bet is designed to be cheap to
lose. What is not allowed is the reverse — a `standard` run quietly finishing as
an express one because the remainder started to look small. That judgment is
made by the part of the run that most wants to be done.

## Failure modes

| Symptom | Real cause |
|---|---|
| A one-line fix that broke something else | Beat 3 skipped: the diff was read instead of the suite being run |
| An express run that lasted all afternoon | A condition went false hours ago and promotion was refused four times |
| A tiny change that a reviewer could not follow | Condition 5: it introduced a name or an interface, and nothing decided either |
| The commit says "fix" | Beat 4 treated as a formality rather than as the beat where why gets recorded |
| The requester asked for something adjacent and got exactly the literal thing | The frame line restated the request instead of naming what would be true after |
| Everything is express | The conditions are being read as a description of the hope rather than as a test |
