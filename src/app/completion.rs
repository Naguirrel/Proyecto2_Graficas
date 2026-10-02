//! Screen-space victory card. All decoration uses raylib's 2D drawing API.

use crate::game::Outcome;
use raylib::prelude::*;

const INK: Color = Color::new(9, 17, 41, 255);
const LIME: Color = Color::new(181, 237, 29, 255);
const BUTTON_RADIUS: f32 = 54.0;
const BUTTON_Y: f32 = 727.0;
const BUTTON_GAP: f32 = 147.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CompletionAction {
    Selector,
    Retry,
    NextWorld,
}

const ACTIONS: [CompletionAction; 3] = [
    CompletionAction::Selector,
    CompletionAction::Retry,
    CompletionAction::NextWorld,
];

/// A 600 x 900 design, scaled to fit even a narrow window.
#[derive(Clone, Copy)]
struct Layout {
    origin: Vector2,
    scale: f32,
}

impl Layout {
    fn new(screen: (i32, i32)) -> Self {
        let scale = (screen.1.max(1) as f32 / 900.0).min(screen.0.max(1) as f32 / 620.0);
        Self {
            origin: Vector2::new(
                screen.0 as f32 * 0.5,
                (screen.1 as f32 - 900.0 * scale) * 0.5,
            ),
            scale,
        }
    }

    fn at(self, x: f32, y: f32) -> Vector2 {
        self.origin + Vector2::new(x, y) * self.scale
    }

    fn button(self, index: usize) -> Vector2 {
        self.at((index as f32 - 1.0) * BUTTON_GAP, BUTTON_Y)
    }
}

/// Reference viewing takes priority; clicks outside the three circles do nothing.
pub(super) fn action(
    outcome: Option<Outcome>,
    reference_blocked: bool,
    retry_pressed: bool,
    click: Option<Vector2>,
    screen: (i32, i32),
) -> Option<CompletionAction> {
    if outcome != Some(Outcome::Won) || reference_blocked {
        return None;
    }
    if retry_pressed {
        return Some(CompletionAction::Retry);
    }
    let mouse = click?;
    let layout = Layout::new(screen);
    ACTIONS.into_iter().enumerate().find_map(|(index, action)| {
        (mouse.distance(layout.button(index)) <= BUTTON_RADIUS * layout.scale).then_some(action)
    })
}

