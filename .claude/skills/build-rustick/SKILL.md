---
name: build-rustick
description: Build the Rustick TUI task manager with a coordinated team of 10 agents. Spawns specialists for each module, validates each phase with user before proceeding, tests compilation and functionality.
---

<context>
Building "Rustick" — a persistent, interactive TUI task manager in Rust.
Experience target: native GUI fluidity, entirely in terminal.
Stack: Rust stable, ratatui + crossterm, tokio, chrono, serde/serde_json, rusqlite (optional).
Project root: {PROJECT_ROOT} (detected at runtime via pwd).
</context>

<team_config>
team_name: rustick-builders
size: 10
mode: validation (user approves each phase before next)
</team_config>

<roles>
| # | Name | Scope | Files Owned |
|---|------|-------|-------------|
| 1 | lead | Orchestration, merge conflicts, final assembly | Cargo.toml, src/main.rs |
| 2 | architect | Module structure, trait design, dependency graph | src/lib.rs, architecture docs |
| 3 | core-loop | Async event loop, crossterm raw mode, tokio channels | src/event/mod.rs, src/event/handler.rs |
| 4 | ui-layout | Ratatui layout, 3-panel composition, widget rendering | src/ui/mod.rs, src/ui/task_list.rs |
| 5 | task-model | Task/Reminder structs, CRUD operations, state mutations | src/model/task.rs, src/model/reminder.rs |
| 6 | timeline | Day/week view, upcoming strip, time markers | src/ui/timeline.rs, src/ui/upcoming.rs |
| 7 | scheduler | Background tokio task, reminder trigger detection, alerts | src/event/scheduler.rs |
| 8 | search-filter | Fuzzy search, focus mode, tag parsing, filter logic | src/model/filter.rs, src/ui/search.rs |
| 9 | keybinds-input | Keybinding system, vim modes, inline input widget | src/config.rs, src/ui/task_input.rs, src/ui/popup.rs |
| 10 | tester | cargo build, cargo clippy, cargo test, integration tests | tests/ |
</roles>

<phases>

## Phase 0 — Project Bootstrap

```actions
- git init
- cargo init --name rustick
- Create .gitignore (target/, .DS_Store)
- Create GitHub repo (public, MIT license)
- git remote add origin
- Initial commit
```

GATE: User validates repo exists on GitHub.

## Phase 1 — Architecture & Cargo.toml

Architect teammate produces:
- Complete Cargo.toml with pinned dependencies
- src/ directory structure (all mod.rs files with placeholder modules)
- App state enum design (modes, panels, focus)

Dependencies:
```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ulid = "1"
fuzzy-matcher = "0.3"
thiserror = "2"
directories = "5"

[dev-dependencies]
tempfile = "3"
```

GATE: User validates architecture + Cargo.toml. `cargo check` passes.

## Phase 2 — Core Event Loop

Core-loop teammate implements:
- Crossterm raw mode enter/exit with panic hook
- Tokio channel (mpsc) for Event enum: Key(KeyEvent), Tick, Reminder(TaskId), Resize(u16,u16)
- Main loop: poll events → dispatch to handler → render UI
- Tick interval: 250ms
- Graceful shutdown on Ctrl+C / SIGTERM

GATE: User validates. App launches, shows blank screen, responds to 'q' to quit.

## Phase 3 — UI Layout

UI-layout teammate implements:
- 3-panel layout: sidebar (20%), main (50%), right panel (30%)
- Task list widget with sections: Overdue (red), Today (yellow), Upcoming (dim), No Date
- Status bar at bottom: mode indicator, task count, clock
- Terminal resize handling

GATE: User validates. App shows structured layout with placeholder data.

## Phase 4 — Task Model & Persistence

Task-model teammate implements:
- Task struct: id (ULID), title, body, status (Todo/Done/Archived), priority (1-4), due_at, created_at, tags (Vec<String>), reminders (Vec<Reminder>)
- Reminder struct: id, trigger_at, recurrence (Option<String> cron pattern), acknowledged
- JSON store: load/save to ~/.local/share/rustick/tasks.json
- Auto-save debounced (500ms after last mutation)
- Session state: panel, cursor, scroll, filters → ~/.local/share/rustick/session.json

GATE: User validates. Tasks persist across restarts.

## Phase 5 — Keybindings & Input

