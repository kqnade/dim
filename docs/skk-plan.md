# Dim Editor - Build Plan

## Current Status (183 tests passing)

### Implemented Modules
- **buffer** - LineBuffer with insert/delete, UTF-8 support, Japanese width
- **selection** - Selection struct (anchor + head)
- **position** - Position (line, col)
- **input** - InputEvent, KeyCode, Modifiers + crossterm parsing
- **undo** - UndoManager with Transaction/EditOp
- **terminal** - Raw mode with crossterm (TTY-safe tests)
- **file_io** - read_file/write_file with backup
- **config** - Config struct with serde (tab_width, skk paths, etc.)
- **editor_state** - Core state: buffer, selection, mode, undo, movement
- **command** - Command enum + CommandRegistry for :w, :q, :open, etc.
- **keymap** - Colemak-DH keymap: m/n/e/i movement, s insert, z/Z undo/redo, x/c/v cut/copy/paste, ; command
- **renderer** - Text area, status line, scroll offset, Japanese truncation
- **app** - Main event loop with Insert/Normal/Command/Search modes
- **skk** - Basic SKK engine with romaji table, dictionary, candidate selection

### Colemak-DH Key Bindings
- `m/n/e/i` = left/down/up/right
- `s` = Insert mode
- `t` = Append (not yet implemented)
- `;` = Command mode
- `x` = delete char/selection
- `c` = yank
- `v` = paste after, `V` = paste before
- `w` = change
- `z` = undo, `Z` = redo

### User Configuration Context
- **Layout**: Colemak-DH
- **SKK**: Uses yaskkserv2 with dictionaries in `~/.skk/`
- **AZIK**: User mentioned using AZIK (romanization style)
- **Neovim config**: `~/.config/nvim/` with skkeleton, kanagawa theme

## What's NOT Implemented

### Critical for MVP
1. **SKK dictionary loading from files** - Need to load `~/.skk/SKK-JISYO.*` files
   - Problem: Files are EUC-JP encoded, need UTF-8 conversion
   - yaskkserv2 binary dict (`dictionary.yaskkserv2`) can be dumped via `yaskkserv2_make_dictionary --dictionary-filename ~/.skk/dictionary.yaskkserv2 --output-jisyo-filename output.txt`
   - iconv conversion from EUC-JP needed

2. **User dictionary write** - SKK user dictionary should be writable
   - Path: `~/.skk/user-dictionary.txt` or similar
   - Save new entries when user registers unknown readings

3. **Word movement** - `l/u/y` for word backward/forward (from user's nvim config)
   - `l` = word backward, `u` = word forward, `y` = WORD forward

4. **Visual mode** - `a`/`A` for visual/line visual (from nvim config)

5. **Selection extension** - Shift+movement to extend selection

6. **Go to line** - `:123` to jump to line 123

7. **Find and replace** - `:%s/old/new/`

### Important but Deferred
- System clipboard (OSC 52)
- Mouse support
- Line numbers in renderer
- Word wrap
- Multiple buffers
- Macro recording
- LSP/Tree-sitter

### Open Issues
- `delete_char` currently works but selection handling could be cleaner
- Command mode `:wq` logic is simplistic
- Search mode needs `n`/`N` for next/previous match
- SKK preedit should be displayed somewhere (currently only status line shows mode)

## Next Agent Tasks (Prioritized)

1. **Implement word movement** (`move_word_forward`, `move_word_backward`)
2. **Add visual mode** (basic selection extension)
3. **Implement go-to-line** (`:123`)
4. **Add find/replace** (`:%s/old/new/`)
5. **Load SKK dictionaries** (handle EUC-JP encoding or use UTF-8 converted version)
6. **User dictionary write** (save learned words to `~/.skk/user.txt`)
7. **Improve search** (`n`/`N` for next/previous)

## Testing Notes
- All public APIs should have tests (TDD)
- Use `cargo test` to verify
- Commit with gitmoji style: `✨ feat:`, `✅ test:`, `🐛 fix:`
- **Do NOT use `--no-gpg-sign`** - if signing fails, stop and wait for user
- **Do NOT touch git config**

## Files to Know
- `src/keymap.rs` - Colemak-DH bindings
- `src/skk.rs` - SKK engine (needs dictionary file loading)
- `src/editor_state.rs` - Core editing operations
- `src/app.rs` - Event loop and command execution
- `src/renderer.rs` - Screen rendering
- `AGENTS.md` - Project guide with architecture and rules
