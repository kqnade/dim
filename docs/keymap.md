# Dim Keymap

## Design Philosophy

- **Normal mode** = 高頻度な編集操作
- **C-x prefix** = global / leader
- **C-x x** = command mode
- `:` = 使わない、または後で optional alias
- **Selection-first**: `x` で行選択 → `d` で削除、`c` でコピー、など

Normal mode は残す。`mnei`, `j/k`, `l/u` みたいな単キー移動・編集のために必要。
Vim の `:` は基本消して、`C-x x` に寄せる。

## Normal mode

### Movement

| Key | Action |
|-----|--------|
| `m` | `move_left` |
| `n` | `move_down` |
| `e` | `move_up` |
| `i` | `move_right` |
| `j` | `page_up` |
| `k` | `page_down` |
| `l` | `word_backward` |
| `u` | `word_forward` |
| `0` | `line_start` |
| `$` | `line_end` |
| `gg` | `file_start` |
| `G` | `file_end` |

### Selection + Edit

| Key | Action |
|-----|--------|
| `a` | `visual_mode` (文字単位選択開始) |
| `x` | `select_line` (行単位選択開始) |
| `d` | `delete_selection` (選択範囲を削除) |
| `c` | `yank_selection` (選択範囲をコピー) |
| `v` | `paste_after` |
| `V` | `paste_before` |
| `z` | `undo` |
| `Z` | `redo` |

**Selection behavior:**
- `a` = visual mode（anchor を現在位置にセット）
- `x` = line select mode（anchor を行頭にセット、head を次行頭に）
- 選択中の移動 (`n`, `e`, `m`, `i`) = 選択を拡張
- `Esc` = 選択を解除（cursor に戻る）

**Example combos:**
- `a` → `n` → `n` = 下2文字を選択
- `a` → `i` → `i` → `i` = 右3文字を選択
- `x` → `n` = 現在行 + 次行を選択
- `x` → `n` → `n` = 現在行 + 下2行を選択
- `x` → `e` → `e` → `e` = 現在行 + 上3行を選択
- `xd` = 行を選択して削除 (line delete)
- `xc` = 行を選択してコピー (line copy)
- `cd` = 選択範囲を削除
- `cc` = 選択範囲をコピー

### Search

| Key | Action |
|-----|--------|
| `/` | `search_forward` |
| `?` | `search_backward` |
| `*` | `search_word_under_cursor_forward` |
| `#` | `search_word_under_cursor_backward` |

### Advanced Edit

| Key | Action |
|-----|--------|
| `.` | `repeat_last_edit` |
| `%` | `jump_matching_pair` |
| `f` | `find_char_forward` |
| `F` | `find_char_backward` |
| `;` | `repeat_char_find` |
| `:` | `repeat_char_find_reverse` |
| `>` | `indent_selection` |
| `<` | `unindent_selection` |
| `=` | `format_selection` |
| `~` | `toggle_case` |

### Misc

| Key | Action |
|-----|--------|
| `qq` | `quit` (ダブルタップ) |
| `Ctrl-x` | `enter_prefix` |
| `Esc` | `cancel` / `normal_mode` |

### Reserved (insert/visual)

| Key | Action |
|-----|--------|
| `r` | block selection (C-v equivalent) |
| `s` | insert mode |
| `t` | append mode |
| `p` | open line below |
| `P` | open line above |

## Insert mode

| Key | Action |
|-----|--------|
| `Esc` | `normal_mode` |
| `Ctrl-x` | `enter_prefix` |

## C-x prefix

| Key | Action |
|-----|--------|
| `C-x C-s` | save |
| `C-x C-f` | open_file / file_picker |
| `C-x C-c` | quit |
| `C-x b` | buffer_picker |
| `C-x k` | close_buffer |
| `C-x t` | toggle_file_tree |
| `C-x g` | project_grep |
| `C-x p` | project_file_picker |
| `C-x n` | toggle_skk |
| `C-x N` | skk_menu |
| `C-x x` | command_mode / command_prompt |

### Reserved / later

| Key | Action |
|-----|--------|
| `C-x 2` | split_horizontal |
| `C-x 3` | split_vertical |
| `C-x 0` | close_window |
| `C-x 1` | only_current_window |
| `C-x o` | other_window |

## Command prompt (opened by C-x x)

```
save
quit
force-quit
open <path>
close-buffer
buffer
tree
grep <query>
set <option> <value>
skk on
skk off
skk reload
```

## SKK menu (after C-x N)

| Key | Action |
|-----|--------|
| `h` | hiragana_mode |
| `k` | katakana_mode |
| `d` | direct_mode |
| `r` | reload_dictionary |
| `s` | save_user_dictionary |
| `e` | edit_user_dictionary |
| `Esc` | close_menu |
