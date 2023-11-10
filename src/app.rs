use eframe::egui;
use egui::{Sense, Rounding, Color32, Stroke, Pos2, Rect};

pub struct ImgProcDemo {
    src_grid: PixGrid,
}

enum Tool {
    Pen (u8),
    Conv,
}

struct PixGrid {
    width: u32,
    height: u32,
    pixels: Vec<Vec<u8>>,

    // ui stuff
    pix_size: u32,
}

impl PixGrid {
    fn new(width: u32, height: u32, init_color: u8, pix_size: u32) -> Self {
        let mut s = Self {
            width,
            height,
            pixels: Vec::with_capacity(height as usize),
            pix_size,
        };
        for y in 0..height {
            let mut row_vec = Vec::with_capacity(width as usize);
            for x in 0..width {
                row_vec.push(init_color);
            }
            s.pixels.push(row_vec);
        }
        s
    }

    /// get the value at the given position, clamped to (0, size).
    /// in other words, the image is extended at the edges.
    fn get_clamped(&self, x: i32, y: i32) -> u8 {
        let x = x.clamp(0, self.width as i32 - 1);
        let y = y.clamp(0, self.height as i32 - 1);
        self.pixels[y as usize][x as usize]
    }

    fn get_o(&self, x: u32, y: u32) -> Option<u8> {
        if x < self.width && y < self.height {
            Some(self.pixels[y as usize][x as usize])
        } else {
            None
        }
    }

    /// if x,y is a valid index, set the color.
    /// return if the color was set.
    fn try_set(&mut self, x: i32, y: i32, color: u8) -> bool {
        if x >= 0 && (x as u32) < self.width && y >= 0 && (y as u32) < self.height {
            self.pixels[y as usize][x as usize] = color;
            return true;
        }
        false
    }

    // for now just draw, no interactivity
    fn draw(&self, ui: &mut egui::Ui) {
        // use a fixed size
        let margin = 2;
        // to draw n pixel boxes, we want n * pixel size and (n+1) * margin size
        let desired_size = egui::vec2(
            (self.width * (self.pix_size + margin) + margin) as f32,
            (self.height * (self.pix_size + margin) + margin) as f32);
        let (rect, mut _response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        // check response for click

        if ui.is_rect_visible(rect) {
            // first, draw the background:
            let painter = ui.painter();
            painter.rect(rect, Rounding::ZERO, Color32::from_gray(20), Stroke::NONE);

            // draw the pixel boxes. could also cache them.
            for y_idx in 0..self.height {
                let y = ((margin + self.pix_size) * y_idx + margin) as f32 + rect.min.y ;
                for x_idx in 0..self.width {
                    let x = ((margin + self.pix_size) * x_idx + margin) as f32 + rect.min.x;
                    painter.rect(
                        Rect::from_min_max(Pos2::new(x, y), Pos2::new(x + self.pix_size as f32, y + self.pix_size as f32)),
                        Rounding::ZERO,
                        Color32::from_gray(self.pixels[y_idx as usize][x_idx as usize]),
                        Stroke::NONE
                    );
                }
            }
        }
    }
}



impl ImgProcDemo {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self {
            src_grid: PixGrid::new(20, 12, 200, 16),
        };
        s.src_grid.try_set(5, 2, 0);
        s
    }
}

impl eframe::App for ImgProcDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Image Processing Demo");
            self.src_grid.draw(ui);
        });
    }
}

