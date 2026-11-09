// QWERTY Touchscreen Keyboard for Waveshare Watch (410x502 AMOLED)
// Responsive, full-width QWERTY layout with Shift, Symbols (?123), Space, Del, and OK.

use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::text::{Alignment, Text};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KeyboardMode {
    Lower,
    Upper,
    Symbols,
    ExtraSymbols,
}

pub struct QwertyKeyboard {
    text: [u8; 128],
    text_len: usize,
    mode: KeyboardMode,
    active: bool,
    caps_locked: bool,
    last_tapped_key: Option<char>,
    feedback_timer: u32,
    done_requested: bool,
}

const ROW0_LOWER: &[char] = &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'];
const ROW0_UPPER: &[char] = &['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'];
const ROW0_SYM:   &[char] = &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];
const ROW0_EXTRA: &[char] = &['[', ']', '{', '}', '#', '%', '^', '*', '+', '='];

const ROW1_LOWER: &[char] = &['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'];
const ROW1_UPPER: &[char] = &['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'];
const ROW1_SYM:   &[char] = &['@', '#', '$', '%', '&', '*', '-', '_', '/'];
const ROW1_EXTRA: &[char] = &['_', '\\', '|', '~', '<', '>', '$', '`', ';'];

const ROW2_LOWER: &[char] = &['z', 'x', 'c', 'v', 'b', 'n', 'm'];
const ROW2_UPPER: &[char] = &['Z', 'X', 'C', 'V', 'B', 'N', 'M'];
const ROW2_SYM:   &[char] = &['(', ')', '"', '\'', ':', ';', '!', '?'];
const ROW2_EXTRA: &[char] = &['<', '>', '{', '}', '!', '?', ':', ';'];

impl QwertyKeyboard {
    pub fn new() -> Self {
        Self {
            text: [0; 128],
            text_len: 0,
            mode: KeyboardMode::Lower,
            active: false,
            caps_locked: false,
            last_tapped_key: None,
            feedback_timer: 0,
            done_requested: false,
        }
    }

    pub fn show(&mut self) {
        self.active = true;
        self.done_requested = false;
    }

    pub fn hide(&mut self) {
        self.active = false;
        self.last_tapped_key = None;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_done(&self) -> bool {
        self.done_requested
    }

    pub fn clear_done(&mut self) {
        self.done_requested = false;
    }

    pub fn get_text(&self) -> &str {
        core::str::from_utf8(&self.text[..self.text_len]).unwrap_or("")
    }

    pub fn set_text(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len().min(self.text.len() - 1);
        self.text[..len].copy_from_slice(&bytes[..len]);
        self.text_len = len;
    }

    pub fn clear_text(&mut self) {
        self.text_len = 0;
    }

    pub fn add_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        for &b in encoded.as_bytes() {
            if self.text_len < self.text.len() - 1 {
                self.text[self.text_len] = b;
                self.text_len += 1;
            }
        }
        self.last_tapped_key = Some(c);
        self.feedback_timer = 200; // 200ms visual feedback
    }

    pub fn delete_last(&mut self) {
        if self.text_len > 0 {
            self.text_len -= 1;
            // Pop until valid utf8 boundary
            while self.text_len > 0 && (self.text[self.text_len] & 0xC0) == 0x80 {
                self.text_len -= 1;
            }
        }
        self.feedback_timer = 200;
    }

    pub fn update(&mut self, dt_ms: u32) -> bool {
        if self.feedback_timer > 0 {
            if dt_ms >= self.feedback_timer {
                self.feedback_timer = 0;
                self.last_tapped_key = None;
            } else {
                self.feedback_timer -= dt_ms;
            }
            return true;
        }
        false
    }

