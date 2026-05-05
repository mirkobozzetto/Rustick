# Rustick

A persistent TUI task manager with calendar, reminders, and real-time alerts — entirely in the terminal.

## Features

- Interactive task management (create, edit, delete, prioritize)
- Calendar date picker for scheduling
- Reminder system with terminal bell alerts
- Fuzzy search across tasks
- Timeline and upcoming events view
- Session persistence (SQLite)
- Vim-style keyboard navigation

## Install

```bash
cargo install --path .
```

## Usage

```bash
rustick
```

## Keybindings

### Navigation
| Key | Action |
|-----|--------|
| j/↓ | Move down |
| k/↑ | Move up |
| h/← | Previous panel |
| l/→/Tab | Next panel |

### Tasks (Main panel)
| Key | Action |
|-----|--------|
| n | New task |
| e | Edit title |
| Enter | Edit description |
| t | Set date/time |
| Space | Toggle done |
| d | Delete |
| 1-4 | Set priority |

### Global
| Key | Action |
|-----|--------|
| / | Search |
| ? | Help |
| q | Quit |

## Date Picker

Press `t` on a task to open the calendar:
- Arrow keys to navigate days
- H/L to change months
- Enter to select, then type time (HH:MM)

## Data

Tasks stored in SQLite at `~/.local/share/rustick/rustick.db`.
Session state persists between launches.

## Stack

- [ratatui](https://ratatui.rs) + crossterm for TUI
- tokio for async event loop
- rusqlite for persistence
- chrono for time handling

## License

MIT