pub(super) fn draw(
    drawing: &mut impl RaylibDraw,
    screen: (i32, i32),
    score: u32,
    stars: usize,
    mouse: Vector2,
    font: &WeakFont,
) {
    let layout = Layout::new(screen);
    let at = |x, y| layout.at(x, y);
    let scale = layout.scale;
    drawing.draw_rectangle(0, 0, screen.0, screen.1, Color::new(0, 5, 12, 190));

    // The broad top and tapered bottom reproduce the original victory banner.
    quad(
        drawing,
        [
            at(-280.0, 10.0),
            at(304.0, 10.0),
            at(234.0, 910.0),
            at(-210.0, 910.0),
        ],
        Color::new(0, 0, 0, 130),
    );
    quad(
        drawing,
        [
            at(-292.0, -12.0),
            at(292.0, -12.0),
            at(220.0, 912.0),
            at(-220.0, 912.0),
        ],
        Color::new(251, 251, 236, 255),
    );
    quad(
        drawing,
        [
            at(-270.0, 3.0),
            at(270.0, 3.0),
            at(200.0, 882.0),
            at(-200.0, 882.0),
        ],
        INK,
    );

    for index in 0..48 {
        let y = 24.0 + ((index * 137 + 23) % 837) as f32;
        let half_width = 247.0 - y * 0.065;
        let x = (((index * 79 + 11) % 101) as f32 / 50.0 - 1.0) * half_width;
        let center = at(x, y);
        let radius = if index % 11 == 0 { 3.0 } else { 1.3 } * scale;
        drawing.draw_circle_v(center, radius * 3.0, Color::new(91, 191, 237, 16));
        drawing.draw_circle_v(center, radius * 1.6, Color::new(119, 220, 251, 36));
        drawing.draw_circle_v(center, radius, Color::new(210, 243, 255, 220));
        if index % 11 == 0 {
            let r = radius * 3.3;
            quad(
                drawing,
                [
                    center + Vector2::new(0.0, -r),
                    center + Vector2::new(radius, 0.0),
                    center + Vector2::new(0.0, r),
                    center - Vector2::new(radius, 0.0),
                ],
                Color::new(190, 237, 255, 230),
            );
            quad(
                drawing,
                [
                    center - Vector2::new(r * 0.75, 0.0),
                    center - Vector2::new(0.0, radius),
                    center + Vector2::new(r * 0.75, 0.0),
                    center + Vector2::new(0.0, radius),
                ],
                Color::new(190, 237, 255, 230),
            );
        }
    }

    lettering(
        drawing,
        "NIVEL COMPLETADO!",
        at(0.0, 78.0),
        46.0 * scale,
        474.0 * scale,
    );

    for (index, (x, y, radius, rotation)) in [
        (-167.0, 325.0, 103.0, -0.13),
        (0.0, 333.0, 94.0, 0.02),
        (158.0, 348.0, 80.0, 0.13),
    ]
    .into_iter()
    .enumerate()
    {
        award_star(drawing, at(x, y), radius * scale, rotation, index < stars);
    }

    lettering(
        drawing,
        &score.to_string(),
        at(0.0, 492.0),
        62.0 * scale,
        350.0 * scale,
    );
    for (index, label) in ["MENU", "REINTENTAR", "SIGUIENTE"].into_iter().enumerate() {
        let center = layout.button(index);
        let hovered = mouse.distance(center) <= BUTTON_RADIUS * scale;
        button(drawing, center, scale, ACTIONS[index], hovered);
        small_label(
            drawing,
            font,
            label,
            at((index as f32 - 1.0) * BUTTON_GAP, 803.0),
            13.0 * scale,
            Color::new(229, 240, 251, 255),
        );
    }
    small_label(
        drawing,
        font,
        "Enter: reintentar  /  Backspace: menu",
        at(0.0, 851.0),
        11.0 * scale,
        Color::new(148, 176, 206, 255),
    );
}

fn triangle(d: &mut impl RaylibDraw, a: Vector2, b: Vector2, c: Vector2, color: Color) {
    d.draw_triangle(a, b, c, color);
    d.draw_triangle(a, c, b, color);
}

fn quad(d: &mut impl RaylibDraw, points: [Vector2; 4], color: Color) {
    triangle(d, points[0], points[1], points[2], color);
    triangle(d, points[0], points[2], points[3], color);
}

fn star_points(center: Vector2, radius: f32, rotation: f32) -> [Vector2; 10] {
    std::array::from_fn(|index| {
        let angle =
            -std::f32::consts::FRAC_PI_2 + rotation + index as f32 * std::f32::consts::PI / 5.0;
        let distance = radius * if index.is_multiple_of(2) { 1.0 } else { 0.46 };
        center + Vector2::new(angle.cos(), angle.sin()) * distance
    })
}

