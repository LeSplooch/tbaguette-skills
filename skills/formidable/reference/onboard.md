# onboard — first run, empty, and zero data

The states you design last are the states most users see first. Empty is not an error and not an afterthought; it is the surface's introduction.

## The empty-state ladder

Not all empties are the same, and the same visual for all of them is the common failure.

| Kind | Means | Design |
|---|---|---|
| **First use** | Nothing exists yet | Explain the value in one line, show the single action that creates the first item, ideally with a sample or template |
| **Cleared** | User finished everything | Acknowledge it. This is a reward state, not a void |
| **No results** | Filter or search matched nothing | Show the query, the reason, and the way back — clear filter, broaden, correct spelling |
| **No access** | Exists but not for you | Say who can grant it and how to ask |
| **Error** | Load failed | Distinguish clearly from empty. Retry, plus what is known |
| **Not yet** | Data pending | Say when to expect it |

An empty state that says "No items" and nothing else fails all six.

## First run

- **Show the product, not a tour.** Coach marks over an interface the user has not used are read by nobody. Teach at the moment of need instead.
- **Delay every ask.** Sign-up, permissions, notifications, and payment come after the user has seen value, each with an in-product explanation before the system prompt, and each with a designed refusal path.
- **Time-to-value is the metric.** Count the steps between opening and the first useful moment; cut them. Pre-fill, provide defaults, offer sample data, and let people skip.
- **Progressive disclosure.** Advanced settings, secondary flows, and power features are present but not in the first screen. Everything visible at first run should be something a new user needs.
- **Design the second session too.** The state a returning user with three items sees is more common than either the empty or the full state.

## Rules

- **Every empty state names the next action** and makes it available right there.
- **Never illustrate emptiness with a sad character** and no path forward. Illustration is optional; the action is not.
- **Skeleton screens over spinners** where the layout is known, and only past ~300ms.
- **Do not fake progress.** A progress bar for an unknowable duration is a lie the user will remember.
- **Force every state and screenshot it.** An onboarding flow reviewed only in the happy path has not been reviewed.
