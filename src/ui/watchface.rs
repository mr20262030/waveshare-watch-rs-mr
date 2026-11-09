// Modern Watchface - High-Contrast AMOLED Design
// Supports 3 selectable styles: Minimal Digital, Classic Analog, and Tactical Sport.
// Bilingual support: Español / English.

use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Line, PrimitiveStyle, Rectangle, RoundedRectangle,
};
use embedded_graphics::text::{Alignment, Text};

use crate::board;
use crate::ui::segments;
use crate::ui::theme::{self, Language, WatchfaceStyle};

const SCREEN_CX: i32 = board::LCD_WIDTH as i32 / 2; // 205
const SCREEN_CY: i32 = board::LCD_HEIGHT as i32 / 2; // 251

// 60-point circle table: (sin * 1000, -cos * 1000) for each 6° step around the dial
const CIRCLE_60: [(i32, i32); 60] = [
    (0, -1000), (105, -995), (208, -978), (309, -951), (407, -914), (500, -866),
    (588, -809), (669, -743), (743, -669), (809, -588), (866, -500), (914, -407),
    (951, -309), (978, -208), (995, -105), (1000, 0), (995, 105), (978, 208),
    (951, 309), (914, 407), (866, 500), (809, 588), (743, 669), (669, 743),
    (588, 809), (500, 866), (407, 914), (309, 951), (208, 978), (105, 995),
    (0, 1000), (-105, 995), (-208, 978), (-309, 951), (-407, 914), (-500, 866),
    (-588, 809), (-669, 743), (-743, 669), (-809, 588), (-866, 500), (-914, 407),
    (-951, 309), (-978, 208), (-995, 105), (-1000, 0), (-995, -105), (-978, -208),
    (-951, -309), (-914, -407), (-866, -500), (-809, -588), (-743, -669), (-669, -743),
    (-588, -809), (-500, -866), (-407, -914), (-309, -951), (-208, -978), (-105, -995),
];

