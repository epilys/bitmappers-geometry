use bitmappers_companion::*;
use minifb::{CursorStyle, Key, MouseButton, MouseMode, Window, WindowOptions};

const WINDOW_WIDTH: usize = 400;
const WINDOW_HEIGHT: usize = 400;
include!("../bizcat.xbm.rs");

struct Bezier {
    points: Vec<Point>,
}

impl Bezier {
    fn new(points: Vec<Point>) -> Self {
        Bezier { points }
    }

    fn get_point(&self, t: f64) -> Option<Point> {
        draw_curve_point(&self.points, t)
    }
}

fn draw_curve_point(points: &[Point], t: f64) -> Option<Point> {
    if points.is_empty() {
        return None;
    }
    if points.len() == 1 {
        //std::dbg!(points[0]);
        return Some(points[0]);
    }
    let mut new_points = Vec::with_capacity(points.len() - 1);
    for chunk in points.windows(2) {
        let p1 = chunk[0];
        let p2 = chunk[1];
        let x = (1. - t) * (p1.0 as f64) + t * (p2.0 as f64);
        let y = (1. - t) * (p1.1 as f64) + t * (p2.1 as f64);
        new_points.push((x as i64, y as i64));
    }
    assert_eq!(new_points.len(), points.len() - 1);
    draw_curve_point(&new_points, t)
}

fn main() {
    let mut bizcat = Image::new(BIZCAT_WIDTH, BIZCAT_HEIGHT, 0, 0);
    bizcat.bytes = bits_to_bytes(BIZCAT_BITS, BIZCAT_WIDTH);
    let bizcat = BitmapFont::new(bizcat, (8, 16), 0, 0);

    let mut buffer: Vec<u32> = vec![WHITE; WINDOW_WIDTH * WINDOW_HEIGHT];
    let mut window = Window::new(
        "Bezier curves - ESC to exit",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions {
            title: true,
            //borderless: true,
            resize: true,
            //transparency: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut image = Image::new(WINDOW_WIDTH, WINDOW_HEIGHT, 0, 0);
    /* Construct an I glyph: */
    let mut curves = vec![];
    curves.push(Bezier::new(vec![(180, 75), (180, 350)]));
    curves.push(Bezier::new(vec![(130, 65), (166, 60), (180, 75)]));
    curves.push(Bezier::new(vec![(230, 75), (230, 350)]));
    curves.push(Bezier::new(vec![(230, 75), (235, 60), (280, 65)]));
    curves.push(Bezier::new(vec![(280, 50), (130, 50)]));
    curves.push(Bezier::new(vec![(280, 370), (130, 370)]));
    curves.push(Bezier::new(vec![(133, 360), (185, 365), (180, 350)]));
    curves.push(Bezier::new(vec![(230, 350), (230, 365), (280, 360)]));
    curves.push(Bezier::new(vec![(130, 65), (130, 50)]));
    curves.push(Bezier::new(vec![(280, 65), (280, 50)]));
    curves.push(Bezier::new(vec![(130, 360), (130, 370)]));
    curves.push(Bezier::new(vec![(280, 360), (280, 370)]));
    enum DragMode {
        Off { selected: Option<usize> },
        On { b: usize, i: usize, x: i64, y: i64 },
    }
    let is_pressed =
        |p: &Point, (x, y): Point| -> bool { (p.0 - x).abs() < 4 && (p.1 - y).abs() < 4 };

    let mut state = DragMode::Off { selected: None };

    while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
        image.clear();
        image.draw_grid(10);
        image.draw(&mut buffer, BLACK, Some(WHITE), WINDOW_WIDTH);
        match &mut state {
            DragMode::Off { selected } => {
                let mut selected = selected.clone();
                if let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) {
                    let x = x as i64;
                    let y = y as i64;
                    let mut set_default = true;
                    'curve_loop: for (b, c) in curves.iter().enumerate() {
                        for (i, p) in c.points.iter().enumerate() {
                            if is_pressed(p, (x, y)) {
                                if window.get_mouse_down(MouseButton::Left) {
                                    state = DragMode::On { b, i, x, y };
                                    selected = Some(b);
                                    window.set_cursor_style(CursorStyle::ClosedHand);
                                } else {
                                    selected = None;
                                    window.set_cursor_style(CursorStyle::OpenHand);
                                }
                                set_default = false;
                                break 'curve_loop;
                            }
                        }
                    }
                    if set_default {
                        window.set_cursor_style(CursorStyle::Arrow);
                    }
                }
                if let Some(b) = selected {
                    for p in &curves[b].points {
                        image.write_str(
                            &bizcat,
                            &format!("({}, {})", p.0, p.1),
                            (p.0, p.1 + bizcat.glyph_height as i64),
                        );
                    }
                }
            }
            DragMode::On {
                b,
                i,
                ref mut x,
                ref mut y,
            } => {
                if window.get_mouse_down(MouseButton::Left) {
                    if let Some((xn, yn)) = window.get_mouse_pos(MouseMode::Clamp) {
                        *x = xn as i64;
                        *y = yn as i64;
                        curves[*b].points[*i] = (*x, *y);
                    }
                    for p in &curves[*b].points {
                        image.write_str(
                            &bizcat,
                            &format!("({}, {})", p.0, p.1),
                            (p.0, p.1 + bizcat.glyph_height as i64),
                        );
                    }
                } else {
                    let b = *b;
                    state = DragMode::Off { selected: Some(b) };
                    window.set_cursor_style(CursorStyle::Arrow);
                }
            }
        }
        for c in &curves {
            for p in &c.points {
                image.plot_square(*p, 3, 0.);
            }
        }
        for c in &curves {
            let mut prev_point = c.points[0];
            let mut sample = 0;
            for t in (0..100).step_by(1) {
                let t = (t as f64) / 100.;
                if let Some(new_point) = c.get_point(t) {
                    if sample == 0 {
                        image.plot_line_width(prev_point, new_point, 2.);
                        sample = 5;
                        prev_point = new_point;
                    }
                    sample -= 1;
                }
            }
            image.plot_line_width(prev_point, *c.points.last().unwrap(), 2.);
        }
        image.draw(&mut buffer, BLACK, None, WINDOW_WIDTH);

        window
            .update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .unwrap();

        let millis = std::time::Duration::from_millis(100);

        std::thread::sleep(millis);
    }
}