Keybinds-input teammate implements:
- Mode system: Normal, Insert, Search, Focus
- Keybinding table (see reference below)
- Inline task creation: 'n' → input line appears → Enter confirms → Esc cancels
- Inline edit: 'e' → edit title in-place → Tab cycles fields
- Confirmation popup for delete ('d')
- Help overlay ('?')

Keybinding reference:
| Key | Mode | Action |
|-----|------|--------|
| j/k | Normal | Move cursor |
| h/l | Normal | Switch panel |
| Tab | Normal | Cycle sections |
| n | Normal | New task (Insert mode) |
| e | Normal | Edit task |
| Space | Normal | Toggle done |
| d | Normal | Delete (confirm) |
| r | Normal | Reschedule reminder |
| / | Normal | Search |
| f | Normal | Focus/filter toggle |
| 1-4 | Normal | Set priority |
| Enter | Insert | Confirm |
| Esc | Any | Back to Normal |
| q | Normal | Quit |
| Ctrl+s | Any | Force save |
| ? | Normal | Help |

GATE: User validates. Can create task, edit it, mark done, delete.

## Phase 6 — Timeline & Upcoming

Timeline teammate implements:
- Day view: vertical 24h strip, current time marker, reminder blocks
- Week view: 7 columns with task counts + reminder dots, h/l navigation
- Upcoming strip (always visible right panel): next 5 reminders with relative time ("in 12m")
- Real-time clock update every Tick

GATE: User validates. Timeline renders, upcoming shows next events.

## Phase 7 — Scheduler & Reminders

Scheduler teammate implements:
- Background tokio task checking reminders every Tick
- When reminder.trigger_at <= now && !acknowledged → send Reminder event
- On Reminder event: pulse animation on task row (alternating color every 500ms)
- Popup: task title + Dismiss / Snooze (5m, 15m, 1h)
- Recurring: after acknowledge, compute next from cron, insert new reminder
- Quick reschedule ('r'): mini-input for relative time ("+2h", "tomorrow", "next monday")

GATE: User validates. Set reminder 10s in future, see it trigger visually.

## Phase 8 — Search & Filter

Search-filter teammate implements:
- '/' activates search overlay with fuzzy matching (fuzzy-matcher crate)
- Results update per keystroke, Enter jumps to matched task
- Focus mode ('f'): filter bar shows active filters
- Filters: by priority, by tag (#tag in title), by due range
- Tags auto-extracted from title text

GATE: User validates. Search finds tasks instantly, filters narrow view.

## Phase 9 — Integration Testing & Polish

Tester teammate:
- `cargo clippy -- -D warnings` → zero warnings
- `cargo test` → all pass
- Integration tests: create task → set reminder → trigger → dismiss → verify state
- Performance: 1000 tasks render without lag
- Edge cases: empty state, Unicode input, rapid key spam, resize during popup
- Session restore test: quit → relaunch → same state

GATE: User validates all tests green. `cargo build --release` succeeds.

## Phase 10 — Ship

Lead teammate:
- Final commit with clean history
- Push to GitHub
- Write README.md (project description, install, usage, keybindings, screenshots)
- Tag v0.1.0
- Confirm binary runs on macOS

GATE: User validates. Repo public on GitHub, binary works.

</phases>

<execution_rules>
- NEVER proceed to next phase without explicit user approval
- Each phase: spawn relevant teammate(s) → they implement → lead reviews → present to user
- If user rejects: teammates fix based on feedback, re-present
- Teammates communicate via SendMessage to lead
- No teammate writes outside their owned files (see roles table)
- All teammates use `cargo check` before declaring done
- Tester runs after every phase (incremental validation)
- On failure: identify which teammate's code broke, route fix to them
</execution_rules>

<constraints>
- No external daemon. Single process.
- No network. Pure local.
- Startup < 50ms for 1000 tasks.
- Memory < 20MB typical.
- Handle resize gracefully.
- Unicode correct (multi-byte cursor positioning).
- Never block main thread. All I/O async.
- Persist on Ctrl+C / SIGTERM.
- No panics. Result + thiserror everywhere.
- No comments in code.
</constraints>

<success_criteria>
- `cargo build --release` zero warnings
- Launch shows full TUI immediately
- Create task → set reminder → trigger → dismiss: all inline, no leaving interface
- Quit → relaunch: exact previous state restored
- Search: 1-keystroke latency
- Resize: no crash, no corruption
- 1000 tasks: no perceptible lag
- GitHub repo public, MIT license, tagged v0.1.0
</success_criteria>
