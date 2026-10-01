use raylib::prelude::*;

use super::{draw_outlined_text, draw_text_centered, point_in_rectangle};

const INITIAL_VOLUME: f32 = 0.65;
const VOLUME_STEP: f32 = 0.05;
const BUTTON_LABELS: [&str; 3] = ["Iniciar", "Sonido", "Salir"];
const BACKGROUND_TOP: Color = Color::new(7, 17, 42, 255);
const BACKGROUND_BOTTOM: Color = Color::new(27, 50, 91, 255);
const BIRD_GREEN: Color = Color::new(156, 222, 46, 255);
const BIRD_GREEN_DARK: Color = Color::new(39, 87, 27, 255);
const WARM_WHITE: Color = Color::new(255, 250, 230, 255);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntroScreen {
    Main,
    Sound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IntroAction {
    None,
    Start,
    Exit,
}

pub(super) struct IntroMenu {
    screen: IntroScreen,
    focus: usize,
    volume: f32,
    muted: bool,
}

impl IntroMenu {
    pub(super) fn new() -> Self {
        Self {
            screen: IntroScreen::Main,
            focus: 0,
            volume: INITIAL_VOLUME,
            muted: false,
        }
    }

    pub(super) fn effective_volume(&self) -> f32 {
        if self.muted { 0.0 } else { self.volume }
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.muted = false;
    }

    pub(super) fn update(&mut self, rl: &RaylibHandle) -> IntroAction {
        let screen = (rl.get_screen_width(), rl.get_screen_height());
        let mouse = rl.get_mouse_position();
        let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        match self.screen {
            IntroScreen::Main => {
                if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                    self.focus = (self.focus + BUTTON_LABELS.len() - 1) % BUTTON_LABELS.len();
                }
                if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                    self.focus = (self.focus + 1) % BUTTON_LABELS.len();
                }
                if clicked && let Some(index) = main_button_at(mouse, screen) {
                    self.focus = index;
                    return self.choose_main_button(index);
                }
                if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                    || rl.is_key_pressed(KeyboardKey::KEY_KP_ENTER)
                {
                    return self.choose_main_button(self.focus);
                }
            }
            IntroScreen::Sound => {
                if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
                    self.screen = IntroScreen::Main;
                    return IntroAction::None;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
                    self.set_volume(self.volume - VOLUME_STEP);
                }
                if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
                    self.set_volume(self.volume + VOLUME_STEP);
                }
                if rl.is_key_pressed(KeyboardKey::KEY_M) {
                    self.muted = !self.muted;
                }
                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
                    && point_in_rectangle(mouse, slider_hit_area(screen))
                {
                    self.set_volume(slider_volume_at(mouse.x, volume_slider(screen)));
                } else if clicked && point_in_rectangle(mouse, sound_mute_button(screen)) {
                    self.muted = !self.muted;
                } else if clicked && point_in_rectangle(mouse, sound_back_button(screen)) {
                    self.screen = IntroScreen::Main;
                }
            }
        }
        IntroAction::None
    }

    fn choose_main_button(&mut self, index: usize) -> IntroAction {
        match index {
            0 => IntroAction::Start,
            1 => {
                self.screen = IntroScreen::Sound;
                IntroAction::None
            }
            2 => IntroAction::Exit,
            _ => IntroAction::None,
        }
    }

    pub(super) fn draw(&self, drawing: &mut RaylibDrawHandle<'_>) {
        let screen = (drawing.get_screen_width(), drawing.get_screen_height());
        let scale = intro_scale(screen);
        drawing.draw_rectangle_gradient_v(
            0,
            0,
            screen.0,
            screen.1,
            BACKGROUND_TOP,
            BACKGROUND_BOTTOM,
        );
        draw_stars(drawing, screen, scale);
        draw_orbit_decoration(drawing, screen, scale);

        match self.screen {
            IntroScreen::Main => self.draw_main(drawing, screen, scale),
            IntroScreen::Sound => self.draw_sound(drawing, screen, scale),
        }
    }

    fn draw_main(&self, drawing: &mut RaylibDrawHandle<'_>, screen: (i32, i32), scale: f32) {
        let title_size = (82.0 * scale).round() as i32;
        let first = "Proyecto 2 ";
        let second = "Gráficas";
        let total_width =
            drawing.measure_text(first, title_size) + drawing.measure_text(second, title_size);
        let start_x = (screen.0 - total_width) / 2;
        let title_y = (screen.1 as f32 * 0.22).round() as i32;
        draw_outlined_text(drawing, first, start_x, title_y, title_size, WARM_WHITE);
        draw_outlined_text(
            drawing,
            second,
            start_x + drawing.measure_text(first, title_size),
            title_y,
            title_size,
            BIRD_GREEN,
        );
        draw_text_centered(
            drawing,
            "Norman Aguirre",
            screen.0 / 2,
            title_y + (112.0 * scale).round() as i32,
            (30.0 * scale).round() as i32,
            WARM_WHITE,
        );

        for (index, label) in BUTTON_LABELS.into_iter().enumerate() {
            draw_menu_button(
                drawing,
                main_button(screen, index),
                label,
                self.focus == index,
                scale,
            );
        }
        draw_text_centered(
            drawing,
            "Flechas y Enter o click para elegir",
            screen.0 / 2,
            screen.1 - (56.0 * scale).round() as i32,
            (22.0 * scale).round() as i32,
            Color::new(202, 222, 244, 255),
        );
    }

    fn draw_sound(&self, drawing: &mut RaylibDrawHandle<'_>, screen: (i32, i32), scale: f32) {
        let title_size = (72.0 * scale).round() as i32;
        let title = "Sonido";
        let title_x = (screen.0 - drawing.measure_text(title, title_size)) / 2;
        draw_outlined_text(
            drawing,
            title,
            title_x,
            (screen.1 as f32 * 0.22).round() as i32,
            title_size,
            BIRD_GREEN,
        );
        let status = if self.muted {
            "Música: desactivada".to_string()
        } else {
            format!("Música: {} %", (self.volume * 100.0).round() as i32)
        };
        draw_text_centered(
            drawing,
            &status,
            screen.0 / 2,
            (screen.1 as f32 * 0.45).round() as i32,
            (30.0 * scale).round() as i32,
            WARM_WHITE,
        );
        let slider = volume_slider(screen);
        drawing.draw_rectangle_rounded(slider, 0.5, 8, Color::new(22, 45, 69, 255));
        drawing.draw_rectangle_rounded(
            Rectangle::new(
                slider.x,
                slider.y,
                slider.width * self.volume,
                slider.height,
            ),
            0.5,
            8,
            BIRD_GREEN,
        );
        drawing.draw_circle_v(
            Vector2::new(
                slider.x + slider.width * self.volume,
                slider.y + slider.height * 0.5,
            ),
            15.0 * scale,
            WARM_WHITE,
        );
        draw_menu_button(
            drawing,
            sound_mute_button(screen),
            if self.muted {
                "Activar música"
            } else {
                "Silenciar"
            },
            false,
            scale,
        );
        draw_menu_button(drawing, sound_back_button(screen), "Volver", false, scale);
        draw_text_centered(
            drawing,
            "Arrastra la barra o usa ← →  •  M: activar/silenciar",
            screen.0 / 2,
            screen.1 - (56.0 * scale).round() as i32,
            (21.0 * scale).round() as i32,
            Color::new(202, 222, 244, 255),
        );
    }
}

