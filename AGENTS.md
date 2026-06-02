# Dim Project Guide for Agents

## Overview

Dim is a **personal terminal-based text editor** being developed for the author's own daily use. It is not a general-purpose editor framework, nor is it intended as a Vim/Neovim replacement.

Key design choices:

- **Vim-like modal editing** — Normal, Insert, Command, Search modes
- **Helix-like selection-first editing** — The basic editing unit is a selection (anchor + head), not a cursor
- **Built-in SKK-style Japanese input** — Not dependent on OS IME
- **Colemak-DH-oriented** — Designed with Colemak-DH in mind, but core editor model is layout-agnostic
- **Command-first design** — Internal commands are defined first; key bindings will be mapped later via a keymap layer

## Project Scope

### Goals

- Terminal-native editing (Linux, WSL2, UTF-8 locale)
- Lightweight file editing
- Selection-first command behavior
- Built-in Japanese input (SKK)
- UTF-8 support with correct Japanese text handling
- Simple, small, understandable implementation
- Personal workflow optimization

### Non-goals (first version)

- Full Vim/Emacs/Helix compatibility
- Plugin system
- LSP, Tree-sitter
- GUI or remote UI
- Lua/Python extension system
- Complex tabs/windows/workspaces
- QWERTY-first design
- Multi-user configuration flexibility

### Deferred Future Scope

- LSP support is desirable later, but is not part of the first-version MVP.
- Tree-sitter support is desirable later, but is not part of the first-version MVP.
- Add language-aware features only after the core editor model, command system, rendering, file operations, and SKK path are stable.

### MVP Scope

The minimum viable version should include:

- Terminal raw mode and screen rendering
- Single buffer (open file, save file)
- Normal, Insert, Command modes
- Basic movement commands
- Selection representation
- Delete, change, yank, paste
- Undo and redo
- UTF-8 input and Japanese display width handling
- SKK romaji-to-kana input and okuri-nasi conversion
- SKK candidate selection, commit, cancel
- User dictionary write

## Architecture

Recommended module boundaries:

```
app          — Main event loop, initialization, teardown
editor_state — Current editor state (buffers, mode, selection, messages, etc.)
buffer       — Text storage and mutation
selection    — Selection data structures and operations
command      — Internal command definitions/registry
keymap       — Maps input events to commands
input        — Parses raw terminal input into structured events
terminal     — Terminal mode setup, teardown, low-level I/O
renderer     — Converts editor state into terminal output
command_line — Command input and parsing
search       — Search state, match finding, navigation
skk          — Japanese input state, romaji conversion, dictionary lookup, candidates
config       — Editor settings loading
undo         — Undo/redo transactions
file_io      — File reading and writing
```

### Important Structural Rule

The editor must follow this flow:

```
Key input -> Keymap -> Command -> Editor state mutation
```

NOT this:

```
Specific key -> Direct editor mutation
```

Commands must be independent from key bindings.

## Editing Model

- Selection-first: the basic unit is `anchor` + `head`
- A cursor is an empty selection (`anchor == head`)
- All editing commands operate on the current selection
- Initial implementation may support only one selection, but the model should not prevent future multi-selection support

## Modes

- **Normal** — Movement, selection, editing commands, mode transitions
- **Insert** — Text input (ASCII, UTF-8, SKK preedit, paste)
- **Command** — Textual commands (write, quit, open_file, etc.)
- **Search** — Forward/reverse search, next/previous match

SKK input states are handled separately from editor modes.

## Commands (Examples)

All editor actions are represented as internal commands. Key bindings are not finalized.

Movement:

- `move_left`, `move_right`, `move_up`, `move_down`
- `move_word_forward`, `move_word_backward`
- `move_line_start`, `move_line_end`, `move_file_start`, `move_file_end`

Selection:

- `extend_selection`, `collapse_selection`
- `select_line`, `select_word`, `select_inside_pair`, `select_around_pair`

Editing:

- `delete_selection`, `change_selection`, `yank_selection`
- `paste_before`, `paste_after`, `replace_selection`

Mode transitions:

- `enter_normal_mode`, `enter_insert_mode`, `enter_command_mode`, `enter_search_mode`

File / buffer:

- `open_file`, `save_file`, `save_file_as`, `close_buffer`, `quit`, `force_quit`

Search:

- `search_forward`, `search_backward`, `search_next`, `search_previous`

SKK:

- `skk_toggle`, `skk_confirm`, `skk_cancel`, `skk_next_candidate`, `skk_previous_candidate`

Undo / redo:

- `undo`, `redo`

## Position Model

The editor must distinguish:

- Byte index
- Character index
- Display column

Display width should account for ASCII, Japanese full-width characters, tabs, and basic symbols. Complex Unicode (emoji ZWJ, combining marks, ambiguous-width characters) may be deferred.

## Rendering

Required UI regions:

- Text area
- Status line (mode, file name, dirty flag, position, encoding, SKK state)
- Command line
- Message area
- SKK preedit / candidate area

Rendering must be separated from editor state mutation. The renderer reads state and produces terminal output.

## File Operations

