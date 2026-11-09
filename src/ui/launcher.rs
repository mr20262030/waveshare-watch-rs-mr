// Modern Materialize-style 2x4 Grid App Launcher
// High-contrast Monochrome / AMOLED dark cards with custom vector icons
// Bilingual support: Español / English

use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::ascii::{FONT_7X13, FONT_9X15};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Text};

use crate::apps::AppState;
use crate::peripherals::touch::SwipeDirection;
use crate::ui::theme::{self, Language};

const CARD_W: u32 = 181;
const CARD_H: i32 = 102;
const GAP_X: i32 = 12;
const GAP_Y: i32 = 10;
const START_Y: i32 = 38;
const MARGIN_X: i32 = 18;
const MAX_SCROLL: i32 = 120;

#[derive(Clone, Copy)]
pub enum AppIcon {
    Xiaozhi,
    Pedometer,
    Snake,
    Game2048,
    Tetris,
    Flappy,
    Maze,
    Mp3Player,
    SmartHome,
    Settings,
}

struct MenuItem {
    name_es: &'static str,
    name_en: &'static str,
    state: AppState,
    icon: AppIcon,
}

const MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        name_es: "Xiaozhi IA",
        name_en: "Xiaozhi AI",
        state: AppState::XiaozhiBoot,
        icon: AppIcon::Xiaozhi,
    },
    MenuItem {
        name_es: "Podómetro",
        name_en: "Pedometer",
        state: AppState::Pedometer,
        icon: AppIcon::Pedometer,
    },
    MenuItem {
        name_es: "Snake",
        name_en: "Snake",
        state: AppState::Snake,
        icon: AppIcon::Snake,
    },
    MenuItem {
        name_es: "2048",
        name_en: "2048",
        state: AppState::Game2048,
        icon: AppIcon::Game2048,
    },
    MenuItem {
        name_es: "Tetris",
        name_en: "Tetris",
        state: AppState::Tetris,
        icon: AppIcon::Tetris,
    },
    MenuItem {
        name_es: "Flappy",
        name_en: "Flappy Bird",
        state: AppState::Flappy,
        icon: AppIcon::Flappy,
    },
    MenuItem {
        name_es: "Laberinto",
        name_en: "Maze (Tilt)",
        state: AppState::Maze,
        icon: AppIcon::Maze,
    },
    MenuItem {
        name_es: "Música",
        name_en: "MP3 Player",
        state: AppState::Mp3Player,
        icon: AppIcon::Mp3Player,
    },
    MenuItem {
        name_es: "Smart Home",
        name_en: "Smart Home",
        state: AppState::SmartHome,
        icon: AppIcon::SmartHome,
    },
    MenuItem {
        name_es: "Ajustes",
        name_en: "Settings",
        state: AppState::Settings,
        icon: AppIcon::Settings,
    },
];

pub struct Launcher {
    scroll_offset: i32,
    target_scroll: i32,
    tapped_idx: Option<usize>,
    pub language: Language,
}