    /// Handles touch coordinates. Returns true if consumed.
    pub fn handle_tap(&mut self, x: u16, y: u16) -> bool {
        if !self.active {
            return false;
        }

        let xi = x as i32;
        let yi = y as i32;

        // 1. Text preview area (y: 175..230)
        if yi >= 175 && yi <= 230 {
            // Clear button tap at right (x: 350..400)
            if xi >= 350 && xi <= 405 {
                self.clear_text();
                return true;
            }
            return true; // consume tap on preview
        }

        // Row 0 (y: 236..286): 10 keys
        if yi >= 236 && yi <= 286 {
            let row_chars = match self.mode {
                KeyboardMode::Lower => ROW0_LOWER,
                KeyboardMode::Upper => ROW0_UPPER,
                KeyboardMode::Symbols => ROW0_SYM,
                KeyboardMode::ExtraSymbols => ROW0_EXTRA,
            };
            for (i, &c) in row_chars.iter().enumerate() {
                let kx = 2 + (i as i32) * 41;
                if xi >= kx && xi < kx + 38 {
                    self.add_char(c);
                    if self.mode == KeyboardMode::Upper && !self.caps_locked {
                        self.mode = KeyboardMode::Lower;
                    }
                    return true;
                }
            }
            return true;
        }

        // Row 1 (y: 290..340): 9 keys
        if yi >= 290 && yi <= 340 {
            let row_chars = match self.mode {
                KeyboardMode::Lower => ROW1_LOWER,
                KeyboardMode::Upper => ROW1_UPPER,
                KeyboardMode::Symbols => ROW1_SYM,
                KeyboardMode::ExtraSymbols => ROW1_EXTRA,
            };
            for (i, &c) in row_chars.iter().enumerate() {
                let kx = 5 + (i as i32) * 45;
                if xi >= kx && xi < kx + 41 {
                    self.add_char(c);
                    if self.mode == KeyboardMode::Upper && !self.caps_locked {
                        self.mode = KeyboardMode::Lower;
                    }
                    return true;
                }
            }
            return true;
        }

        // Row 2 (y: 344..394): [Shift] + letters + [Del]
        if yi >= 344 && yi <= 394 {
            // Shift / Mode button at left (x: 8..62)
            if xi >= 8 && xi <= 62 {
                match self.mode {
                    KeyboardMode::Lower => self.mode = KeyboardMode::Upper,
                    KeyboardMode::Upper => self.mode = KeyboardMode::Lower,
                    KeyboardMode::Symbols => self.mode = KeyboardMode::ExtraSymbols,
                    KeyboardMode::ExtraSymbols => self.mode = KeyboardMode::Symbols,
                }
                return true;
            }

            // Backspace / Delete at right (x: 345..405)
            if xi >= 345 && xi <= 405 {
                self.delete_last();
                return true;
            }

            // Letter keys in between (7 or 8 keys)
            let row_chars = match self.mode {
                KeyboardMode::Lower => ROW2_LOWER,
                KeyboardMode::Upper => ROW2_UPPER,
                KeyboardMode::Symbols => ROW2_SYM,
                KeyboardMode::ExtraSymbols => ROW2_EXTRA,
            };
            let start_x = 68;
            for (i, &c) in row_chars.iter().enumerate() {
                let kx = start_x + (i as i32) * 39;
                if xi >= kx && xi < kx + 36 {
                    self.add_char(c);
                    if self.mode == KeyboardMode::Upper && !self.caps_locked {
                        self.mode = KeyboardMode::Lower;
                    }
                    return true;
                }
            }
            return true;
        }

        // Row 3 (y: 398..452): [?123] [ , ] [ SPACE ] [ . ] [ OK ]
        if yi >= 398 && yi <= 452 {
            // Mode switch (?123 / ABC) at x: 12..74
            if xi >= 12 && xi <= 74 {
                self.mode = match self.mode {
                    KeyboardMode::Lower | KeyboardMode::Upper => KeyboardMode::Symbols,
                    KeyboardMode::Symbols | KeyboardMode::ExtraSymbols => KeyboardMode::Lower,
                };
                return true;
            }

            // Comma / symbol at x: 80..118
            if xi >= 80 && xi <= 118 {
                let c = match self.mode {
                    KeyboardMode::Symbols | KeyboardMode::ExtraSymbols => '_',
                    _ => ',',
                };
                self.add_char(c);
                return true;
            }

            // Spacebar at x: 124..276
            if xi >= 124 && xi <= 276 {
                self.add_char(' ');
                return true;
            }

            // Dot / symbol at x: 282..320
            if xi >= 282 && xi <= 320 {
                let c = match self.mode {
                    KeyboardMode::Symbols | KeyboardMode::ExtraSymbols => '=',
                    _ => '.',
                };
                self.add_char(c);
                return true;
            }

            // Done / OK at x: 326..400
            if xi >= 326 && xi <= 400 {
                self.done_requested = true;
                self.hide();
                return true;
            }

            return true;
        }

        // Tap outside keyboard area closes it
        if yi < 170 {
            self.hide();
            return false;
        }

        true
    }

