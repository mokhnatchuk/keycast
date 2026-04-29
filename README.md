# keycast

A fast Wayland daemon that fixes text typed in the wrong keyboard layout.

Press a hotkey > the selected text is instantly converted to the correct layout and pasted back. No dialogs, no delays.

## The Problem

You type `ghbdsn` instead of `привіт`. Or `руддщ` instead of `hello`. It happens to every bilingual typist - you switch layouts mid-sentence, or forget to switch at all, and end up with gibberish.

**keycast** fixes this in one keystroke: select the garbled text, hit your hotkey, and it's translated to the correct layout.

## How It Works

```
┌────────┐   Unix Socket   ┌─────────┐       ┌──────────────┐
│ Hotkey │ ──── 0x01 ────> │ keycast │ ────> | 1. Ctrl+C    | < copy selection
└────────┘                 │ daemon  │       | 2. Clipboard | < read via arboard
                           └─────────┘       | 3. Translate | < O(1) char mapping
                                             | 4. Clipboard | < write via arboard
                                             | 5. Ctrl+V    | < paste back
                                             └──────────────┘
```

### Architecture

- **IPC**: Unix Domain Socket with `sync_channel(1)` + `try_send()` - natural debounce, prevents queue buildup on held hotkeys
- **Layout mapping**: `match` on `char` - LLVM compiles to jump tables for O(1) lookup, zero dependencies
- **Keyboard simulation**: `/dev/uinput` via `evdev` crate - no X11 required, pure Wayland
- **Clipboard**: `arboard` crate - native Wayland clipboard via `wlr-data-control` protocol, no external tools needed

### Dead Key Support

The Faroese layout has two dead keys: acute accent (´) and diaeresis (¨). These compose with vowels to produce á, é, ö, ü, and so on. keycast correctly decomposes and recomposes these across layouts.

## Installation

### Prerequisites

- Linux with Wayland
- Wayland compositor supporting `wlr-data-control` protocol (Sway, Hyprland, Niri, river, dwl, etc.)
- `/dev/uinput` access (member of the `input` group, or `sudo`)
- Rust 1.85+ (edition 2024)

### Build

```bash
git clone https://github.com/mokhnatchuk/keycast.git
cd keycast
cargo build --release
```

### Install

```bash
sudo cp target/release/keycast /usr/local/bin/
```

### Permissions

```bash
sudo usermod -aG input $USER
```

Log out and back in for the group change to take effect.

## Usage

Start the daemon:

```bash
keycast
```

Trigger a fix:

```bash
keycast --trigger
```

Add **keycast** to your compositor/DE autostart and bind `keycast --trigger` to a hotkey of your choice. The exact syntax depends on your environment - refer to its documentation for autostart and keybinding configuration.

Select garbled text, press your hotkey - and it's fixed.

### Direction Detection

keycast counts characters belonging to each layout and translates toward the minority:

- `ghbdsn` (Latin) > detects mostly Faroese characters > converts to Ukrainian `привіт`
- `руддщ` (Cyrillic) > detects mostly Ukrainian characters > converts to Faroese `hello`
- Mixed text is resolved by majority vote

## Supported Layouts

Currently supports **Faroese <> Ukrainian**, mapped from real `wev` key event data.

| Key position | Faroese | Ukrainian |
|---|---|---|
| Top row | q w e r t y u i o p å ¨ | й ц у к е н г ш щ з х ї |
| Home row | a s d f g h j k l æ ø | ф і в а п р о л д ж є |
| Bottom row | z x c v b n m | я ч с м и т ь б ю |
| Extra key | ' (apostrophe) | ґ (ghe with upturn) |

Dead keys (Faroese only):

- `´` (acute) > á é í ó ú ý / Á É Í Ó Ú Ý
- `¨` (diaeresis) > ä ë ï ö ü ÿ / Ä Ë Ï Ö Ü Ÿ
