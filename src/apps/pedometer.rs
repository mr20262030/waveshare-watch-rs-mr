// Modern Pedometer App - Real-Time Step Counter with AMOLED High-Contrast UI
// Uses 3-axis accelerometer data from QMI8658 to detect steps dynamically.
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

use crate::apps::{App, AppInput, AppResult};
use crate::peripherals::touch::SwipeDirection;
use crate::ui::theme::{self, Language};

pub struct PedometerApp {
    steps: u32,
    goal: u32,
    active_ms: u32,
    last_mag: f32,
    avg_mag: f32,
    step_debounce_ms: u32,
    armed: bool,
    pub language: Language,
}

impl PedometerApp {
    pub fn new() -> Self {
        Self {
            steps: 0,
            goal: 10000,
            active_ms: 0,
            last_mag: 1.0,
            avg_mag: 1.0,
            step_debounce_ms: 0,
            armed: true,
            language: Language::Spanish,
        }
    }

    pub fn steps(&self) -> u32 {
        self.steps
    }

    pub fn reset(&mut self) {
        self.steps = 0;
        self.active_ms = 0;
    }
}

impl App for PedometerApp {
    fn name(&self) -> &str {
        match self.language {
            Language::Spanish => "Podómetro",
            Language::English => "Pedometer",
        }
    }

    fn setup(&mut self) {
        self.step_debounce_ms = 0;
        self.armed = true;
    }

    fn update(&mut self, input: &AppInput) -> AppResult {
        // Swipe right or back exits to launcher
        if let Some(SwipeDirection::Right) = input.swipe {
            return AppResult::Exit;
        }

        // Tap on reset button (y: 430..485, x: 80..330)
        if input.tap {
            if let Some(tp) = input.touch {
                let xi = tp.x as i32;
                let yi = tp.y as i32;
                if yi >= 430 && yi <= 485 && xi >= 80 && xi <= 330 {
                    self.reset();
                    return AppResult::Continue;
                }
                // Back button zone at top-left
                if yi <= 50 && xi <= 70 {
                    return AppResult::Exit;
                }
            }
        }

        // --- Step Detection Algorithm ---
        let dt = input.dt_ms.max(1);
        if self.step_debounce_ms > dt {
            self.step_debounce_ms -= dt;
        } else {
            self.step_debounce_ms = 0;
        }

        let (ax, ay, az) = input.accel;
        // Total acceleration magnitude in g using standard core integer isqrt
        let mx = (ax * 100.0) as i32;
        let my = (ay * 100.0) as i32;
        let mz = (az * 100.0) as i32;
        let mag = (((mx * mx + my * my + mz * mz) as u32).isqrt() as f32) / 100.0;

        // Low-pass filter for baseline gravity tracking (alpha = 0.05)
        self.avg_mag += (mag - self.avg_mag) * 0.05;
        let delta = mag - self.avg_mag;

        // Step peak detection:
        // Arm when signal dips slightly below mean
        if delta < -0.12 {
            self.armed = true;
        }

        // Trigger step on positive acceleration peak > 0.25g above dynamic mean with min 260ms refractory window
        if self.armed && delta > 0.25 && self.step_debounce_ms == 0 {
            self.steps += 1;
            self.armed = false;
            self.step_debounce_ms = 260; // Max ~3.8 steps/sec
            self.active_ms += 450;
        }

        self.last_mag = mag;

        AppResult::Continue
    }

    fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        // Pure AMOLED Black Background
        let _ = Rectangle::new(Point::zero(), Size::new(410, 502))
            .into_styled(PrimitiveStyle::with_fill(theme::COLOR_BG))
            .draw(d);

        // 1. Header (Title + Back Chevron)
        let _ = theme::draw_chevron_right(d, 28, 30, theme::COLOR_TEXT_SECONDARY);
        // Turn chevron into a left arrow by drawing opposite lines
        let _ = Line::new(Point::new(32, 23), Point::new(24, 30))
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_TEXT_SECONDARY, 2))
            .draw(d);
        let _ = Line::new(Point::new(24, 30), Point::new(32, 37))
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_TEXT_SECONDARY, 2))
            .draw(d);

        let title = match self.language {
            Language::Spanish => "PODÓMETRO",
            Language::English => "PEDOMETER",
        };
        let _ = Text::with_alignment(
            title,
            Point::new(205, 36),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_PRIMARY),
            Alignment::Center,
        )
        .draw(d);

        // 2. Central Goal Progress Ring (Radius 85, Center: 205, 175)
        let cx = 205;
        let cy = 170;
        let ring_r = 85;

        // Outer track ring
        let _ = Circle::new(Point::new(cx - ring_r, cy - ring_r), (ring_r * 2) as u32)
            .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_SURFACE_MID, 8))
            .draw(d);

        // Runner / Step Icon in top of ring
        let _ = theme::draw_steps_icon(d, cx, cy - 35);

        // Giant Steps Numeric Counter
        let mut steps_buf = [0u8; 16];
        let steps_str = format_u32_commas(&mut steps_buf, self.steps);
        let _ = Text::with_alignment(
            steps_str,
            Point::new(cx, cy + 15),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_PRIMARY),
            Alignment::Center,
        )
        .draw(d);

        // Steps Label & Goal
        let sub_label = match self.language {
            Language::Spanish => "PASOS / META 10K",
            Language::English => "STEPS / 10K GOAL",
        };
        let _ = Text::with_alignment(
            sub_label,
            Point::new(cx, cy + 38),
            MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_MUTED),
            Alignment::Center,
        )
        .draw(d);

        // 3. Goal Progress Bar below Ring (w: 240, h: 6)
        let bar_x = cx - 120;
        let bar_y = cy + ring_r + 15;
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(bar_x, bar_y), Size::new(240, 6)),
            Size::new(3, 3),
        )
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_SURFACE_MID))
        .draw(d);

        let pct = ((self.steps as u64 * 100) / self.goal as u64).min(100) as u32;
        let fill_w = (pct * 240) / 100;
        if fill_w > 0 {
            let _ = RoundedRectangle::with_equal_corners(
                Rectangle::new(Point::new(bar_x, bar_y), Size::new(fill_w, 6)),
                Size::new(3, 3),
            )
            .into_styled(PrimitiveStyle::with_fill(theme::COLOR_GOLD_BRIGHT))
            .draw(d);
        }

        // 4. Metric Cards (Distance, Calories, Time) at y: 300..410
        // Card 1: Distance (x: 20, w: 116)
        let _ = render_stat_card(
            d,
            20,
            300,
            116,
            88,
            format_distance(self.steps),
            "KM",
            match self.language {
                Language::Spanish => "DISTANCIA",
                Language::English => "DISTANCE",
            },
        );

        // Card 2: Calories (x: 147, w: 116)
        let _ = render_stat_card(
            d,
            147,
            300,
            116,
            88,
            format_calories(self.steps),
            "KCAL",
            match self.language {
                Language::Spanish => "CALORIAS",
                Language::English => "CALORIES",
            },
        );

        // Card 3: Active Time (x: 274, w: 116)
        let _ = render_stat_card(
            d,
            274,
            300,
            116,
            88,
            format_time(self.active_ms),
            "MIN",
            match self.language {
                Language::Spanish => "TIEMPO",
                Language::English => "ACTIVE",
            },
        );

        // 5. Bottom Action Button: [ REINICIAR / RESET ]
        let btn_label = match self.language {
            Language::Spanish => "REINICIAR",
            Language::English => "RESET",
        };
        let _ = theme::draw_pill_button(d, 105, 435, 200, 46, btn_label, false);
    }
}

