// Modern High-Contrast Monochrome / Grayscale UI Theme for Waveshare Watch
// Designed for AMOLED displays: Deep Black (#000000), Crisp White (#FFFFFF), and Neutral Grays.

use alloc::format;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Line, PrimitiveStyle, Rectangle, RoundedRectangle,
};
use embedded_graphics::text::{Alignment, Text};

// === Color Palette Constants (AMOLED High Contrast) ===
pub const COLOR_BG: Rgb565 = Rgb565::BLACK;                 // Pure AMOLED Black (#000000)
pub const COLOR_SURFACE_LOW: Rgb565 = Rgb565::new(2, 4, 2); // Darkest gray (#101010)
pub const COLOR_SURFACE_MID: Rgb565 = Rgb565::new(4, 8, 4); // Medium dark gray (#1E1E1E)
pub const COLOR_CARD_BG: Rgb565 = Rgb565::new(5, 10, 5);    // Elevated Card (#242424)
pub const COLOR_BORDER: Rgb565 = Rgb565::new(8, 16, 8);     // Subtle Border (#3C3C3C)
pub const COLOR_BORDER_BRIGHT: Rgb565 = Rgb565::new(14, 28, 14); // Light border highlight (#686868)
pub const COLOR_TEXT_PRIMARY: Rgb565 = Rgb565::WHITE;       // Pure White (#FFFFFF)
pub const COLOR_TEXT_SECONDARY: Rgb565 = Rgb565::new(26, 52, 26); // Crisp Gray (#CCCCCC)
pub const COLOR_TEXT_MUTED: Rgb565 = Rgb565::new(18, 36, 18);     // Medium Gray (#888888)

// Functional accents
pub const COLOR_ACCENT_GREEN: Rgb565 = Rgb565::new(4, 52, 10);    // Emerald Connected
pub const COLOR_ACCENT_CYAN: Rgb565 = Rgb565::new(0, 50, 30);      // Tech Cyan
pub const COLOR_ACCENT_AMBER: Rgb565 = Rgb565::new(31, 40, 0);     // Charging / Alert
pub const COLOR_ACCENT_RED: Rgb565 = Rgb565::new(28, 6, 6);        // Danger / Low batt

// Metallic Golden Gradient Accents
pub const COLOR_GOLD_BRIGHT: Rgb565 = Rgb565::new(31, 56, 4); // #FFD700 Rich Gold
pub const COLOR_GOLD_MID: Rgb565 = Rgb565::new(26, 42, 2);    // #D4AF37 Classic Gold
pub const COLOR_GOLD_DARK: Rgb565 = Rgb565::new(16, 20, 0);   // #805500 Bronze Shadow

// === Language Support (Español / English) ===
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Language {
    Spanish,
    English,
}

impl Language {
    pub fn toggle(self) -> Self {
        match self {
            Language::Spanish => Language::English,
            Language::English => Language::Spanish,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Language::Spanish => "ES",
            Language::English => "EN",
        }
    }
}

// === Watchface Styles ===
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WatchfaceStyle {
    MinimalDigital = 0,
    AnalogClassic = 1,
    SportTactical = 2,
}

impl WatchfaceStyle {
    pub fn next(self) -> Self {
        match self {
            WatchfaceStyle::MinimalDigital => WatchfaceStyle::AnalogClassic,
            WatchfaceStyle::AnalogClassic => WatchfaceStyle::SportTactical,
            WatchfaceStyle::SportTactical => WatchfaceStyle::MinimalDigital,
        }
    }

    pub fn name(self, lang: Language) -> &'static str {
        match (self, lang) {
            (WatchfaceStyle::MinimalDigital, Language::Spanish) => "Minimal",
            (WatchfaceStyle::MinimalDigital, Language::English) => "Minimal",
            (WatchfaceStyle::AnalogClassic, Language::Spanish) => "Analógico",
            (WatchfaceStyle::AnalogClassic, Language::English) => "Analog",
            (WatchfaceStyle::SportTactical, Language::Spanish) => "Deportivo",
            (WatchfaceStyle::SportTactical, Language::English) => "Tactical",
        }
    }
}