fn award_star(d: &mut impl RaylibDraw, center: Vector2, radius: f32, rotation: f32, earned: bool) {
    let points = star_points(center, radius, rotation);
    if !earned {
        for index in 0..10 {
            triangle(d, center, points[index], points[(index + 1) % 10], INK);
            d.draw_line_ex(
                points[index],
                points[(index + 1) % 10],
                radius * 0.025,
                Color::new(74, 88, 119, 255),
            );
        }
        return;
    }
    let depth = Vector2::new(radius * 0.17, radius * 0.14);
    for index in 0..10 {
        let a = points[index];
        let b = points[(index + 1) % 10];
        triangle(
            d,
            center + depth,
            a + depth,
            b + depth,
            Color::new(61, 35, 4, 255),
        );
        quad(
            d,
            [a, b, b + depth, a + depth],
            if index < 5 {
                Color::new(231, 133, 0, 255)
            } else {
                Color::new(183, 89, 0, 255)
            },
        );
    }
    for index in 0..10 {
        let color = [
            Color::new(255, 232, 91, 255),
            Color::new(255, 199, 25, 255),
            Color::new(255, 247, 151, 255),
            Color::new(255, 212, 43, 255),
        ][index % 4];
        triangle(d, center, points[index], points[(index + 1) % 10], color);
        d.draw_line_ex(
            points[index],
            points[(index + 1) % 10],
            radius * 0.023,
            Color::new(255, 243, 143, 255),
        );
    }
    d.draw_line_ex(
        points[8],
        points[9],
        radius * 0.044,
        Color::new(255, 255, 220, 255),
    );
    d.draw_line_ex(
        points[9],
        points[0],
        radius * 0.044,
        Color::new(255, 255, 220, 255),
    );
}

fn button(
    d: &mut impl RaylibDraw,
    center: Vector2,
    scale: f32,
    action: CompletionAction,
    hovered: bool,
) {
    let radius = BUTTON_RADIUS * scale;
    d.draw_circle_v(
        center + Vector2::new(1.0, 5.0) * scale,
        radius + 4.0 * scale,
        Color::new(0, 0, 0, 150),
    );
    for (inset, color) in [
        (-2.0, Color::new(41, 50, 35, 255)),
        (
            0.0,
            if hovered {
                Color::new(255, 251, 156, 255)
            } else {
                Color::WHITE
            },
        ),
        (5.0, Color::new(42, 84, 7, 255)),
        (8.0, Color::new(95, 173, 0, 255)),
    ] {
        d.draw_circle_v(center, radius - inset * scale, color);
    }
    d.draw_circle_sector(
        center,
        radius - 9.0 * scale,
        190.0,
        345.0,
        32,
        if hovered {
            Color::new(185, 240, 12, 255)
        } else {
            Color::new(155, 217, 0, 255)
        },
    );
    let at = |x, y| center + Vector2::new(x, y) * scale;
    let edge = Color::new(45, 82, 12, 255);
    match action {
        CompletionAction::Selector => {
            for y in [-20.0, 0.0, 20.0] {
                d.draw_circle_v(at(-19.0, y), 6.5 * scale, edge);
                d.draw_circle_v(at(-19.0, y), 4.5 * scale, Color::WHITE);
                d.draw_line_ex(at(-3.0, y), at(24.0, y), 12.0 * scale, edge);
                d.draw_line_ex(at(-3.0, y), at(24.0, y), 8.0 * scale, Color::WHITE);
            }
        }
        CompletionAction::Retry => {
            d.draw_ring(center, 19.0 * scale, 32.0 * scale, 50.0, 360.0, 48, edge);
            d.draw_ring(
                center,
                22.0 * scale,
                29.0 * scale,
                50.0,
                360.0,
                48,
                Color::WHITE,
            );
            triangle(d, at(12.0, 0.0), at(39.0, 0.0), at(28.0, 27.0), edge);
            triangle(
                d,
                at(16.0, 3.0),
                at(35.0, 3.0),
                at(28.0, 21.0),
                Color::WHITE,
            );
        }
        CompletionAction::NextWorld => {
            for x in [-23.0, 0.0] {
                triangle(
                    d,
                    at(x - 3.0, -29.0),
                    at(x + 30.0, 0.0),
                    at(x - 3.0, 29.0),
                    edge,
                );
                triangle(
                    d,
                    at(x, -23.0),
                    at(x + 26.0, 0.0),
                    at(x, 23.0),
                    Color::WHITE,
                );
            }
        }
    }
}

