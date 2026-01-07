//! QR Share - Minimal Zellij plugin for QR code token display.
//!
//! Opens a floating pane, generates a web login token,
//! and displays it as a scannable QR code.

mod qr_display;

use qr_display::generate_qr_lines;
use std::collections::BTreeMap;
use zellij_tile::prelude::*;

// Layout constants
const MIN_ROWS_FOR_QR: usize = 20;
const QR_SPACING: usize = 2;

#[derive(Debug, Default)]
struct QrSharePlugin {
    rows: usize,
    cols: usize,
    screen: Screen,
    error: Option<String>,
    permissions_granted: bool,
}

#[derive(Debug, Clone, Default)]
enum Screen {
    #[default]
    WaitingForPermissions,
    Token(String),
}

register_plugin!(QrSharePlugin);

impl ZellijPlugin for QrSharePlugin {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        subscribe(&[EventType::Key, EventType::PermissionRequestResult, EventType::Timer]);
        request_permission(&[
            PermissionType::ChangeApplicationState,
            PermissionType::StartWebServer,
        ]);
        // Timer fallback for cached permissions (no PermissionRequestResult event in that case)
        set_timeout(0.1);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::PermissionRequestResult(status) => {
                if status == PermissionStatus::Granted {
                    self.permissions_granted = true;
                    self.on_permissions_granted();
                } else {
                    self.error = Some("Permission denied".to_string());
                }
                true
            }
            Event::Timer(_) => {
                // Handle cached permissions (no PermissionRequestResult event fires)
                if matches!(self.screen, Screen::WaitingForPermissions) && self.error.is_none() {
                    self.on_permissions_granted();
                }
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.rows = rows;
        self.cols = cols;

        // Render based on current screen state - NO API calls here
        match &self.screen {
            Screen::WaitingForPermissions => self.render_waiting_screen(),
            Screen::Token(token) => self.render_token_screen(token.clone()),
        }

        if let Some(error) = &self.error {
            self.render_error(error);
        }
    }
}

impl QrSharePlugin {
    fn on_permissions_granted(&mut self) {
        rename_plugin_pane(get_plugin_ids().plugin_id, "QR Token");
        self.create_token();
    }

    fn handle_key(&mut self, key: KeyWithModifier) -> bool {
        match key.bare_key {
            BareKey::Esc if key.has_no_modifiers() => {
                close_self();
                false
            }
            _ => false,
        }
    }

    fn create_token(&mut self) {
        match generate_web_login_token(None) {
            Ok(token) => self.screen = Screen::Token(token),
            Err(e) => self.error = Some(e),
        }
    }

    fn render_waiting_screen(&self) {
        let msg = "Waiting for permissions...";
        let x = self.cols.saturating_sub(msg.len()) / 2;
        let y = self.rows / 2;
        print_text_with_coordinates(Text::new(msg), x, y, None, None);
    }

    fn render_token_screen(&self, token: String) {
        // Try to render QR code
        let qr_height = self.render_qr(&token);

        // Calculate position for text content
        let token_label = "Token: ";
        let token_line = format!("{}{}", token_label, token);
        let info_line = "Scan QR or copy token to log in";
        let esc_line = "<Esc> Close";

        let content_height = 5; // token + blank + info + blank + esc
        let width = token_line.len().max(info_line.len());
        let x = self.cols.saturating_sub(width) / 2;

        let y = if qr_height > 0 {
            // Position below QR
            let combined = qr_height + content_height;
            let start = self.rows.saturating_sub(combined) / 2;
            start + qr_height
        } else {
            // Center without QR
            self.rows.saturating_sub(content_height) / 2
        };

        print_text_with_coordinates(
            Text::new(&token_line).color_range(2, ..token_label.len()),
            x,
            y,
            None,
            None,
        );
        print_text_with_coordinates(
            Text::new(info_line).color_range(0, ..),
            x,
            y + 2,
            None,
            None,
        );
        print_text_with_coordinates(
            Text::new(esc_line).color_range(3, ..=4),
            x,
            y + 4,
            None,
            None,
        );
    }

    fn render_qr(&self, token: &str) -> usize {
        if self.rows < MIN_ROWS_FOR_QR {
            return 0;
        }

        match generate_qr_lines(token) {
            Ok(lines) => {
                let qr_height = lines.len();
                let content_height = 5;
                let total_needed = qr_height + QR_SPACING + content_height;

                if self.rows < total_needed {
                    return 0;
                }

                let combined = qr_height + QR_SPACING + content_height;
                let start_y = self.rows.saturating_sub(combined) / 2;

                for (i, line) in lines.iter().enumerate() {
                    let line_width = line.chars().count();
                    let qr_x = self.cols.saturating_sub(line_width) / 2;
                    print_text_with_coordinates(
                        Text::new(line),
                        qr_x,
                        start_y + i,
                        None,
                        None,
                    );
                }

                qr_height + QR_SPACING
            }
            Err(_) => 0,
        }
    }

    fn render_error(&self, error: &str) {
        let x = self.cols.saturating_sub(error.len()) / 2;
        let y = self.rows.saturating_sub(2);
        print_text_with_coordinates(
            Text::new(error).color_range(3, ..),
            x,
            y,
            None,
            None,
        );
    }
}
