// Settings app - WiFi, Timezone, Time Adjust, Language Selector, and System Controls
// Uses responsive QWERTY keyboard for text input
// Bilingual support: Español / English

use embedded_graphics::geometry::Point as EgPoint;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::text::{Alignment, Text};

use crate::peripherals::wifi::{WifiConfig, WifiState};
use crate::ui::qwerty_keyboard::QwertyKeyboard;
use crate::ui::theme::{self, Language};

#[derive(Clone, Copy, PartialEq)]
enum SettingsField {
    None,
    Ssid,
    Password,
}

pub struct SettingsApp {
    pub wifi_config: WifiConfig,
    pub wifi_state: WifiState,
    pub keyboard: QwertyKeyboard,
    pub timezone_offset: i8, // default -6 for UTC-6
    pub hour_adjust_pending: i8,
    pub sync_ntp_requested: bool,
    pub wifi_connect_requested: bool,
    pub exit_requested: bool,
    pub reboot_requested: bool,
    pub language: Language,
    active_field: SettingsField,
    editing: bool,
}

impl SettingsApp {
    pub fn new() -> Self {
        Self {
            wifi_config: WifiConfig::new(),
            wifi_state: WifiState::Disconnected,
            keyboard: QwertyKeyboard::new(),
            timezone_offset: -6,
            hour_adjust_pending: 0,
            sync_ntp_requested: false,
            wifi_connect_requested: false,
            exit_requested: false,
            reboot_requested: false,
            language: Language::Spanish,
            active_field: SettingsField::None,
            editing: false,
        }
    }

    pub fn set_initial_creds(&mut self, ssid: &str, pass: &str) {
        if !ssid.is_empty() {
            self.wifi_config.set_ssid(ssid);
        }
        if !pass.is_empty() {
            self.wifi_config.set_password(pass);
        }
    }

    /// Handle tap at screen position. Returns true if consumed.
    pub fn handle_tap(&mut self, x: u16, y: u16) -> bool {
        let xi = x as i32;
        let yi = y as i32;

        // 1. If Keyboard is active, forward taps to it
        if self.keyboard.is_active() {
            if self.keyboard.handle_tap(x, y) {
                // Live sync text into active field
                match self.active_field {
                    SettingsField::Ssid => self.wifi_config.set_ssid(self.keyboard.get_text()),
                    SettingsField::Password => self.wifi_config.set_password(self.keyboard.get_text()),
                    SettingsField::None => {}
                }
                if self.keyboard.is_done() {
                    self.keyboard.clear_done();
                    self.editing = false;
                    self.active_field = SettingsField::None;
                }
                return true;
            }
            if yi < 170 {
                self.keyboard.hide();
                self.editing = false;
                self.active_field = SettingsField::None;
                return true;
            }
            return true;
        }

        // 2. Back button "< APPS" at top left (x: 10..100, y: 10..48)
        if xi >= 10 && xi <= 100 && yi >= 10 && yi <= 48 {
            self.exit_requested = true;
            return true;
        }

        // 3. Language Selector Pills [ ES ] and [ EN ] at top right (y: 10..48)
        // [ ES ] at x: 285..335
        if yi >= 10 && yi <= 48 && xi >= 285 && xi <= 335 {
            self.language = Language::Spanish;
            return true;
        }
        // [ EN ] at x: 345..395
        if yi >= 10 && yi <= 48 && xi >= 345 && xi <= 395 {
            self.language = Language::English;
            return true;
        }

        // 4. SSID field tap (y: 55..98, x: 15..395)
        if yi >= 55 && yi <= 98 && xi >= 15 && xi <= 395 {
            self.active_field = SettingsField::Ssid;
            self.keyboard.clear_text();
            self.keyboard.set_text(self.wifi_config.ssid_str());
            self.keyboard.show();
            self.editing = true;
            return true;
        }

        // 5. Password field tap (y: 102..145, x: 15..395)
        if yi >= 102 && yi <= 145 && xi >= 15 && xi <= 395 {
            self.active_field = SettingsField::Password;
            self.keyboard.clear_text();
            self.keyboard.set_text(self.wifi_config.pass_str());
            self.keyboard.show();
            self.editing = true;
            return true;
        }

        // 6. Connect WiFi button (y: 150..188, x: 15..395)
        if yi >= 150 && yi <= 188 && xi >= 15 && xi <= 395 {
            if self.wifi_state == WifiState::Connected {
                self.wifi_state = WifiState::Disconnected;
            } else {
                self.wifi_state = WifiState::Connecting;
                self.wifi_connect_requested = true;
            }
            return true;
        }

        // 7. Timezone adjustment buttons (y: 218..264)
        // [-] at x: 275..325
        if yi >= 218 && yi <= 264 && xi >= 275 && xi <= 325 {
            if self.timezone_offset > -12 {
                self.timezone_offset -= 1;
                self.hour_adjust_pending -= 1;
            }
            return true;
        }
        // [+] at x: 335..385
        if yi >= 218 && yi <= 264 && xi >= 335 && xi <= 385 {
            if self.timezone_offset < 14 {
                self.timezone_offset += 1;
                self.hour_adjust_pending += 1;
            }
            return true;
        }

        // 8. Manual Hour adjust / NTP sync buttons (y: 272..316)
        // [-1h] at x: 195..250
        if yi >= 272 && yi <= 316 && xi >= 195 && xi <= 250 {
            self.hour_adjust_pending -= 1;
            return true;
        }
        // [+1h] at x: 258..315
        if yi >= 272 && yi <= 316 && xi >= 258 && xi <= 315 {
            self.hour_adjust_pending += 1;
            return true;
        }
        // [NTP] at x: 325..395
        if yi >= 272 && yi <= 316 && xi >= 325 && xi <= 395 {
            self.sync_ntp_requested = true;
            return true;
        }

        // 9. Reboot button (y: 335..380, x: 15..395)
        if yi >= 335 && yi <= 380 && xi >= 15 && xi <= 395 {
            self.reboot_requested = true;
            return true;
        }

        false
    }