fn small_label(
    d: &mut impl RaylibDraw,
    font: &WeakFont,
    text: &str,
    center: Vector2,
    size: f32,
    color: Color,
) {
    let size = size.max(1.0);
    let spacing = size * 0.1;
    let width = font.measure_text(text, size, spacing).x;
    d.draw_text_ex(
        font,
        text,
        Vector2::new(center.x - width * 0.5, center.y),
        size,
        spacing,
        color,
    );
}

/// Chunky, slightly slanted vector lettering, without adding a font asset.
fn lettering(d: &mut impl RaylibDraw, text: &str, center: Vector2, height: f32, max_width: f32) {
    let advance = 0.76;
    let height = height.min(max_width / (text.len() as f32 * advance));
    let left = center.x - (text.len() as f32 * advance - 0.30) * height * 0.5;
    for (offset, color, weight) in [
        (Vector2::new(0.0, 0.06), Color::new(53, 96, 9, 255), 0.215),
        (Vector2::zero(), LIME, 0.18),
    ] {
        for (index, ch) in text.chars().enumerate() {
            let lean = if index % 3 == 0 { -0.035 } else { 0.035 };
            let at = |(x, y): (f32, f32)| {
                Vector2::new(
                    left + (index as f32 * advance + x * 0.46 + (1.0 - y) * lean + offset.x)
                        * height,
                    center.y + (y + offset.y) * height,
                )
            };
            for path in glyph(ch) {
                for pair in path.windows(2) {
                    d.draw_line_ex(at(pair[0]), at(pair[1]), weight * height, color);
                }
                for &point in *path {
                    d.draw_circle_v(at(point), weight * height * 0.5, color);
                }
            }
        }
    }
}

type Stroke = &'static [(f32, f32)];

