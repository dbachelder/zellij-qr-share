# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build --release    # Build WASM plugin (outputs to target/wasm32-wasip1/release/qr-share.wasm)
cargo clippy             # Lint
```

## Deploy for Testing

```bash
/bin/cp -f target/wasm32-wasip1/release/qr-share.wasm ~/.config/zellij/plugins/
rm -rf ~/.cache/zellij/                        # Clear plugin cache
pkill -9 -f "zellij --server"                  # Kill existing servers
zellij                                         # Start fresh
```

Then trigger via keybinding (see README.md for config).

## Architecture

This is a Zellij plugin compiled to WASM (`wasm32-wasip1` target configured in `.cargo/config.toml`).

- **src/main.rs** - Plugin entry point implementing `ZellijPlugin` trait. State machine: `WaitingForPermissions` → `Token` (QR display). Uses `zellij_tile::prelude` for rendering via `print_text_with_coordinates()`.
- **src/qr_display.rs** - QR generation using `qrcode` crate with `unicode::Dense1x2` renderer for terminal display.

Key Zellij APIs used:
- `generate_web_login_token(None)` - Creates web auth token (requires `StartWebServer` permission)
- `request_permission()` / `PermissionRequestResult` - Permission flow
- `subscribe(&[EventType::Key, ...])` - Event subscription

## Zellij Plugin Patterns

**CRITICAL: Never call APIs in `render()`**

The `render()` function must ONLY draw to the screen. Making API calls (like `generate_web_login_token()`) or modifying significant state in `render()` will corrupt Zellij's internal state and cause:
- Missing status bar / broken UI
- Controls stop working (close pane, etc.)
- QR not appearing until second session opens

**Correct pattern:**
```rust
fn load(&mut self, _config: BTreeMap<String, String>) {
    request_permission(&[...]);  // Request permissions here
    subscribe(&[EventType::PermissionRequestResult, ...]);
}

fn update(&mut self, event: Event) -> bool {
    match event {
        Event::PermissionRequestResult(PermissionStatus::Granted) => {
            self.do_api_call();  // API calls in update(), after permission granted
            true
        }
        _ => false,
    }
}

fn render(&mut self, rows: usize, cols: usize) {
    // ONLY drawing here - no API calls, no state changes
    match &self.screen {
        Screen::Waiting => self.draw_waiting(),
        Screen::Ready(data) => self.draw_data(data),
    }
}
```

**Permission flow:**
1. `load()` → `request_permission()` - triggers permission dialog
2. User grants → `update()` receives `PermissionRequestResult::Granted`
3. `update()` calls APIs and transitions state
4. `render()` draws based on current state

## Known Issues

**Double-loading plugins breaks state:** Running `zellij plugin -- file:...` from the command line while the plugin is already loaded (e.g., via keybinding) causes state corruption. The plugin gets stuck on "Waiting for permissions..." because events don't fire correctly for duplicate instances.

**Solution:** Always use a keybinding with `LaunchOrFocusPlugin` instead of manual `zellij plugin` commands. This ensures only one instance runs.