    pub fn update(&mut self, dt_ms: u32) {
        self.keyboard.update(dt_ms);
    }

    pub fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        // Pure AMOLED Black Background
        let _ = Rectangle::new(EgPoint::zero(), Size::new(410, 502))
            .into_styled(PrimitiveStyle::with_fill(theme::COLOR_BG))
            .draw(d);

        let style_title = MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_PRIMARY);
        let style_label = MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_MUTED);
        let style_value = MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_PRIMARY);
        let style_green = MonoTextStyle::new(&FONT_10X20, theme::COLOR_ACCENT_GREEN);
        let style_btn = MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_PRIMARY);

        // Header with Back button
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(10, 10), Size::new(85, 36)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_SURFACE_MID))
        .draw(d);
        let _ = Text::with_alignment("< APPS", EgPoint::new(52, 33), style_btn, Alignment::Center).draw(d);

        let title_text = match self.language {
            Language::Spanish => "AJUSTES",
            Language::English => "SETTINGS",
        };
        let _ = Text::with_alignment(title_text, EgPoint::new(185, 33), style_title, Alignment::Center).draw(d);

        // Language Pills [ ES ] and [ EN ]
        let is_es = self.language == Language::Spanish;
        let es_bg = if is_es { theme::COLOR_TEXT_PRIMARY } else { theme::COLOR_SURFACE_MID };
        let es_fg = if is_es { theme::COLOR_BG } else { theme::COLOR_TEXT_SECONDARY };
        let en_bg = if !is_es { theme::COLOR_TEXT_PRIMARY } else { theme::COLOR_SURFACE_MID };
        let en_fg = if !is_es { theme::COLOR_BG } else { theme::COLOR_TEXT_SECONDARY };

        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(285, 12), Size::new(48, 32)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_fill(es_bg))
        .draw(d);
        let _ = Text::with_alignment("ES", EgPoint::new(309, 33), MonoTextStyle::new(&FONT_10X20, es_fg), Alignment::Center).draw(d);

        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(345, 12), Size::new(48, 32)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_fill(en_bg))
        .draw(d);
        let _ = Text::with_alignment("EN", EgPoint::new(369, 33), MonoTextStyle::new(&FONT_10X20, en_fg), Alignment::Center).draw(d);

        // === 1. WiFi SSID ===
        let ssid_bg = if self.active_field == SettingsField::Ssid && self.editing {
            theme::COLOR_SURFACE_MID
        } else {
            theme::COLOR_CARD_BG
        };
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(15, 55), Size::new(380, 42)),
            Size::new(8, 8),
        )
        .into_styled(PrimitiveStyle::with_fill(ssid_bg))
        .draw(d);
        let _ = Text::new("WiFi:", EgPoint::new(25, 82), style_label).draw(d);
        let ssid = self.wifi_config.ssid_str();
        let placeholder_tap = match self.language {
            Language::Spanish => "(tocar para escribir)",
            Language::English => "(tap to enter)",
        };
        let ssid_display = if ssid.is_empty() { placeholder_tap } else { ssid };
        let _ = Text::new(ssid_display, EgPoint::new(90, 82), style_value).draw(d);

        // === 2. WiFi Password ===
        let pass_bg = if self.active_field == SettingsField::Password && self.editing {
            theme::COLOR_SURFACE_MID
        } else {
            theme::COLOR_CARD_BG
        };
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(15, 102), Size::new(380, 42)),
            Size::new(8, 8),
        )
        .into_styled(PrimitiveStyle::with_fill(pass_bg))
        .draw(d);
        let pass_label = match self.language {
            Language::Spanish => "Pass:",
            Language::English => "Pass:",
        };
        let _ = Text::new(pass_label, EgPoint::new(25, 129), style_label).draw(d);
        let pass_len = self.wifi_config.pass_len;
        let pass_display = if pass_len > 0 { "********" } else { placeholder_tap };
        let _ = Text::new(pass_display, EgPoint::new(90, 129), style_value).draw(d);

        // === 3. Connect Button ===
        let (btn_color, btn_text) = match (self.wifi_state, self.language) {
            (WifiState::Disconnected, Language::Spanish) => (theme::COLOR_SURFACE_MID, "CONECTAR WIFI"),
            (WifiState::Disconnected, Language::English) => (theme::COLOR_SURFACE_MID, "CONNECT WIFI"),
            (WifiState::Connecting, Language::Spanish) => (theme::COLOR_ACCENT_AMBER, "CONECTANDO..."),
            (WifiState::Connecting, Language::English) => (theme::COLOR_ACCENT_AMBER, "CONNECTING..."),
            (WifiState::Connected, Language::Spanish) => (theme::COLOR_ACCENT_GREEN, "CONECTADO"),
            (WifiState::Connected, Language::English) => (theme::COLOR_ACCENT_GREEN, "CONNECTED"),
            (WifiState::Error, Language::Spanish) => (theme::COLOR_ACCENT_RED, "ERROR AL CONECTAR"),
            (WifiState::Error, Language::English) => (theme::COLOR_ACCENT_RED, "CONNECTION ERROR"),
        };
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(15, 150), Size::new(380, 38)),
            Size::new(8, 8),
        )
        .into_styled(PrimitiveStyle::with_fill(btn_color))
        .draw(d);
        let _ = Text::with_alignment(btn_text, EgPoint::new(205, 175), style_btn, Alignment::Center).draw(d);

        // === 4. Timezone Section ===
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(15, 218), Size::new(380, 46)),
            Size::new(8, 8),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_CARD_BG))
        .draw(d);

        let mut tz_buf = [0u8; 32];
        let tz_str = fmt_timezone(&mut tz_buf, self.timezone_offset, self.language);
        let _ = Text::new(tz_str, EgPoint::new(25, 247), style_value).draw(d);

        // [-] button
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(275, 223), Size::new(48, 36)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_SURFACE_MID))
        .draw(d);
        let _ = Text::with_alignment("-", EgPoint::new(299, 247), style_btn, Alignment::Center).draw(d);

        // [+] button
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(335, 223), Size::new(48, 36)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_SURFACE_MID))
        .draw(d);
        let _ = Text::with_alignment("+", EgPoint::new(359, 247), style_btn, Alignment::Center).draw(d);

        // === 5. Hour Adjustment & NTP ===
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(15, 272), Size::new(380, 46)),
            Size::new(8, 8),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_CARD_BG))
        .draw(d);
        let adj_label = match self.language {
            Language::Spanish => "Ajuste:",
            Language::English => "Adjust:",
        };
        let _ = Text::new(adj_label, EgPoint::new(25, 301), style_label).draw(d);

        // [-1h]
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(195, 277), Size::new(55, 36)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_SURFACE_MID))
        .draw(d);
        let _ = Text::with_alignment("-1h", EgPoint::new(222, 301), style_btn, Alignment::Center).draw(d);

        // [+1h]
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(260, 277), Size::new(55, 36)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_SURFACE_MID))
        .draw(d);
        let _ = Text::with_alignment("+1h", EgPoint::new(287, 301), style_btn, Alignment::Center).draw(d);

        // [NTP]
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(325, 277), Size::new(65, 36)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_ACCENT_GREEN))
        .draw(d);
        let _ = Text::with_alignment("NTP", EgPoint::new(357, 301), MonoTextStyle::new(&FONT_10X20, theme::COLOR_BG), Alignment::Center).draw(d);

        // === 6. Reboot Button ===
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(15, 335), Size::new(380, 42)),
            Size::new(8, 8),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_ACCENT_RED))
        .draw(d);
        let reboot_text = match self.language {
            Language::Spanish => "REINICIAR RELOJ",
            Language::English => "REBOOT WATCH",
        };
        let _ = Text::with_alignment(reboot_text, EgPoint::new(205, 362), style_btn, Alignment::Center).draw(d);

        // Footer info
        let footer_text = match self.language {
            Language::Spanish => "Teclado QWERTY Activo",
            Language::English => "QWERTY Keyboard Active",
        };
        let _ = Text::with_alignment("Waveshare ESP32-S3 Watch RS", EgPoint::new(205, 420), style_label, Alignment::Center).draw(d);
        let _ = Text::with_alignment(footer_text, EgPoint::new(205, 445), style_green, Alignment::Center).draw(d);

        // Draw keyboard overlay if active
        if self.keyboard.is_active() {
            let prompt = match (self.active_field, self.language) {
                (SettingsField::Ssid, Language::Spanish) => "Ingresar WiFi SSID:",
                (SettingsField::Ssid, Language::English) => "Enter WiFi SSID:",
                (SettingsField::Password, Language::Spanish) => "Ingresar Contraseña:",
                (SettingsField::Password, Language::English) => "Enter Password:",
                _ => "Escribir / Type:",
            };
            let _ = Rectangle::new(EgPoint::new(0, 135), Size::new(410, 36))
                .into_styled(PrimitiveStyle::with_fill(theme::COLOR_BG))
                .draw(d);
            let _ = Text::new(prompt, EgPoint::new(20, 160), style_title).draw(d);

            self.keyboard.render(d);
        }
    }
}

fn fmt_timezone<'a>(buf: &'a mut [u8; 32], offset: i8, lang: Language) -> &'a str {
    let sign = if offset >= 0 { '+' } else { '-' };
    let abs_val = offset.unsigned_abs();
    let prefix = match lang {
        Language::Spanish => "Zona",
        Language::English => "Zone",
    };
    let s = alloc::format!("{}: UTC {}{:02}:00", prefix, sign, abs_val);
    let bytes = s.as_bytes();
    let len = bytes.len().min(buf.len());
    buf[..len].copy_from_slice(&bytes[..len]);
    core::str::from_utf8(&buf[..len]).unwrap_or("UTC")
}