fn glyph(ch: char) -> &'static [Stroke] {
    match ch {
        'N' => &[&[(0.0, 1.0), (0.0, 0.0), (1.0, 1.0), (1.0, 0.0)]],
        'I' => &[
            &[(0.5, 0.0), (0.5, 1.0)],
            &[(0.15, 0.0), (0.85, 0.0)],
            &[(0.15, 1.0), (0.85, 1.0)],
        ],
        'V' => &[&[(0.0, 0.0), (0.5, 1.0), (1.0, 0.0)]],
        'E' => &[
            &[(1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (1.0, 1.0)],
            &[(0.0, 0.5), (0.8, 0.5)],
        ],
        'L' => &[&[(0.0, 0.0), (0.0, 1.0), (1.0, 1.0)]],
        'C' => &[&[
            (1.0, 0.12),
            (0.75, 0.0),
            (0.2, 0.0),
            (0.0, 0.2),
            (0.0, 0.8),
            (0.2, 1.0),
            (0.8, 1.0),
            (1.0, 0.88),
        ]],
        'O' | '0' => &[&[
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.2),
            (1.0, 0.8),
            (0.8, 1.0),
            (0.2, 1.0),
            (0.0, 0.8),
            (0.0, 0.2),
            (0.2, 0.0),
        ]],
        'M' => &[&[(0.0, 1.0), (0.0, 0.0), (0.5, 0.45), (1.0, 0.0), (1.0, 1.0)]],
        'P' => &[&[
            (0.0, 1.0),
            (0.0, 0.0),
            (0.8, 0.0),
            (1.0, 0.15),
            (1.0, 0.4),
            (0.8, 0.55),
            (0.0, 0.55),
        ]],
        'T' => &[&[(0.0, 0.0), (1.0, 0.0)], &[(0.5, 0.0), (0.5, 1.0)]],
        'A' => &[
            &[(0.0, 1.0), (0.4, 0.0), (0.6, 0.0), (1.0, 1.0)],
            &[(0.2, 0.58), (0.8, 0.58)],
        ],
        'D' => &[&[
            (0.0, 1.0),
            (0.0, 0.0),
            (0.6, 0.0),
            (1.0, 0.2),
            (1.0, 0.8),
            (0.6, 1.0),
            (0.0, 1.0),
        ]],
        '!' => &[&[(0.5, 0.0), (0.5, 0.65)], &[(0.5, 0.94), (0.5, 1.0)]],
        '1' => &[
            &[(0.1, 0.22), (0.55, 0.0), (0.55, 1.0)],
            &[(0.1, 1.0), (1.0, 1.0)],
        ],
        '2' => &[&[
            (0.0, 0.15),
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.2),
            (0.9, 0.4),
            (0.0, 0.9),
            (0.0, 1.0),
            (1.0, 1.0),
        ]],
        '3' => &[&[
            (0.0, 0.0),
            (1.0, 0.0),
            (0.55, 0.45),
            (1.0, 0.6),
            (1.0, 0.85),
            (0.8, 1.0),
            (0.0, 1.0),
        ]],
        '4' => &[&[(0.75, 1.0), (0.75, 0.0), (0.0, 0.65), (1.0, 0.65)]],
        '5' => &[&[
            (1.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.45),
            (0.8, 0.45),
            (1.0, 0.6),
            (1.0, 0.85),
            (0.8, 1.0),
            (0.0, 1.0),
        ]],
        '6' => &[&[
            (0.85, 0.0),
            (0.3, 0.0),
            (0.0, 0.3),
            (0.0, 0.85),
            (0.2, 1.0),
            (0.8, 1.0),
            (1.0, 0.8),
            (1.0, 0.6),
            (0.8, 0.45),
            (0.0, 0.45),
        ]],
        '7' => &[&[(0.0, 0.0), (1.0, 0.0), (0.3, 1.0)]],
        '8' => &[&[
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.2),
            (0.85, 0.45),
            (0.15, 0.55),
            (0.0, 0.8),
            (0.2, 1.0),
            (0.8, 1.0),
            (1.0, 0.8),
            (0.85, 0.55),
            (0.15, 0.45),
            (0.0, 0.2),
            (0.2, 0.0),
        ]],
        '9' => &[&[
            (1.0, 0.55),
            (0.2, 0.55),
            (0.0, 0.4),
            (0.0, 0.2),
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.2),
            (1.0, 0.7),
            (0.7, 1.0),
            (0.1, 1.0),
        ]],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_buttons_select_their_actions_without_overlapping() {
        for screen in [(1920, 1080), (800, 600), (480, 270), (360, 640)] {
            let layout = Layout::new(screen);
            for (index, expected) in ACTIONS.into_iter().enumerate() {
                let center = layout.button(index);
                assert_eq!(
                    action(Some(Outcome::Won), false, false, Some(center), screen),
                    Some(expected)
                );
                let radius = BUTTON_RADIUS * layout.scale;
                assert!(center.x - radius >= 0.0 && center.x + radius <= screen.0 as f32);
                assert!(center.y - radius >= 0.0 && center.y + radius <= screen.1 as f32);
            }
            let gap = (layout.button(0) + layout.button(1)) * 0.5;
            assert_eq!(
                action(Some(Outcome::Won), false, false, Some(gap), screen),
                None
            );
        }
    }

    #[test]
    fn victory_actions_only_run_when_the_card_is_interactive() {
        let screen = (1280, 720);
        let click = Some(Layout::new(screen).button(2));
        for outcome in [None, Some(Outcome::Playing), Some(Outcome::Lost)] {
            assert_eq!(action(outcome, false, true, click, screen), None);
        }
        assert_eq!(action(Some(Outcome::Won), true, true, click, screen), None);
        assert_eq!(
            action(Some(Outcome::Won), false, true, None, screen),
            Some(CompletionAction::Retry)
        );
        assert_eq!(action(Some(Outcome::Won), false, false, None, screen), None);
    }
}
