use macroquad::prelude::*;

use crate::recipe::{self, Item, Recipe};

#[derive(Debug)]
struct ItemBox {
    item: Item,
    x: f32,
    y: f32,
}

impl ItemBox {
    fn new(item: Item) -> Self {
        unsafe {
            NEW_X += 22.0;
            NEW_Y += 16.0;

            if NEW_X >= screen_width() - 200.0 {
                NEW_X -= screen_width() - 210.0;
            }

            if NEW_Y >= screen_height() - 21.0 {
                NEW_Y = 50.0;
            }
        }

        Self {
            item,
            x: unsafe { NEW_X },
            y: unsafe { NEW_Y },
        }
    }

    fn mv(&mut self, by_x: f32, by_y: f32) {
        self.x += by_x;
        self.y += by_y;
    }

    fn draw(&self) {
        draw_text(&self.item.name, self.x, self.y, 20.0, WHITE);
        draw_rectangle_lines(
            self.x - 2.0,
            self.y - 15.0,
            self.item.name.len() as f32 * 10.0,
            22.0,
            2.0,
            RED,
        );
    }

    fn bounds(&self) -> Rect {
        Rect::new(
            self.x - 2.0,
            self.y - 15.0,
            self.item.name.len() as f32 * 10.0,
            22.0,
        )
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        let size = self.bounds();
        contains(&self.item.name, size.x, size.y, x, y)
    }
}

fn contains(name: &str, sx: f32, sy: f32, x: f32, y: f32) -> bool {
    let sw = name.len() as f32 * 10.0;
    let sh = 22.0;
    (sx <= x && sx + sw >= x) && (sy <= y && sy + sh >= y)
}

static mut NEW_X: f32 = 50.0;

static mut NEW_Y: f32 = 50.0;

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = recipe::Item::init()?;
    let mut available: Vec<Item> = items
        .iter()
        .filter(|i| i.recipes[0] == Recipe::Starter)
        .cloned()
        .collect();

    let mut boxes: Vec<ItemBox> = available.iter().map(|i| ItemBox::new(i.clone())).collect();

    let mut ty = 30.0;
    for item in &mut boxes {
        item.mv(50.0, ty);
        ty += 35.0;
    }

    let mut scroll = 0.0;

    let (mut mx, mut my) = mouse_position();
    let mut was_pressed = false;
    let mut is_pressed;
    loop {
        if is_mouse_button_down(MouseButton::Left) {
            is_pressed = true;
        } else {
            is_pressed = false;
        }

        let right = screen_width();
        let bottom = screen_height();

        if is_key_pressed(KeyCode::Escape) {
            #[cfg(not(target_family="wasm"))]
            return Ok(());
        } else if is_key_pressed(KeyCode::C) {
            boxes.clear();
        } else if is_key_pressed(KeyCode::Up) && scroll > 0.0 {
            scroll -= 30.0;
        } else if is_key_pressed(KeyCode::Down) {
            scroll += 30.0;
        }

        clear_background(DARKGRAY);

        let lines = textwrap::wrap("C to clear,  Up / Down arrows to scroll,  Esc to quit", ((right - 15.0) / 10.0) as usize);
        let start = bottom - (lines.len() as f32 * 15.0);
        for (i, line) in lines.iter().enumerate() {
            draw_text(line, 15.0, start + (i as f32 * 15.0), 20.0, BLUE);
        }

        for item in &boxes {
            item.draw();
        }

        let tx = right - 100.0;
        let mut ty = 30.0 - scroll;
        for item in &available {
            draw_text(&item.name, tx, ty, 20.0, GREEN);
            draw_rectangle_lines(
                tx - 2.0,
                ty - 15.0,
                item.name.len() as f32 * 10.0,
                22.0,
                4.0,
                RED,
            );

            ty += 23.0;
        }

        let (nx, ny) = mouse_position();
        if !was_pressed && is_pressed && nx >= tx {
            let mut ty = 30.0 - scroll;
            for item in &available {
                if contains(&item.name, tx - 2.0, ty - 15.0, nx, ny) {
                    boxes.push(ItemBox::new(item.clone()));
                    break;
                }

                ty += 23.0;
            }
        } else if (nx, ny) != (mx, my) {
            if is_pressed {
                for item in &mut boxes {
                    if item.contains(mx, my) {
                        item.mv(nx - mx, ny - my);
                        break;
                    }
                }
            }
        }

        if is_mouse_button_down(MouseButton::Right) {
            let mut remove = None;
            for (i, item) in boxes.iter().enumerate() {
                if item.contains(nx, ny) {
                    remove = Some(i);
                }
            }

            if let Some(i) = remove {
                boxes.remove(i);
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            let mut rects = Vec::with_capacity(boxes.len());
            for item in &boxes {
                rects.push(item.bounds());
            }

            let mut overlap = None;
            for (i1, rect1) in rects.iter().enumerate() {
                for (i2, rect2) in rects.iter().enumerate() {
                    if i1 == i2 {
                        continue;
                    }

                    if rect1.overlaps(rect2)
                        && rect1.contains(Vec2::new(nx, ny))
                        && rect2.contains(Vec2::new(nx, ny))
                    {
                        overlap = Some((i1, i2));
                        break;
                    }
                }

                if overlap.is_some() {
                    break;
                }
            }

            if let Some((i1, i2)) = overlap {
                let item1 = &boxes[i1];
                let item2 = &boxes[i2];

                let mut new_x = item1.x;
                let mut new_y = item1.y;
                let mut new = Vec::new();
                for item in &items {
                    if item.can_make(&item1.item, &item2.item) {
                        new.push(item.clone());
                    }
                }

                if !new.is_empty() {
                    boxes.remove(i1);

                    if i2 < i1 {
                        boxes.remove(i2);
                    } else {
                        boxes.remove(i2 - 1);
                    }
                }

                for item in new {
                    if !available.contains(&item) {
                        available.push(item.clone());
                    }

                    boxes.push(ItemBox {
                        item,
                        x: new_x,
                        y: new_y,
                    });

                    new_x += 10.0;
                    new_y += 24.0;
                }
            }
        }

        was_pressed = is_pressed;
        (mx, my) = (nx, ny);

        next_frame().await
    }
}