fn intro_scale(screen: (i32, i32)) -> f32 {
    (screen.0.max(1) as f32 / 1920.0)
        .min(screen.1.max(1) as f32 / 1080.0)
        .clamp(0.55, 2.0)
}

fn main_button(screen: (i32, i32), index: usize) -> Rectangle {
    let scale = intro_scale(screen);
    let width = 380.0 * scale;
    let height = 70.0 * scale;
    let y = screen.1 as f32 * 0.55 + index as f32 * 88.0 * scale;
    Rectangle::new((screen.0 as f32 - width) * 0.5, y, width, height)
}

fn main_button_at(mouse: Vector2, screen: (i32, i32)) -> Option<usize> {
    (0..BUTTON_LABELS.len()).find(|&index| point_in_rectangle(mouse, main_button(screen, index)))
}

fn volume_slider(screen: (i32, i32)) -> Rectangle {
    let scale = intro_scale(screen);
    let width = 420.0 * scale;
    Rectangle::new(
        (screen.0 as f32 - width) * 0.5,
        screen.1 as f32 * 0.55,
        width,
        14.0 * scale,
    )
}

fn slider_hit_area(screen: (i32, i32)) -> Rectangle {
    let slider = volume_slider(screen);
    let margin = 18.0 * intro_scale(screen);
    Rectangle::new(
        slider.x - margin,
        slider.y - margin,
        slider.width + 2.0 * margin,
        slider.height + 2.0 * margin,
    )
}

fn slider_volume_at(mouse_x: f32, slider: Rectangle) -> f32 {
    ((mouse_x - slider.x) / slider.width).clamp(0.0, 1.0)
}

fn sound_mute_button(screen: (i32, i32)) -> Rectangle {
    let scale = intro_scale(screen);
    let width = 310.0 * scale;
    Rectangle::new(
        (screen.0 as f32 - width) * 0.5,
        screen.1 as f32 * 0.65,
        width,
        64.0 * scale,
    )
}

fn sound_back_button(screen: (i32, i32)) -> Rectangle {
    let scale = intro_scale(screen);
    let width = 310.0 * scale;
    Rectangle::new(
        (screen.0 as f32 - width) * 0.5,
        screen.1 as f32 * 0.76,
        width,
        64.0 * scale,
    )
}