- `open_file`, `save_file`, `save_file_as`, `close_buffer`
- Quitting with unsaved changes should warn or fail
- `force_quit` discards unsaved changes

Initial implementation may support a single visible buffer.

## Undo / Redo

Undoable operations: insert, delete, replace, paste, SKK commit.

SKK preedit changes should NOT become individual undo entries. The entire SKK commit (romaji input → kana conversion → candidate confirmation → commit) should undo as a single text insertion.

## Clipboard

First version: internal yank buffer only (`yank_selection`, `delete_selection_and_yank`, `paste_before`, `paste_after`).

System clipboard (OSC 52, etc.) is optional and can be deferred.

## Search

- Literal search, forward/reverse, next/previous match
- Highlight current match
- Regex search is optional

## SKK Input

Built-in SKK engine. Not dependent on OS IME.

Input flow:

```
terminal key input -> editor input layer -> insert mode -> SKK engine -> preedit state -> candidate lookup -> committed text -> buffer insertion
```

Preedit text must NOT be inserted into the buffer until confirmed.

### SKK States

- **Direct** — Bypass SKK, insert directly
- **Hiragana** — Romaji → hiragana preedit
- **Katakana** — Romaji → katakana preedit
- **Composing** — Building reading string, preedit visible but not committed
- **Converting** — Showing conversion candidates
- **Registering** — Registering a new candidate for unknown reading

### SKK Dictionary

- SKK-JISYO-style dictionaries
- System dictionary + user dictionary
- Initial support: okuri-nasi entries
- User dictionary must be writable
- Candidate registration updates the user dictionary

### SKK Rendering

Preedit and candidates rendered separately from buffer text. The renderer should show preedit text, conversion marker, current candidate, candidate list, and registration prompt.

## Input Protocol

Parse terminal input into structured events:

- Text input
- Control key
- Escape sequence
- Special key
- Paste event
- Terminal response

Requirements:

- Distinguish text input from editor commands
- Support UTF-8 input
- Support control-key sequences where possible
- Avoid relying only on legacy ambiguous terminal encodings
- Provide fallback when enhanced protocol is unavailable

May support enhanced keyboard protocols (e.g., Kitty keyboard protocol).

## Command-line Commands

Textual commands resolve to internal commands.

Required concepts:

- write current buffer
- quit / force quit / write and quit
- open file / close buffer
- set option
- show help

The command system should allow multiple frontends (command-line input, key binding, prefix command, future command palette) all calling the same internal command implementation.

## Prefix Command System

Optional Emacs-like prefix commands. Useful for file operations, buffer operations, global editor commands. Represented as a keymap state. No concrete prefix key is defined yet.

## Configuration

Minimal in the first version.

Categories:

- Editor options
- Display options
- SKK options
- File options

Possible options:

- `tab_width`
- `show_line_numbers`, `show_relative_line_numbers`
- `theme`
- `skk_enabled`
- `skk_system_dictionary_path`, `skk_user_dictionary_path`
- `default_encoding`, `line_ending`

Key bindings are intentionally not finalized. Keep command names separate from key bindings so a keymap can be added later cleanly.

## Error Handling

Recoverable errors reported in the message area. Examples: file not found, permission denied, write failed, invalid command, dictionary load failed, terminal protocol unsupported, unsaved changes.

Fatal errors must restore terminal state before exiting.

## Persistence

- User SKK dictionary (required)
- Possibly editor settings and recent files (optional)

## Design Principles

- Small core
- Command-first design
- Selection-first editing
- Mode-aware input
- SKK-native Japanese input
- Colemak-DH-oriented workflow
- No Vim compatibility burden
- No plugin system at first
- Terminal state must be restored
- Text correctness over feature count
- **Test-driven development (TDD)** — write tests before implementation
- **High test coverage** — target 90%+ coverage for all public APIs
- **Edge-case testing** — empty inputs, boundaries, invalid ranges, UTF-8 Japanese text

## Testing Guidelines

- Use `#[cfg(test)]` modules at the bottom of each source file.
- Every `pub` function should have at least one corresponding test.
- Write failing tests first, then implement the minimal code to make them pass.
- `cargo test` must pass before any task is considered complete.
- Include edge cases: empty buffers, first/last line, line start/end, multi-byte UTF-8, mixed line endings (`\n` and `\r\n`).
- **Commit after each green test** — once a test passes, commit immediately. Keep history traceable: `test: ...` for test additions, `feat: ...` for implementations, `refactor: ...` for cleanups.

## Open Decisions

These are intentionally undecided and should be decided after the core editor model is stable:

- Exact key bindings
- Exact command-line syntax
- Exact prefix command key
- Buffer storage structure (line-based initially; may later use rope, gap buffer, piece table)
- Theme format
- Status line design
- SKK visual markers
- System clipboard support
- Multiple buffer UI
- Window split support

## Agent Rules

- **Do not touch git config** — Never modify `.git/config`, global git config, or signing settings.
- **If commit signing fails** — Stop and wait for user. Do not use `--no-gpg-sign`.

## One-line Summary

A personal Colemak-DH-oriented terminal editor with Vim-like modes, Helix-like selection-first editing, optional Emacs-like prefix commands, and built-in SKK Japanese input.
