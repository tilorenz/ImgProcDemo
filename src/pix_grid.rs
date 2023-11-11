use egui::{Sense, Rounding, Color32, Stroke, Pos2, Rect};

pub struct PixGrid {
    width: u32,
    height: u32,
    pixels: Vec<Vec<u8>>,

    // ui stuff
    pub pix_size: u32,
    pub margin: u32,
    hovered_idx: Option<(u32, u32)>,
    pressed: bool,
    rect: Rect,
}

impl PixGrid {
    pub fn new(width: u32, height: u32, init_color: u8, pix_size: u32) -> Self {
        let mut s = Self {
            width,
            height,
            pixels: Vec::with_capacity(height as usize),
            pix_size,
            margin: 3,
            hovered_idx: None,
            pressed: false,
            rect: Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(0.0, 0.0)),
        };
        for _y in 0..height {
            let mut row_vec = Vec::with_capacity(width as usize);
            for _x in 0..width {
                row_vec.push(init_color);
            }
            s.pixels.push(row_vec);
        }
        s
    }

    /// get the value without any tests
    pub fn get(&self, x: u32, y: u32) -> u8 {
        self.pixels[y as usize][x as usize]
    }

    /// get the value at the given position, clamped to (0, size).
    /// in other words, the image is extended at the edges.
    pub fn get_clamped(&self, x: i32, y: i32) -> u8 {
        let x = x.clamp(0, self.width as i32 - 1);
        let y = y.clamp(0, self.height as i32 - 1);
        self.pixels[y as usize][x as usize]
    }

    pub fn get_o(&self, x: u32, y: u32) -> Option<u8> {
        if x < self.width && y < self.height {
            Some(self.pixels[y as usize][x as usize])
        } else {
            None
        }
    }

    /// if x,y is a valid index, set the color.
    /// return if the color was set.
    pub fn try_set(&mut self, x: i32, y: i32, color: u8) -> bool {
        if x >= 0 && (x as u32) < self.width && y >= 0 && (y as u32) < self.height {
            self.pixels[y as usize][x as usize] = color;
            return true;
        }
        false
    }

    pub fn draw_outline(&self, ui: &mut egui::Ui, from_ix: u32, from_iy: u32, to_ix: u32, to_iy: u32) {
        // instead of drawing invividual lines, draw a rect in the background...
        let br_rect = Rect::from_min_max(
            Pos2::new(
                (from_ix * (self.pix_size + self.margin)) as f32 + self.rect.min.x,
                (from_iy * (self.pix_size + self.margin)) as f32 + self.rect.min.y),
            Pos2::new(
                ((to_ix + 1) * self.pix_size + (to_ix + 2) * self.margin) as f32 + self.rect.min.x,
                ((to_iy + 1) * self.pix_size + (to_iy + 2) * self.margin) as f32 + self.rect.min.y)
        );
        ui.painter().rect(br_rect, Rounding::ZERO, Color32::from_rgb(20, 200, 20), Stroke::NONE);

        // then redraw the boxes on top
        for iy in from_iy..=to_iy {
            for ix in from_ix..=to_ix {
                let color = self.get(ix, iy);
                self.draw_rect_at_idx(ui, ix, iy, color);
            }
        }
    }

    pub fn draw_outline_clamped(&self, ui: &mut egui::Ui, from_ix: i32, from_iy: i32, to_ix: i32, to_iy: i32) {
        //println!("before: {} {}, {} {} (as: {} {})", from_ix, from_iy, to_ix, to_iy, from_ix as u32, from_iy as u32);
        let from_ix = from_ix.clamp(0, self.width as i32 - 1) as u32;
        let from_iy = from_iy.clamp(0, self.height as i32 - 1) as u32;
        let to_ix = to_ix.clamp(0, self.width as i32 - 1) as u32;
        let to_iy = to_iy.clamp(0, self.height as i32 - 1) as u32;
        //println!("after: {} {}, {} {}", from_ix, from_iy, to_ix, to_iy);
        self.draw_outline(ui, from_ix, from_iy, to_ix, to_iy);
    }

    pub fn try_draw_rect_at_idx(&self, ui: &mut egui::Ui, ix: i32, iy: i32, color: u8) {
        if ix >= 0 && (ix as u32) < self.width && iy >= 0 && (iy as u32) < self.height {
            self.draw_rect_at_idx(ui, ix as u32, iy as u32, color);
        }
    }

    pub fn draw_rect_at_idx(&self, ui: &mut egui::Ui, ix: u32, iy: u32, color: u8) {
        let x = ((self.margin + self.pix_size) * ix + self.margin) as f32 + self.rect.min.x;
        let y = ((self.margin + self.pix_size) * iy + self.margin) as f32 + self.rect.min.y ;
        ui.painter().rect(
            Rect::from_min_max(Pos2::new(x, y), Pos2::new(x + self.pix_size as f32, y + self.pix_size as f32)),
            Rounding::ZERO,
            Color32::from_gray(color),
            Stroke::NONE
        );
    }

    // for now just draw, no interactivity
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        // to draw n pixel boxes, we want n * pixel size and (n+1) * margin size
        let desired_size = egui::vec2(
            (self.width * (self.pix_size + self.margin) + self.margin) as f32,
            (self.height * (self.pix_size + self.margin) + self.margin) as f32);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
        self.rect = rect;

        // check for hover and mouse press (button down, not just click so tools can be dragged)
        if let Some(pos) = response.hover_pos() {
            let pos = pos - self.rect.min.to_vec2();
            // clamping in case the pointer is on the last margin
            let ix = (pos.x as u32 / (self.pix_size + self.margin)).clamp(0, self.width - 1);
            let iy = (pos.y as u32 / (self.pix_size + self.margin)).clamp(0, self.height - 1);
            self.hovered_idx = Some((ix, iy));
            self.pressed = response.is_pointer_button_down_on();
        } else {
            self.hovered_idx = None;
            self.pressed = false;
        }

        if ui.is_rect_visible(self.rect) {
            // first, draw the background:
            let painter = ui.painter();
            painter.rect(self.rect, Rounding::ZERO, Color32::from_gray(20), Stroke::NONE);

            // draw the pixel boxes. could also cache them.
            for iy in 0..self.height {
                for ix in 0..self.width {
                    self.draw_rect_at_idx(ui, ix, iy, self.pixels[iy as usize][ix as usize]);
                }
            }
        }
    }

    /// sets the whole image to color
    pub fn reset_to_color(&mut self, color: u8) {
        for iy in 0..self.height {
            for ix in 0..self.width {
                self.pixels[iy as usize][ix as usize] = color;
            }
        }
    }

    pub fn pressed(&self) -> bool { self.pressed }
    pub fn hovered_idx(&self) -> Option<(u32, u32)> { self.hovered_idx }
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
}