// === Date Formatting in ES and EN ===
fn day_of_week(year: u32, month: u32, day: u32) -> usize {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    ((y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day) % 7) as usize
}

pub fn fmt_date(buf: &mut [u8; 32], day: u8, month: u8, year: u8, lang: Language) -> &str {
    let y = 2000u32 + (year as u32);
    let m = (month as u32).clamp(1, 12);
    let d = (day as u32).clamp(1, 31);
    let dow = day_of_week(y, m, d);

    let weekdays_es = ["DOM", "LUN", "MAR", "MIE", "JUE", "VIE", "SAB"];
    let weekdays_en = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
    let months_es = ["ENE", "FEB", "MAR", "ABR", "MAY", "JUN", "JUL", "AGO", "SEP", "OCT", "NOV", "DIC"];
    let months_en = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];

    let (w_str, m_str) = match lang {
        Language::Spanish => (weekdays_es[dow], months_es[(m - 1) as usize]),
        Language::English => (weekdays_en[dow], months_en[(m - 1) as usize]),
    };

    let formatted = format!("{} {:02} {}", w_str, d, m_str);
    let bytes = formatted.as_bytes();
    let len = bytes.len().min(buf.len());
    buf[..len].copy_from_slice(&bytes[..len]);
    core::str::from_utf8(&buf[..len]).unwrap_or("")
}

// === UI Widget Helpers ===

/// Draws an elevated modern Card container
pub fn draw_card<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    rect: Rectangle,
    corner_radius: u32,
    is_pressed: bool,
) -> Result<(), D::Error> {
    let bg = if is_pressed { Rgb565::new(8, 16, 8) } else { COLOR_CARD_BG };
    let border = if is_pressed { COLOR_TEXT_PRIMARY } else { COLOR_BORDER };

    RoundedRectangle::with_equal_corners(rect, Size::new(corner_radius, corner_radius))
        .into_styled(PrimitiveStyle::with_fill(bg))
        .draw(d)?;

    RoundedRectangle::with_equal_corners(rect, Size::new(corner_radius, corner_radius))
        .into_styled(PrimitiveStyle::with_stroke(border, 1))
        .draw(d)?;

    Ok(())
}

/// Draws a sleek pill button
pub fn draw_pill_button<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    label: &str,
    is_active: bool,
) -> Result<(), D::Error> {
    let rect = Rectangle::new(Point::new(x, y), Size::new(w, h));
    let radius = h / 2;

    let (bg, border, fg) = if is_active {
        (COLOR_TEXT_PRIMARY, COLOR_TEXT_PRIMARY, COLOR_BG)
    } else {
        (COLOR_SURFACE_MID, COLOR_BORDER_BRIGHT, COLOR_TEXT_PRIMARY)
    };

    RoundedRectangle::with_equal_corners(rect, Size::new(radius, radius))
        .into_styled(PrimitiveStyle::with_fill(bg))
        .draw(d)?;

    if !is_active {
        RoundedRectangle::with_equal_corners(rect, Size::new(radius, radius))
            .into_styled(PrimitiveStyle::with_stroke(border, 1))
            .draw(d)?;
    }

    let style = MonoTextStyle::new(&FONT_10X20, fg);
    Text::with_alignment(
        label,
        Point::new(x + (w as i32) / 2, y + (h as i32) / 2 + 6),
        style,
        Alignment::Center,
    )
    .draw(d)?;

    Ok(())
}