fn dial_point(step: usize, radius: i32, cx: i32, cy: i32) -> Point {
    let entry = CIRCLE_60[step % 60];
    Point::new(cx + (entry.0 * radius) / 1000, cy + (entry.1 * radius) / 1000)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderOutcome {
    pub full_redraw: bool,
}

pub struct WatchFace {
    hours: u8,
    minutes: u8,
    seconds: u8,
    battery_percent: u8,
    battery_voltage: u16,
    is_charging: bool,
    accel_x: i16,
    accel_y: i16,
    accel_z: i16,
    day: u8,
    month: u8,
    year: u8,
    full_redraw: bool,
    pub wifi_connected: bool,
    pub ble_on: bool,
    pub gyro_enabled: bool,
    pub brightness: u8,
    pub cpu_mhz: u16,
    pub style: WatchfaceStyle,
    pub language: Language,
    pub steps: u32,
}

impl WatchFace {
    pub fn new() -> Self {
        Self {
            hours: 0,
            minutes: 0,
            seconds: 0,
            battery_percent: 0,
            battery_voltage: 0,
            is_charging: false,
            accel_x: 0,
            accel_y: 0,
            accel_z: 0,
            day: 6,
            month: 11,
            year: 26,
            full_redraw: true,
            wifi_connected: false,
            ble_on: false,
            gyro_enabled: false,
            brightness: 0xA0,
            cpu_mhz: 160,
            style: WatchfaceStyle::MinimalDigital,
            language: Language::Spanish,
            steps: 0,
        }
    }

    pub fn update_time(&mut self, h: u8, m: u8, s: u8) {
        if self.hours != h || self.minutes != m || self.seconds != s {
            self.hours = h;
            self.minutes = m;
            self.seconds = s;
            self.full_redraw = true;
        }
    }

    pub fn update_date(&mut self, day: u8, month: u8, year: u8) {
        if self.day != day || self.month != month || self.year != year {
            self.day = day;
            self.month = month;
            self.year = year;
            self.full_redraw = true;
        }
    }

    pub fn update_battery(&mut self, pct: u8, mv: u16, chg: bool) {
        if self.battery_percent != pct || self.battery_voltage != mv || self.is_charging != chg {
            self.battery_percent = pct;
            self.battery_voltage = mv;
            self.is_charging = chg;
            self.full_redraw = true;
        }
    }

    pub fn update_accel(&mut self, x: f32, y: f32, z: f32) {
        let nx = (x * 100.0) as i16;
        let ny = (y * 100.0) as i16;
        let nz = (z * 100.0) as i16;
        if (nx - self.accel_x).abs() > 1 || (ny - self.accel_y).abs() > 1 || (nz - self.accel_z).abs() > 1 {
            self.accel_x = nx;
            self.accel_y = ny;
            self.accel_z = nz;
            if self.style == WatchfaceStyle::SportTactical {
                self.full_redraw = true;
            }
        }
    }

    pub fn force_redraw(&mut self) {
        self.full_redraw = true;
    }

    pub fn next_style(&mut self) {
        self.style = self.style.next();
        self.full_redraw = true;
    }

    pub fn toggle_language(&mut self) {
        self.language = self.language.toggle();
        self.full_redraw = true;
    }

    pub fn needs_render(&self) -> bool {
        self.full_redraw
    }

    /// Minimalist Always-On Display (AOD): Super-dim HH:MM on pure black AMOLED
    pub fn render_aod<D: DrawTarget<Color = Rgb565>>(&mut self, d: &mut D) -> Result<(), D::Error> {
        let _ = Rectangle::new(Point::zero(), Size::new(410, 502))
            .into_styled(PrimitiveStyle::with_fill(theme::COLOR_BG))
            .draw(d);

        segments::draw_hhmm(
            d,
            SCREEN_CX,
            180,
            self.hours,
            self.minutes,
            Rgb565::new(6, 12, 6),
            theme::COLOR_BG,
        )?;

        Ok(())
    }

    /// Check if tap is in the Apps button zone (bottom left/center)
    pub fn is_apps_zone(x: u16, y: u16) -> bool {
        let xi = x as i32;
        let yi = y as i32;
        yi >= 435 && yi <= 495 && xi >= 20 && xi <= 200
    }

    /// Check if tap is in the Style switcher zone (bottom right)
    pub fn is_style_zone(x: u16, y: u16) -> bool {
        let xi = x as i32;
        let yi = y as i32;
        yi >= 435 && yi <= 495 && xi >= 210 && xi <= 390
    }

    /// Check if tap is in the Top Status bar (WiFi or Language toggle)
    pub fn is_top_status_zone(x: u16, y: u16) -> bool {
        let yi = y as i32;
        yi >= 5 && yi <= 60
    }

    pub fn is_wifi_zone(x: u16, y: u16) -> bool {
        let xi = x as i32;
        let yi = y as i32;
        (yi >= 5 && yi <= 55 && xi >= 10 && xi <= 110)
        || (yi >= 155 && yi <= 240 && xi >= 18 && xi <= 200)
    }

    pub fn brightness_from_tap(_x: u16, _y: u16) -> Option<u8> {
        None
    }

    pub fn render<D: DrawTarget<Color = Rgb565>>(&mut self, d: &mut D) -> Result<RenderOutcome, D::Error> {
        // Pure AMOLED Black background
        let _ = Rectangle::new(Point::zero(), Size::new(410, 502))
            .into_styled(PrimitiveStyle::with_fill(theme::COLOR_BG))
            .draw(d);

        // 1. Top Modern Status Bar
        self.render_status_bar(d)?;

        // 2. Render Watchface according to selected style
        match self.style {
            WatchfaceStyle::MinimalDigital => self.render_minimal_digital(d)?,
            WatchfaceStyle::AnalogClassic => self.render_analog_classic(d)?,
            WatchfaceStyle::SportTactical => self.render_sport_tactical(d)?,
        }

        // 3. Bottom Modern Action Buttons ([ APPS ] and [ ESTILO / STYLE ])
        self.render_bottom_controls(d)?;

        self.full_redraw = false;
        Ok(RenderOutcome { full_redraw: true })
    }

    /// Sleek status bar: WiFi capsule on left, Battery on right
    fn render_status_bar<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) -> Result<(), D::Error> {
        // Left: WiFi Status Pill
        let wifi_color = if self.wifi_connected {
            theme::COLOR_ACCENT_GREEN
        } else {
            theme::COLOR_TEXT_MUTED
        };
        let wifi_text = if self.wifi_connected { "WIFI" } else { "WIFI OFF" };
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(18, 14), Size::new(82, 28)),
            Size::new(14, 14),
        )
        .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_BORDER, 1))
        .draw(d);
        let _ = Text::with_alignment(
            wifi_text,
            Point::new(59, 33),
            MonoTextStyle::new(&FONT_10X20, wifi_color),
            Alignment::Center,
        )
        .draw(d);

        // Right: Battery Status Pill
        theme::draw_battery_status(d, 280, 20, self.battery_percent, self.is_charging)?;

        Ok(())
    }

    /// Style 1: Minimalist Digital - Large clean digits, date, second bar
    fn render_minimal_digital<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) -> Result<(), D::Error> {
        let cx = SCREEN_CX;

        // Giant Digital Clock (HH:MM)
        segments::draw_hhmm(
            d,
            cx,
            120,
            self.hours,
            self.minutes,
            theme::COLOR_TEXT_PRIMARY,
            theme::COLOR_BG,
        )?;

        // Seconds indicator: Smooth horizontal progress track (y: 240, w: 260)
        let bar_x = cx - 130;
        let bar_y = 245;
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(bar_x, bar_y), Size::new(260, 6)),
            Size::new(3, 3),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_SURFACE_MID))
        .draw(d);

        let sec_w = ((self.seconds as u32) * 260) / 60;
        if sec_w > 0 {
            let _ = RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(bar_x, bar_y), Size::new(sec_w, 6)),
                Size::new(3, 3),
            )
            .into_styled(PrimitiveStyle::with_fill(theme::COLOR_TEXT_PRIMARY))
            .draw(d);
        }

        // Seconds numeric label
        let mut sec_buf = [0u8; 8];
        sec_buf[0] = b':';
        sec_buf[1] = b'0' + (self.seconds / 10);
        sec_buf[2] = b'0' + (self.seconds % 10);
        let sec_str = core::str::from_utf8(&sec_buf[..3]).unwrap_or("");
        let _ = Text::with_alignment(
            sec_str,
            Point::new(cx, 276),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_MUTED),
            Alignment::Center,
        )
        .draw(d);

        // Date Bar in Spanish / English (e.g. "VIERNES 06 NOV")
        let mut date_buf = [0u8; 32];
        let ds = theme::fmt_date(&mut date_buf, self.day, self.month, self.year, self.language);
        let _ = Text::with_alignment(
            ds,
            Point::new(cx, 320),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_SECONDARY),
            Alignment::Center,
        )
        .draw(d);

        // Steps indicator with runner icon and golden accent
        self.render_steps_badge(d, cx, 360)?;

        // Style Badge
        let style_name = self.style.name(self.language);
        let _ = Text::with_alignment(
            style_name,
            Point::new(cx, 400),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_MUTED),
            Alignment::Center,
        )
        .draw(d);

        Ok(())
    }

    /// Style 2: Classic Analog - Round dial, sword hands, date complication
    fn render_analog_classic<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) -> Result<(), D::Error> {
        let cx = SCREEN_CX;
        let cy = 230;
        let r = 165;

        // Dial outer rim
        Circle::new(Point::new(cx - r, cy - r), (r * 2) as u32)
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_BORDER, 2))
            .draw(d)?;

        // Dial 12 Hour Tick Marks
        for i in 0..12 {
            let step = i * 5;
            let is_cardinal = i % 3 == 0; // 12, 3, 6, 9
            let tick_len = if is_cardinal { 18 } else { 10 };
            let tick_width = if is_cardinal { 3 } else { 1 };
            let tick_color = if is_cardinal {
                theme::COLOR_TEXT_PRIMARY
            } else {
                theme::COLOR_TEXT_MUTED
            };

            let p_outer = dial_point(step, r - 4, cx, cy);
            let p_inner = dial_point(step, r - 4 - tick_len, cx, cy);

            Line::new(p_outer, p_inner)
                .into_styled(PrimitiveStyle::with_stroke(tick_color, tick_width))
                .draw(d)?;
        }

        // Complication: Date Window at 3 o'clock (x: cx + 75, y: cy - 12)
        let win_x = cx + 70;
        let win_y = cy - 14;
        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(win_x, win_y), Size::new(42, 28)),
            Size::new(4, 4),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_SURFACE_MID))
        .draw(d)?;
        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(win_x, win_y), Size::new(42, 28)),
            Size::new(4, 4),
        )
        .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_BORDER_BRIGHT, 1))
        .draw(d)?;

        let mut day_buf = [0u8; 4];
        day_buf[0] = b'0' + (self.day / 10);
        day_buf[1] = b'0' + (self.day % 10);
        let day_str = core::str::from_utf8(&day_buf[..2]).unwrap_or("");
        Text::with_alignment(
            day_str,
            Point::new(win_x + 21, win_y + 19),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_PRIMARY),
            Alignment::Center,
        )
        .draw(d)?;

        // Hour Hand (width 5, radius 75)
        let hour_step = ((self.hours as usize % 12) * 5) + (self.minutes as usize / 12);
        let h_target = dial_point(hour_step, 75, cx, cy);
        Line::new(Point::new(cx, cy), h_target)
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_TEXT_PRIMARY, 5))
            .draw(d)?;

        // Minute Hand (width 3, radius 120)
        let min_step = self.minutes as usize;
        let m_target = dial_point(min_step, 120, cx, cy);
        Line::new(Point::new(cx, cy), m_target)
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_TEXT_SECONDARY, 3))
            .draw(d)?;

        // Second Hand (width 1, radius 135) + Counterweight (length 25)
        let sec_step = self.seconds as usize;
        let s_target = dial_point(sec_step, 135, cx, cy);
        let s_tail = dial_point((sec_step + 30) % 60, 25, cx, cy);
        Line::new(s_tail, s_target)
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_ACCENT_CYAN, 1))
            .draw(d)?;

        // Center Cap (Circle with metallic ring)
        Circle::new(Point::new(cx - 7, cy - 7), 14)
            .into_styled(PrimitiveStyle::with_fill(theme::COLOR_TEXT_PRIMARY))
            .draw(d)?;
        Circle::new(Point::new(cx - 3, cy - 3), 6)
            .into_styled(PrimitiveStyle::with_fill(theme::COLOR_BG))
            .draw(d)?;

        // Steps complication at 6 o'clock (below dial)
        self.render_steps_badge(d, cx, cy + r + 22)?;

        Ok(())
    }

    /// Style 3: Tactical Sport - Golden horizon ball, WiFi & Battery gauges
    fn render_sport_tactical<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) -> Result<(), D::Error> {
        let cx = SCREEN_CX;

        // Digital Time HH:MM:SS in upper area
        segments::draw_time(
            d,
            cx,
            75,
            self.hours,
            self.minutes,
            self.seconds,
            theme::COLOR_TEXT_PRIMARY,
            theme::COLOR_BG,
        )?;

        // Date in Spanish / English
        let mut date_buf = [0u8; 32];
        let ds = theme::fmt_date(&mut date_buf, self.day, self.month, self.year, self.language);
        Text::with_alignment(
            ds,
            Point::new(cx, 160),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_SECONDARY),
            Alignment::Center,
        )
        .draw(d)?;

        // --- WiFi Signal Capsule Switch (left) and Battery Gauge (right) ---
        // WiFi status capsule (x: 30..168, y: 180..210)
        theme::draw_capsule_switch(d, 38, 182, 52, 26, self.wifi_connected)?;
        let wifi_label = if self.wifi_connected { "WIFI" } else { "OFF" };
        let wifi_color = if self.wifi_connected { theme::COLOR_ACCENT_GREEN } else { theme::COLOR_TEXT_MUTED };
        Text::new(
            wifi_label,
            Point::new(98, 200),
            MonoTextStyle::new(&FONT_10X20, wifi_color),
        )
        .draw(d)?;

        // Battery gauge (right side, x: 240..390, y: 180..210)
        theme::draw_battery_status(d, 250, 186, self.battery_percent, self.is_charging)?;

        // --- Live Horizon / Gyro Level Ball (y: 225..380) ---
        let horizon_cy = 310;
        let horizon_r = 55;

        // Outer ring
        Circle::new(Point::new(cx - horizon_r, horizon_cy - horizon_r), (horizon_r * 2) as u32)
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_BORDER, 2))
            .draw(d)?;

        // Inner ring
        Circle::new(Point::new(cx - horizon_r + 12, horizon_cy - 12 + 12 - horizon_r), ((horizon_r - 12) * 2) as u32)
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_SURFACE_MID, 1))
            .draw(d)?;

        // Crosshair
        Line::new(
            Point::new(cx - horizon_r + 8, horizon_cy),
            Point::new(cx + horizon_r - 8, horizon_cy),
        )
        .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_BORDER_BRIGHT, 1))
        .draw(d)?;
        Line::new(
            Point::new(cx, horizon_cy - horizon_r + 8),
            Point::new(cx, horizon_cy + horizon_r - 8),
        )
        .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_BORDER_BRIGHT, 1))
        .draw(d)?;

        // Leveling ball based on accelerometer tilt — Golden 3D Gradient
        let ball_ox = (self.accel_x as i32 * (horizon_r - 12)) / 100;
        let ball_oy = (self.accel_y as i32 * (horizon_r - 12)) / 100;
        let bx = (cx + ball_ox).clamp(cx - horizon_r + 10, cx + horizon_r - 10);
        let by = (horizon_cy + ball_oy).clamp(horizon_cy - horizon_r + 10, horizon_cy + horizon_r - 10);

        theme::draw_golden_ball(d, bx, by, 10)?;

        let level_text = if self.language == Language::Spanish { "NIVEL" } else { "HORIZON" };
        Text::with_alignment(
            level_text,
            Point::new(cx, horizon_cy + horizon_r + 18),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_MUTED),
            Alignment::Center,
        )
        .draw(d)?;

        // Steps indicator at bottom of sport face
        self.render_steps_badge(d, cx, horizon_cy + horizon_r + 42)?;

        Ok(())
    }

    /// Compact steps badge: runner icon + step count in golden text
    fn render_steps_badge<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D, cx: i32, y: i32) -> Result<(), D::Error> {
        // Runner icon (small, on the left of center)
        theme::draw_steps_icon(d, cx - 60, y - 4)?;

        // Step count number
        let mut buf = [0u8; 16];
        let s = fmt_steps(&mut buf, self.steps);
        Text::with_alignment(
            s,
            Point::new(cx + 10, y + 6),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_GOLD_BRIGHT),
            Alignment::Center,
        )
        .draw(d)?;

        Ok(())
    }

    /// Bottom Action Icons: App Grid (left) and Style Dial (right)
    fn render_bottom_controls<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) -> Result<(), D::Error> {
        // Left: Apps icon circle (cx: 110, cy: 465)
        Circle::new(Point::new(80, 440), 50)
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_BORDER_BRIGHT, 1))
            .draw(d)?;
        theme::draw_apps_icon(d, 105, 465, theme::COLOR_TEXT_PRIMARY)?;

        // Right: Style dial icon circle (cx: 305, cy: 465)
        Circle::new(Point::new(275, 440), 50)
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_BORDER_BRIGHT, 1))
            .draw(d)?;
        theme::draw_style_icon(d, 300, 465, theme::COLOR_TEXT_PRIMARY)?;

        Ok(())
    }
}