    pub fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        if !self.active {
            return;
        }

        // Background shadow / dim over upper content
        let _ = Rectangle::new(Point::new(0, 170), Size::new(410, 332))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(d);

        // 1. Text Preview Box (y: 175..228) - Pure Black with subtle gray border
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(8, 175), Size::new(394, 52)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(d);

        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(8, 175), Size::new(394, 52)),
            Size::new(6, 6),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(8, 16, 8), 1))
        .draw(d);

        let style_white = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let style_dim = MonoTextStyle::new(&FONT_10X20, Rgb565::CSS_GRAY);

        // Render current typed string (with horizontal scrolling if long)
        let txt = self.get_text();
        let display_txt = if txt.len() > 30 {
            &txt[txt.len() - 30..]
        } else {
            txt
        };
        let _ = Text::new(display_txt, Point::new(18, 207), style_white).draw(d);

        // Cursor in Pure White
        let cursor_x = 18 + (display_txt.len() as i32) * 10;
        let _ = Rectangle::new(Point::new(cursor_x, 188), Size::new(2, 24))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(d);

        // Clear button [ X ]
        let _ = Text::with_alignment("X", Point::new(385, 207), style_dim, Alignment::Center).draw(d);

        // Neutral High-Contrast Grayscale Keys: Pure Black background, Charcoal keys, White text (NO BLUE)
        let key_bg = Rgb565::new(3, 7, 3);          // Neutral dark gray (#181818)
        let key_pressed_bg = Rgb565::new(14, 28, 14); // Light neutral gray on tap (#707070)
        let key_border = Rgb565::new(7, 14, 7);      // Subtle border (#383838)
        let key_special_bg = Rgb565::new(6, 12, 6);  // Neutral medium gray (#303030)
        let ok_bg = Rgb565::new(8, 16, 8);           // Neutral dark gray

        // 2. Row 0 (10 keys)
        let row0 = match self.mode {
            KeyboardMode::Lower => ROW0_LOWER,
            KeyboardMode::Upper => ROW0_UPPER,
            KeyboardMode::Symbols => ROW0_SYM,
            KeyboardMode::ExtraSymbols => ROW0_EXTRA,
        };
        for (i, &c) in row0.iter().enumerate() {
            let kx = 2 + (i as i32) * 41;
            let bg = if self.last_tapped_key == Some(c) { key_pressed_bg } else { key_bg };
            let fg = Rgb565::WHITE;
            let _ = RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(kx, 236), Size::new(38, 48)),
                Size::new(5, 5),
            )
            .into_styled(PrimitiveStyle::with_fill(bg))
            .draw(d);

            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            let _ = Text::with_alignment(
                s,
                Point::new(kx + 19, 267),
                MonoTextStyle::new(&FONT_10X20, fg),
                Alignment::Center,
            )
            .draw(d);
        }

        // 3. Row 1 (9 keys)
        let row1 = match self.mode {
            KeyboardMode::Lower => ROW1_LOWER,
            KeyboardMode::Upper => ROW1_UPPER,
            KeyboardMode::Symbols => ROW1_SYM,
            KeyboardMode::ExtraSymbols => ROW1_EXTRA,
        };
        for (i, &c) in row1.iter().enumerate() {
            let kx = 5 + (i as i32) * 45;
            let bg = if self.last_tapped_key == Some(c) { key_pressed_bg } else { key_bg };
            let fg = Rgb565::WHITE;
            let _ = RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(kx, 290), Size::new(41, 48)),
                Size::new(5, 5),
            )
            .into_styled(PrimitiveStyle::with_fill(bg))
            .draw(d);

            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            let _ = Text::with_alignment(
                s,
                Point::new(kx + 20, 321),
                MonoTextStyle::new(&FONT_10X20, fg),
                Alignment::Center,
            )
            .draw(d);
        }

        // 4. Row 2 ([Shift] + 7/8 keys + [Del])
        // Shift key at left
        let shift_label = match self.mode {
            KeyboardMode::Lower => "SHF",
            KeyboardMode::Upper => "CAP",
            KeyboardMode::Symbols => "#+=",
            KeyboardMode::ExtraSymbols => "123",
        };
        let shift_bg = match self.mode {
            KeyboardMode::Upper => Rgb565::WHITE,
            _ => key_special_bg,
        };
        let shift_fg = match self.mode {
            KeyboardMode::Upper => Rgb565::BLACK,
            _ => Rgb565::WHITE,
        };
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(8, 344), Size::new(54, 48)),
            Size::new(5, 5),
        )
        .into_styled(PrimitiveStyle::with_fill(shift_bg))
        .draw(d);
        let _ = Text::with_alignment(
            shift_label,
            Point::new(35, 375),
            MonoTextStyle::new(&FONT_10X20, shift_fg),
            Alignment::Center,
        )
        .draw(d);

        // Letter keys
        let row2 = match self.mode {
            KeyboardMode::Lower => ROW2_LOWER,
            KeyboardMode::Upper => ROW2_UPPER,
            KeyboardMode::Symbols => ROW2_SYM,
            KeyboardMode::ExtraSymbols => ROW2_EXTRA,
        };
        let start_x = 68;
        for (i, &c) in row2.iter().enumerate() {
            let kx = start_x + (i as i32) * 39;
            let bg = if self.last_tapped_key == Some(c) { key_pressed_bg } else { key_bg };
            let fg = Rgb565::WHITE;
            let _ = RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(kx, 344), Size::new(36, 48)),
                Size::new(5, 5),
            )
            .into_styled(PrimitiveStyle::with_fill(bg))
            .draw(d);

            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            let _ = Text::with_alignment(
                s,
                Point::new(kx + 18, 375),
                MonoTextStyle::new(&FONT_10X20, fg),
                Alignment::Center,
            )
            .draw(d);
        }

        // Backspace key at right
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(348, 344), Size::new(54, 48)),
            Size::new(5, 5),
        )
        .into_styled(PrimitiveStyle::with_fill(key_special_bg))
        .draw(d);
        let _ = Text::with_alignment(
            "DEL",
            Point::new(375, 375),
            style_white,
            Alignment::Center,
        )
        .draw(d);

        // 5. Row 3: [?123] [ , ] [ SPACE ] [ . ] [ OK ]
        // Mode toggle (?123 / ABC)
        let mode_label = match self.mode {
            KeyboardMode::Lower | KeyboardMode::Upper => "?123",
            KeyboardMode::Symbols | KeyboardMode::ExtraSymbols => "ABC",
        };
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(12, 398), Size::new(62, 48)),
            Size::new(5, 5),
        )
        .into_styled(PrimitiveStyle::with_fill(key_special_bg))
        .draw(d);
        let _ = Text::with_alignment(
            mode_label,
            Point::new(43, 429),
            style_white,
            Alignment::Center,
        )
        .draw(d);

        // Comma / symbol
        let comma_str = match self.mode {
            KeyboardMode::Symbols | KeyboardMode::ExtraSymbols => "_",
            _ => ",",
        };
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(80, 398), Size::new(38, 48)),
            Size::new(5, 5),
        )
        .into_styled(PrimitiveStyle::with_fill(key_bg))
        .draw(d);
        let _ = Text::with_alignment(
            comma_str,
            Point::new(99, 429),
            style_white,
            Alignment::Center,
        )
        .draw(d);

        // SPACE bar
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(124, 398), Size::new(152, 48)),
            Size::new(5, 5),
        )
        .into_styled(PrimitiveStyle::with_fill(key_bg))
        .draw(d);
        let _ = Text::with_alignment(
            "ESPACIO",
            Point::new(200, 429),
            style_dim,
            Alignment::Center,
        )
        .draw(d);

        // Period / symbol
        let dot_str = match self.mode {
            KeyboardMode::Symbols | KeyboardMode::ExtraSymbols => "=",
            _ => ".",
        };
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(282, 398), Size::new(38, 48)),
            Size::new(5, 5),
        )
        .into_styled(PrimitiveStyle::with_fill(key_bg))
        .draw(d);
        let _ = Text::with_alignment(
            dot_str,
            Point::new(301, 429),
            style_white,
            Alignment::Center,
        )
        .draw(d);

        // DONE / OK button
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(326, 398), Size::new(74, 48)),
            Size::new(5, 5),
        )
        .into_styled(PrimitiveStyle::with_fill(ok_bg))
        .draw(d);
        let _ = Text::with_alignment(
            "OK",
            Point::new(363, 429),
            style_white,
            Alignment::Center,
        )
        .draw(d);
    }
}