/// Draws an iOS / Material style smooth Capsule Switch
pub fn draw_capsule_switch<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    is_on: bool,
) -> Result<(), D::Error> {
    let rect = Rectangle::new(Point::new(x, y), Size::new(w, h));
    let radius = h / 2;
    let track_bg = if is_on { COLOR_TEXT_PRIMARY } else { COLOR_SURFACE_LOW };
    let border = if is_on { COLOR_TEXT_PRIMARY } else { COLOR_BORDER_BRIGHT };

    RoundedRectangle::with_equal_corners(rect, Size::new(radius, radius))
        .into_styled(PrimitiveStyle::with_fill(track_bg))
        .draw(d)?;

    RoundedRectangle::with_equal_corners(rect, Size::new(radius, radius))
        .into_styled(PrimitiveStyle::with_stroke(border, 1))
        .draw(d)?;

    // Knob
    let knob_d = h - 6;
    let knob_x = if is_on {
        x + (w as i32) - (knob_d as i32) - 3
    } else {
        x + 3
    };
    let knob_color = if is_on { COLOR_BG } else { COLOR_TEXT_SECONDARY };

    Circle::new(Point::new(knob_x, y + 3), knob_d)
        .into_styled(PrimitiveStyle::with_fill(knob_color))
        .draw(d)?;

    Ok(())
}

/// Draws a battery pill capsule with progress bar and percent
pub fn draw_battery_status<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    x: i32,
    y: i32,
    pct: u8,
    charging: bool,
) -> Result<(), D::Error> {
    // Battery shell (32px wide x 16px high)
    let shell = Rectangle::new(Point::new(x, y), Size::new(32, 16));
    RoundedRectangle::with_equal_corners(shell, Size::new(3, 3))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_BORDER_BRIGHT, 1))
        .draw(d)?;

    // Positive terminal nipple
    Rectangle::new(Point::new(x + 32, y + 4), Size::new(3, 8))
        .into_styled(PrimitiveStyle::with_fill(COLOR_BORDER_BRIGHT))
        .draw(d)?;

    // Fill level
    let fill_w = ((pct.min(100) as u32) * 26) / 100;
    if fill_w > 0 {
        let fill_color = if charging {
            COLOR_ACCENT_AMBER
        } else if pct <= 20 {
            COLOR_ACCENT_RED
        } else {
            COLOR_TEXT_PRIMARY
        };
        Rectangle::new(Point::new(x + 3, y + 3), Size::new(fill_w, 10))
            .into_styled(PrimitiveStyle::with_fill(fill_color))
            .draw(d)?;
    }

    // Percentage text
    let mut buf = [0u8; 8];
    let s = format_pct(&mut buf, pct);
    let style = MonoTextStyle::new(&FONT_10X20, COLOR_TEXT_PRIMARY);
    Text::new(s, Point::new(x + 42, y + 14), style).draw(d)?;

    if charging {
        Text::new("+", Point::new(x + 85, y + 14), MonoTextStyle::new(&FONT_10X20, COLOR_ACCENT_AMBER)).draw(d)?;
    }

    Ok(())
}

fn format_pct<'a>(buf: &'a mut [u8; 8], pct: u8) -> &'a str {
    if pct >= 100 {
        "100%"
    } else {
        buf[0] = b'0' + (pct / 10);
        buf[1] = b'0' + (pct % 10);
        buf[2] = b'%';
        core::str::from_utf8(&buf[..3]).unwrap_or("0%")
    }
}

// === Material-style Vector Icons for Launcher Apps ===

pub fn draw_icon_container<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    x: i32,
    y: i32,
    size: u32,
) -> Result<(), D::Error> {
    let rect = Rectangle::new(Point::new(x, y), Size::new(size, size));
    let radius = (size / 4).max(6);
    RoundedRectangle::with_equal_corners(rect, Size::new(radius, radius))
        .into_styled(PrimitiveStyle::with_fill(COLOR_SURFACE_MID))
        .draw(d)?;
    RoundedRectangle::with_equal_corners(rect, Size::new(radius, radius))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_BORDER_BRIGHT, 1))
        .draw(d)?;
    Ok(())
}