impl Launcher {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            target_scroll: 0,
            tapped_idx: None,
            language: Language::Spanish,
        }
    }

    pub fn update(
        &mut self,
        swipe: Option<SwipeDirection>,
        tap: bool,
        tap_x: u16,
        tap_y: u16,
    ) -> Option<AppState> {
        match swipe {
            Some(SwipeDirection::Up) => {
                self.target_scroll = (self.target_scroll + 120).min(MAX_SCROLL);
            }
            Some(SwipeDirection::Down) => {
                self.target_scroll = (self.target_scroll - 120).max(0);
            }
            Some(SwipeDirection::Right) => {
                return Some(AppState::Watchface);
            }
            _ => {}
        }

        let diff = self.target_scroll - self.scroll_offset;
        if diff.abs() > 2 {
            self.scroll_offset += diff / 3;
        } else {
            self.scroll_offset = self.target_scroll;
        }

        if tap {
            let x = tap_x as i32;
            let y = tap_y as i32 + self.scroll_offset;
            for (i, item) in MENU_ITEMS.iter().enumerate() {
                let row = (i / 2) as i32;
                let col = (i % 2) as i32;
                let card_x = if col == 0 {
                    MARGIN_X
                } else {
                    MARGIN_X + (CARD_W as i32) + GAP_X
                };
                let card_y = START_Y + row * (CARD_H + GAP_Y);

                if x >= card_x
                    && x < card_x + (CARD_W as i32)
                    && y >= card_y
                    && y < card_y + CARD_H
                {
                    self.tapped_idx = Some(i);
                    return Some(item.state);
                }
            }
        }

        None
    }

    pub fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        // Pure AMOLED Black Background
        let _ = Rectangle::new(Point::zero(), Size::new(410, 502))
            .into_styled(PrimitiveStyle::with_fill(theme::COLOR_BG))
            .draw(d);

        // Minimalist Page / Status Indicator at Top Center
        let style_title = MonoTextStyle::new(&FONT_7X13, theme::COLOR_TEXT_MUTED);
        let title_text = match self.language {
            Language::Spanish => "APPS",
            Language::English => "APPS",
        };
        let _ = Text::with_alignment(title_text, Point::new(205, 20), style_title, Alignment::Center).draw(d);

        // Scroll page dots
        let dot_color_p1 = if self.scroll_offset < 60 {
            theme::COLOR_TEXT_PRIMARY
        } else {
            theme::COLOR_TEXT_MUTED
        };
        let dot_color_p2 = if self.scroll_offset >= 60 {
            theme::COLOR_TEXT_PRIMARY
        } else {
            theme::COLOR_TEXT_MUTED
        };
        let _ = Circle::new(Point::new(198, 26), 4)
            .into_styled(PrimitiveStyle::with_fill(dot_color_p1))
            .draw(d);
        let _ = Circle::new(Point::new(208, 26), 4)
            .into_styled(PrimitiveStyle::with_fill(dot_color_p2))
            .draw(d);

        let style_name = MonoTextStyle::new(&FONT_9X15, theme::COLOR_TEXT_PRIMARY);

        for (i, item) in MENU_ITEMS.iter().enumerate() {
            let row = (i / 2) as i32;
            let col = (i % 2) as i32;
            let card_x = if col == 0 {
                MARGIN_X
            } else {
                MARGIN_X + (CARD_W as i32) + GAP_X
            };
            let card_y = START_Y + row * (CARD_H + GAP_Y) - self.scroll_offset;

            // Cull offscreen cards
            if card_y + CARD_H < 25 || card_y > 502 {
                continue;
            }

            let card_rect = Rectangle::new(Point::new(card_x, card_y), Size::new(CARD_W, CARD_H as u32));
            let is_pressed = self.tapped_idx == Some(i);

            // Large tactile rounded card (radius 16)
            let _ = theme::draw_card(d, card_rect, 16, is_pressed);

            // Large Icon container (46x46 px centered horizontally in card)
            let cx = card_x + (CARD_W as i32) / 2;
            let cy = card_y + 36;
            let _ = theme::draw_icon_container(d, cx - 23, cy - 23, 46);

            // Render vector icon centered
            match item.icon {
                AppIcon::Xiaozhi => {
                    let _ = theme::draw_xiaozhi_icon(d, cx, cy);
                }
                AppIcon::Pedometer => {
                    let _ = theme::draw_steps_icon(d, cx, cy);
                }
                AppIcon::Snake => {
                    let _ = theme::draw_snake_icon(d, cx, cy);
                }
                AppIcon::Game2048 => {
                    let _ = theme::draw_2048_icon(d, cx, cy);
                }
                AppIcon::Tetris => {
                    let _ = theme::draw_tetris_icon(d, cx, cy);
                }
                AppIcon::Flappy => {
                    let _ = theme::draw_flappy_icon(d, cx, cy);
                }
                AppIcon::Maze => {
                    let _ = theme::draw_maze_icon(d, cx, cy);
                }
                AppIcon::Mp3Player => {
                    let _ = theme::draw_music_icon(d, cx, cy);
                }
                AppIcon::SmartHome => {
                    let _ = theme::draw_home_icon(d, cx, cy);
                }
                AppIcon::Settings => {
                    let _ = theme::draw_gear_icon(d, cx, cy);
                }
            }

            // Bold app name centered below the icon
            let name = match self.language {
                Language::Spanish => item.name_es,
                Language::English => item.name_en,
            };

            let _ = Text::with_alignment(
                name,
                Point::new(cx, card_y + CARD_H - 14),
                style_name,
                Alignment::Center,
            )
            .draw(d);
        }
    }
}