fn render_stat_card<D: DrawTarget<Color = Rgb565>>(
    d: &mut D,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    val: [u8; 8],
    unit: &'static str,
    title: &'static str,
) -> Result<(), D::Error> {
    let rect = Rectangle::new(Point::new(x, y), Size::new(w, h));
    RoundedRectangle::with_equal_corners(rect, Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_fill(theme::COLOR_CARD_BG))
        .draw(d)?;
    RoundedRectangle::with_equal_corners(rect, Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_stroke(theme::COLOR_BORDER, 1))
        .draw(d)?;

    let val_str = core::str::from_utf8(&val).unwrap_or("0");
    let val_clean = val_str.trim_matches('\0');

    // Large metric value
    Text::with_alignment(
        val_clean,
        Point::new(x + (w as i32) / 2, y + 30),
        MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_PRIMARY),
        Alignment::Center,
    )
    .draw(d)?;

    // Unit
    Text::with_alignment(
        unit,
        Point::new(x + (w as i32) / 2, y + 52),
        MonoTextStyle::new(&FONT_10X20, theme::COLOR_GOLD_MID),
        Alignment::Center,
    )
    .draw(d)?;

    // Title label
    Text::with_alignment(
        title,
        Point::new(x + (w as i32) / 2, y + 74),
        MonoTextStyle::new(&FONT_10X20, theme::COLOR_TEXT_MUTED),
        Alignment::Center,
    )
    .draw(d)?;

    Ok(())
}

fn format_u32_commas<'a>(buf: &'a mut [u8; 16], val: u32) -> &'a str {
    if val == 0 {
        return "0";
    }
    let mut temp = [0u8; 12];
    let mut n = val;
    let mut len = 0;
    while n > 0 {
        temp[len] = b'0' + (n % 10) as u8;
        n /= 10;
        len += 1;
    }

    let mut out_len = 0;
    for i in 0..len {
        if i > 0 && (len - i) % 3 == 0 {
            buf[out_len] = b',';
            out_len += 1;
        }
        buf[out_len] = temp[len - 1 - i];
        out_len += 1;
    }
    core::str::from_utf8(&buf[..out_len]).unwrap_or("0")
}

fn format_distance(steps: u32) -> [u8; 8] {
    // Approx 0.75m per step -> km = steps * 75 / 100000
    let meters = (steps as u64 * 75) / 100;
    let km = meters / 1000;
    let dec = (meters % 1000) / 100;

    let mut buf = [0u8; 8];
    buf[0] = b'0' + ((km / 10) % 10) as u8;
    buf[1] = b'0' + (km % 10) as u8;
    buf[2] = b'.';
    buf[3] = b'0' + (dec % 10) as u8;
    buf
}

fn format_calories(steps: u32) -> [u8; 8] {
    // Approx 0.04 kcal per step
    let kcal = (steps * 4) / 100;
    let mut buf = [0u8; 8];
    if kcal >= 1000 {
        buf[0] = b'0' + ((kcal / 1000) % 10) as u8;
        buf[1] = b'0' + ((kcal / 100) % 10) as u8;
        buf[2] = b'0' + ((kcal / 10) % 10) as u8;
        buf[3] = b'0' + (kcal % 10) as u8;
    } else {
        buf[0] = b'0' + ((kcal / 100) % 10) as u8;
        buf[1] = b'0' + ((kcal / 10) % 10) as u8;
        buf[2] = b'0' + (kcal % 10) as u8;
    }
    buf
}

fn format_time(active_ms: u32) -> [u8; 8] {
    let mins = active_ms / 60000;
    let mut buf = [0u8; 8];
    buf[0] = b'0' + ((mins / 10) % 10) as u8;
    buf[1] = b'0' + (mins % 10) as u8;
    buf
}