/// 3D Spherical Golden Ball with multi-tone gradient and specular glint
pub fn draw_golden_ball<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    cx: i32,
    cy: i32,
    r: i32,
) -> Result<(), D::Error> {
    // 1. Dark bronze outer shadow ring
    Circle::new(Point::new(cx - r, cy - r), (r * 2) as u32)
        .into_styled(PrimitiveStyle::with_fill(COLOR_GOLD_DARK))
        .draw(d)?;

    // 2. Warm gold body
    if r > 2 {
        Circle::new(Point::new(cx - r + 2, cy - r + 2), ((r - 2) * 2) as u32)
            .into_styled(PrimitiveStyle::with_fill(COLOR_GOLD_MID))
            .draw(d)?;
    }

    // 3. Bright golden hemisphere (offset towards top-left)
    if r > 4 {
        Circle::new(Point::new(cx - r + 3, cy - r + 3), ((r - 4) * 2) as u32)
            .into_styled(PrimitiveStyle::with_fill(COLOR_GOLD_BRIGHT))
            .draw(d)?;
    }

    // 4. White specular light glint
    if r >= 6 {
        Circle::new(Point::new(cx - r / 2, cy - r / 2), 3)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(d)?;
    }

    Ok(())
}

/// 2x2 App Grid Icon for Watchface Bottom Navigation
pub fn draw_apps_icon<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    cx: i32,
    cy: i32,
    color: Rgb565,
) -> Result<(), D::Error> {
    let s = 8u32;
    for &dy in &[-9, 2] {
        for &dx in &[-9, 2] {
            RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(cx + dx, cy + dy), Size::new(s, s)),
                Size::new(2, 2),
            )
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(d)?;
        }
    }
    Ok(())
}

/// Watch Dial / Style Switcher Icon for Watchface Bottom Navigation
pub fn draw_style_icon<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    cx: i32,
    cy: i32,
    color: Rgb565,
) -> Result<(), D::Error> {
    // Watch circular bezel
    Circle::new(Point::new(cx - 11, cy - 11), 22)
        .into_styled(PrimitiveStyle::with_stroke(color, 2))
        .draw(d)?;
    // Small crown at 3 o'clock
    Rectangle::new(Point::new(cx + 11, cy - 2), Size::new(3, 4))
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(d)?;
    // Watch hands
    Line::new(Point::new(cx, cy), Point::new(cx, cy - 7))
        .into_styled(PrimitiveStyle::with_stroke(color, 2))
        .draw(d)?;
    Line::new(Point::new(cx, cy), Point::new(cx + 6, cy))
        .into_styled(PrimitiveStyle::with_stroke(color, 2))
        .draw(d)?;
    // Pivot
    Circle::new(Point::new(cx - 1, cy - 1), 3)
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(d)?;
    Ok(())
}

/// WiFi Wave Icon
pub fn draw_wifi_symbol<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    cx: i32,
    cy: i32,
    is_connected: bool,
    color: Rgb565,
) -> Result<(), D::Error> {
    if is_connected {
        // Upper arc
        Circle::new(Point::new(cx - 12, cy - 12), 24)
            .into_styled(PrimitiveStyle::with_stroke(color, 2))
            .draw(d)?;
        // Inner arc
        Circle::new(Point::new(cx - 7, cy - 7), 14)
            .into_styled(PrimitiveStyle::with_stroke(color, 2))
            .draw(d)?;
        // Dot
        Circle::new(Point::new(cx - 2, cy + 2), 4)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(d)?;
    } else {
        // Disconnected WiFi symbol (dim with strike-through)
        Circle::new(Point::new(cx - 2, cy + 2), 4)
            .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_MUTED))
            .draw(d)?;
        Line::new(Point::new(cx - 10, cy - 6), Point::new(cx + 10, cy + 6))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_MUTED, 2))
            .draw(d)?;
    }
    Ok(())
}