/// Format step count with K suffix for large numbers (e.g. "1,234" or "12.3K")
fn fmt_steps<'a>(buf: &'a mut [u8; 16], steps: u32) -> &'a str {
    if steps >= 10000 {
        // Show as "XX.XK"
        let k = steps / 1000;
        let d = (steps % 1000) / 100;
        let mut i = 0;
        if k >= 10 {
            buf[i] = b'0' + ((k / 10) % 10) as u8;
            i += 1;
        }
        buf[i] = b'0' + (k % 10) as u8;
        i += 1;
        buf[i] = b'.';
        i += 1;
        buf[i] = b'0' + (d as u8);
        i += 1;
        buf[i] = b'K';
        i += 1;
        core::str::from_utf8(&buf[..i]).unwrap_or("0")
    } else if steps >= 1000 {
        // Show as "X,XXX"
        let thousands = steps / 1000;
        let rest = steps % 1000;
        buf[0] = b'0' + (thousands as u8);
        buf[1] = b',';
        buf[2] = b'0' + ((rest / 100) % 10) as u8;
        buf[3] = b'0' + ((rest / 10) % 10) as u8;
        buf[4] = b'0' + (rest % 10) as u8;
        core::str::from_utf8(&buf[..5]).unwrap_or("0")
    } else {
        // Show as plain number
        let mut i = 0;
        if steps >= 100 {
            buf[i] = b'0' + ((steps / 100) % 10) as u8;
            i += 1;
        }
        if steps >= 10 {
            buf[i] = b'0' + ((steps / 10) % 10) as u8;
            i += 1;
        }
        buf[i] = b'0' + (steps % 10) as u8;
        i += 1;
        core::str::from_utf8(&buf[..i]).unwrap_or("0")
    }
}
