# Dim Implementation Plan

This plan follows the project direction in `README.md` and `AGENTS.md`.

## 1. Project Foundation

- Create a Rust/Cargo project.
- Keep `src/main.rs` thin: startup, shutdown, and fatal error handling.
- Put core editor logic under `src/lib.rs` modules.
- Use minimal dependencies, starting with terminal handling and Unicode width support only when needed.

## 2. Core Model

- Implement `buffer` as a simple line-based text buffer.
- Implement `selection` with `anchor` and `head`.
- Implement position helpers that distinguish byte index, character index, and display column.
- Implement `editor_state` with mode, buffer, selection, messages, dirty flag, yank buffer, and file path.
- Start with one buffer and one selection, without blocking future multi-selection support.

## 3. Command-First Architecture

- Define editor actions as internal commands.
- Route all state mutation through command execution.
- Keep key bindings separate from commands.
- Preserve the required flow: `Key input -> Keymap -> Command -> Editor state mutation`.

## 4. Minimal Editing MVP

- Insert text.
- Move left, right, up, and down.
- Move to line start/end and file start/end.
- Delete, change, yank, and paste.
- Track dirty state.
- Open and save files.

## 5. Terminal, Input, and Rendering

- Add terminal raw mode setup and guaranteed restoration.
- Parse text input, control keys, escape sequences, special keys, and paste events.
- Render text area, status line, command line, and message area.
- Keep rendering read-only against editor state.

## 6. Undo and Redo

- Add an undo/redo transaction model.
- Make insert, delete, replace, and paste undoable.
- Keep the model compatible with future SKK commit-as-single-transaction behavior.

## 7. Command-Line Mode

- Add command-line input state.
- Map textual commands to internal commands.
- Support initial commands: `write`, `quit`, `force_quit`, `write_and_quit`, and `open_file`.

## 8. Search

- Implement literal forward and backward search.
- Track the current search match.
- Add next/previous match commands.
- Defer regex search.

## 9. SKK

- Keep SKK state separate from editor modes.
- Implement Direct, Hiragana, Katakana, Composing, Converting, and Registering states.
- Add romaji-to-kana conversion.
- Support okuri-nasi SKK dictionary lookup.
- Add candidate selection, confirm, and cancel.
- Add writable user dictionary.
- Render preedit and candidates outside buffer text.
- Ensure SKK preedit does not mutate the buffer until confirmed.

## 10. Deferred Language Intelligence

- Do not include LSP or Tree-sitter in the first MVP.
- Keep the architecture from blocking future LSP integration.
- Keep the buffer, position, and rendering models precise enough to support future Tree-sitter integration.
- Revisit LSP and Tree-sitter after the core editor model, command execution, terminal rendering, file operations, and SKK path are stable.

## 11. Documentation

- Update README with build instructions once the Cargo project exists.
- Document implemented commands and current limitations.
- Keep AGENTS-aligned architecture notes in sync when structure changes.