/// Pedometer Steps Icon (Walking Runner / Footprints)
pub fn draw_steps_icon<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    cx: i32,
    cy: i32,
) -> Result<(), D::Error> {
    // Runner head
    Circle::new(Point::new(cx - 3, cy - 14), 7)
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY))
        .draw(d)?;
    // Torso
    Line::new(Point::new(cx, cy - 7), Point::new(cx - 2, cy + 2))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 3))
        .draw(d)?;
    // Forward leg
    Line::new(Point::new(cx - 2, cy + 2), Point::new(cx + 7, cy + 12))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 2))
        .draw(d)?;
    // Back leg
    Line::new(Point::new(cx - 2, cy + 2), Point::new(cx - 9, cy + 10))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_SECONDARY, 2))
        .draw(d)?;
    // Arms
    Line::new(Point::new(cx - 1, cy - 4), Point::new(cx + 8, cy - 1))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 2))
        .draw(d)?;
    Line::new(Point::new(cx - 1, cy - 4), Point::new(cx - 8, cy - 2))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_SECONDARY, 2))
        .draw(d)?;
    Ok(())
}

/// Xiaozhi AI Assistant Robot Icon
pub fn draw_xiaozhi_icon<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    cx: i32,
    cy: i32,
) -> Result<(), D::Error> {
    // Antenna
    Line::new(Point::new(cx, cy - 8), Point::new(cx, cy - 15))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 2))
        .draw(d)?;
    Circle::new(Point::new(cx - 3, cy - 19), 6)
        .into_styled(PrimitiveStyle::with_fill(COLOR_ACCENT_CYAN))
        .draw(d)?;

    // Ears
    Rectangle::new(Point::new(cx - 18, cy - 2), Size::new(3, 10))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_SECONDARY))
        .draw(d)?;
    Rectangle::new(Point::new(cx + 15, cy - 2), Size::new(3, 10))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_SECONDARY))
        .draw(d)?;

    // Robot Head Outer Frame
    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(cx - 15, cy - 8), Size::new(30, 22)),
        Size::new(4, 4),
    )
    .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 2))
    .draw(d)?;

    // Glowing Eyes
    Circle::new(Point::new(cx - 9, cy - 2), 5)
        .into_styled(PrimitiveStyle::with_fill(COLOR_ACCENT_CYAN))
        .draw(d)?;
    Circle::new(Point::new(cx + 4, cy - 2), 5)
        .into_styled(PrimitiveStyle::with_fill(COLOR_ACCENT_CYAN))
        .draw(d)?;

    // Friendly Mouth Line
    Line::new(Point::new(cx - 6, cy + 7), Point::new(cx + 6, cy + 7))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_SECONDARY, 2))
        .draw(d)?;

    Ok(())
}

pub fn draw_snake_icon<D: DrawTarget<Color = Rgb565>>(d: &mut D, cx: i32, cy: i32) -> Result<(), D::Error> {
    Rectangle::new(Point::new(cx - 14, cy - 10), Size::new(17, 7))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx - 2, cy - 10), Size::new(7, 19))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx + 5, cy + 2), Size::new(12, 7))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Circle::new(Point::new(cx + 9, cy - 10), 6)
        .into_styled(PrimitiveStyle::with_fill(COLOR_ACCENT_GREEN)).draw(d)?;
    Ok(())
}

pub fn draw_2048_icon<D: DrawTarget<Color = Rgb565>>(d: &mut D, cx: i32, cy: i32) -> Result<(), D::Error> {
    let s = 12u32;
    Rectangle::new(Point::new(cx - 14, cy - 14), Size::new(s, s))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx + 2, cy - 14), Size::new(s, s))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_SECONDARY)).draw(d)?;
    Rectangle::new(Point::new(cx - 14, cy + 2), Size::new(s, s))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_SECONDARY)).draw(d)?;
    Rectangle::new(Point::new(cx + 2, cy + 2), Size::new(s, s))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Ok(())
}