fn draw_menu_button(
    drawing: &mut RaylibDrawHandle<'_>,
    rectangle: Rectangle,
    label: &str,
    selected: bool,
    scale: f32,
) {
    let fill = if selected {
        Color::new(190, 237, 65, 255)
    } else {
        BIRD_GREEN
    };
    drawing.draw_rectangle_rounded(rectangle, 0.35, 10, fill);
    drawing.draw_rectangle_rounded_lines_ex(rectangle, 0.35, 10, 4.0 * scale, BIRD_GREEN_DARK);
    let font_size = (31.0 * scale).round() as i32;
    let text_y = (rectangle.y + (rectangle.height - font_size as f32) * 0.5).round() as i32;
    draw_text_centered(
        drawing,
        label,
        (rectangle.x + rectangle.width * 0.5).round() as i32,
        text_y,
        font_size,
        Color::new(25, 62, 20, 255),
    );
}

fn draw_stars(drawing: &mut RaylibDrawHandle<'_>, screen: (i32, i32), scale: f32) {
    for index in 0..80_u32 {
        let x = ((index.wrapping_mul(83).wrapping_add(29) % 997) as f32 / 997.0) * screen.0 as f32;
        let y = ((index.wrapping_mul(157).wrapping_add(43) % 991) as f32 / 991.0) * screen.1 as f32;
        let radius = if index % 9 == 0 { 2.2 } else { 1.1 } * scale;
        drawing.draw_circle_v(Vector2::new(x, y), radius, Color::new(208, 232, 255, 190));
    }
}

fn draw_orbit_decoration(drawing: &mut RaylibDrawHandle<'_>, screen: (i32, i32), scale: f32) {
    let center_x = (screen.0 as f32 * 0.88).round() as i32;
    let center_y = (screen.1 as f32 * 0.22).round() as i32;
    drawing.draw_circle(
        center_x,
        center_y,
        98.0 * scale,
        Color::new(67, 117, 170, 255),
    );
    drawing.draw_circle(
        center_x,
        center_y,
        79.0 * scale,
        Color::new(92, 156, 204, 255),
    );
    drawing.draw_circle_lines(
        center_x,
        center_y,
        128.0 * scale,
        Color::new(180, 228, 255, 90),
    );
    drawing.draw_circle_v(
        Vector2::new(
            center_x as f32 - 122.0 * scale,
            center_y as f32 + 39.0 * scale,
        ),
        11.0 * scale,
        BIRD_GREEN,
    );
}

#[cfg(test)]
mod tests {
    use super::{
        IntroAction, IntroMenu, IntroScreen, main_button, main_button_at, slider_volume_at,
        sound_back_button, sound_mute_button, volume_slider,
    };
    use raylib::prelude::Vector2;

    #[test]
    fn main_buttons_have_separate_click_targets() {
        let screen = (1920, 1080);
        for index in 0..3 {
            let button = main_button(screen, index);
            let center = Vector2::new(
                button.x + button.width * 0.5,
                button.y + button.height * 0.5,
            );
            assert_eq!(main_button_at(center, screen), Some(index));
        }
        assert_eq!(main_button_at(Vector2::new(0.0, 0.0), screen), None);
    }

    #[test]
    fn sound_menu_preserves_volume_when_muted() {
        let mut menu = IntroMenu::new();
        menu.choose_main_button(1);
        assert_eq!(menu.screen, IntroScreen::Sound);
        menu.set_volume(0.35);
        menu.muted = true;
        assert_eq!(menu.effective_volume(), 0.0);
        menu.muted = false;
        assert!((menu.effective_volume() - 0.35).abs() < 0.001);
        menu.set_volume(2.0);
        assert_eq!(menu.effective_volume(), 1.0);
    }

    #[test]
    fn slider_maps_edges_to_zero_and_full_volume() {
        let slider = volume_slider((800, 600));
        assert_eq!(slider_volume_at(slider.x - 50.0, slider), 0.0);
        assert_eq!(
            slider_volume_at(slider.x + slider.width + 50.0, slider),
            1.0
        );
        assert!((slider_volume_at(slider.x + slider.width * 0.5, slider) - 0.5).abs() < 0.001);
    }

    #[test]
    fn intro_navigation_targets_start_sound_and_exit() {
        let mut menu = IntroMenu::new();
        assert_eq!(menu.choose_main_button(0), IntroAction::Start);
        assert_eq!(menu.choose_main_button(1), IntroAction::None);
        assert_eq!(menu.screen, IntroScreen::Sound);
        assert_eq!(menu.choose_main_button(2), IntroAction::Exit);
        let screen = (800, 600);
        let mute = sound_mute_button(screen);
        let back = sound_back_button(screen);
        assert!(mute.y + mute.height < back.y);
    }
}
