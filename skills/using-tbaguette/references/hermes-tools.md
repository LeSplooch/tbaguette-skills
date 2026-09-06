# Hermes Agent Tool Mapping

Skills speak in actions ("dispatch a subagent", "create a todo", "read a file"). On Hermes Agent these resolve to the tools below.

## Tools

| Action skills request | Hermes tool |
|---|---|
| Read a file | `read_file` |
| Create a new file | `write_file` |
| Edit a file (targeted patch) | `patch` |
| Run a shell command | `terminal` |
| Search file contents | `search_files` |
| Find files by name | `terminal` with `find` |
| Fetch a URL / read a webpage | `web_extract(urls=[...])` |
| Search the web | `web_search(query=...)` |
| Dispatch a subagent | `delegate_task(goal=..., context=..., toolsets=[...], role="leaf")` |
| Task tracking | `todo` tool |
| Invoke a skill | `skill_view("skill-name")` |

## Instructions file

When a skill mentions "your instructions file," on Hermes Agent this is **AGENTS.md** in the project directory, or **`SOUL.md`** globally at `~/.hermes/SOUL.md`.

## Invoking a skill

Hermes Agent has a `skills` toolset with `skill_view` and `skills_list`.
TBaguette's skills are registered with that loader under the `TBaguette:`
namespace, which Hermes derives from the plugin name — so the prefix is
required, not decoration, and the bare name is a miss:

```
skill_view("TBaguette:orienting-in-unfamiliar-code")
```

They are explicit loads rather than entries in `<available_skills>`, so
`skills_list` is how you see what is there. If a lookup returns "not found",
read the SKILL.md directly, using the absolute skills directory the session's
bootstrap prints rather than a guessed path — Hermes names the install
directory for the plugin, not for the repository.

## Subagent dispatch

Use `delegate_task` to spawn isolated subagents for parallel or sequential workstreams:

```
delegate_task(goal="...", context="...", toolsets=[...], role="leaf")
```

If `delegate_task` is unavailable, do the work inline rather than inventing tool calls.

## Task tracking

Use the `todo` tool for task tracking within a session. For multi-agent task boards, use `hermes kanban` CLI if available. Treat older `TodoWrite` references as the task-tracking action.