pub fn draw_tetris_icon<D: DrawTarget<Color = Rgb565>>(d: &mut D, cx: i32, cy: i32) -> Result<(), D::Error> {
    Rectangle::new(Point::new(cx - 14, cy - 9), Size::new(28, 9))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx - 5, cy), Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Ok(())
}

pub fn draw_flappy_icon<D: DrawTarget<Color = Rgb565>>(d: &mut D, cx: i32, cy: i32) -> Result<(), D::Error> {
    Circle::new(Point::new(cx - 10, cy - 10), 20)
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx + 8, cy - 4), Size::new(7, 7))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_SECONDARY)).draw(d)?;
    Circle::new(Point::new(cx + 1, cy - 7), 4)
        .into_styled(PrimitiveStyle::with_fill(COLOR_BG)).draw(d)?;
    Rectangle::new(Point::new(cx - 12, cy - 2), Size::new(10, 6))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_MUTED)).draw(d)?;
    Ok(())
}

pub fn draw_maze_icon<D: DrawTarget<Color = Rgb565>>(d: &mut D, cx: i32, cy: i32) -> Result<(), D::Error> {
    Circle::new(Point::new(cx - 14, cy - 14), 28)
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 2)).draw(d)?;
    Circle::new(Point::new(cx - 7, cy - 7), 14)
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_SECONDARY, 2)).draw(d)?;
    Circle::new(Point::new(cx - 2, cy - 2), 4)
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Ok(())
}

pub fn draw_music_icon<D: DrawTarget<Color = Rgb565>>(d: &mut D, cx: i32, cy: i32) -> Result<(), D::Error> {
    Circle::new(Point::new(cx - 12, cy + 3), 10)
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Circle::new(Point::new(cx + 2, cy - 2), 10)
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx - 5, cy - 12), Size::new(3, 19))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx + 9, cy - 16), Size::new(3, 18))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Line::new(Point::new(cx - 5, cy - 12), Point::new(cx + 11, cy - 16))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 4)).draw(d)?;
    Ok(())
}

pub fn draw_home_icon<D: DrawTarget<Color = Rgb565>>(d: &mut D, cx: i32, cy: i32) -> Result<(), D::Error> {
    Line::new(Point::new(cx - 14, cy - 2), Point::new(cx, cy - 15))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 2)).draw(d)?;
    Line::new(Point::new(cx, cy - 15), Point::new(cx + 14, cy - 2))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 2)).draw(d)?;
    Rectangle::new(Point::new(cx - 11, cy - 2), Size::new(22, 16))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 2)).draw(d)?;
    Rectangle::new(Point::new(cx - 4, cy + 4), Size::new(8, 10))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Ok(())
}

pub fn draw_gear_icon<D: DrawTarget<Color = Rgb565>>(d: &mut D, cx: i32, cy: i32) -> Result<(), D::Error> {
    Circle::new(Point::new(cx - 12, cy - 12), 24)
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_PRIMARY, 3)).draw(d)?;
    Circle::new(Point::new(cx - 4, cy - 4), 8)
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx - 3, cy - 16), Size::new(6, 6))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx - 3, cy + 10), Size::new(6, 6))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx - 16, cy - 3), Size::new(6, 6))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Rectangle::new(Point::new(cx + 10, cy - 3), Size::new(6, 6))
        .into_styled(PrimitiveStyle::with_fill(COLOR_TEXT_PRIMARY)).draw(d)?;
    Ok(())
}

pub fn draw_chevron_right<D: DrawTarget<Color = Rgb565>>(d: &mut D, x: i32, y: i32, color: Rgb565) -> Result<(), D::Error> {
    Line::new(Point::new(x, y - 7), Point::new(x + 6, y))
        .into_styled(PrimitiveStyle::with_stroke(color, 2)).draw(d)?;
    Line::new(Point::new(x + 6, y), Point::new(x, y + 7))
        .into_styled(PrimitiveStyle::with_stroke(color, 2)).draw(d)?;
    Ok(())
}
