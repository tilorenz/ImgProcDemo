use eframe::egui;
use egui::{Sense, Rounding, Color32, Stroke, Pos2, Rect};

#[derive(PartialEq)]
enum Tool {
    Pen,
    Conv,
    Cpy,
}

// the tools' parameters are stored in an own struct rather than inside the tool enum to make them
// persistent over tool changes (the pen should keep its color after switching to another tool...)
pub struct ToolVars {
    pen_color: u8,
    conv: Convolution,
}

impl Tool {
    fn interact(&self, ui: &mut egui::Ui, tool_vars: &ToolVars, src_grid: &mut PixGrid, dst_grid: &mut PixGrid) {
        match self {
            Tool::Pen => {
                if let Some((ix, iy)) = src_grid.hovered_idx {
                    src_grid.draw_outline(ui, ix, iy, ix, iy);
                    src_grid.draw_rect_at_idx(ui, ix, iy, tool_vars.pen_color);
                    if src_grid.pressed {
                        src_grid.try_set(ix as i32, iy as i32, tool_vars.pen_color);
                    }
                }
            },
            Tool::Conv => {
                if let Some((ix, iy)) = src_grid.hovered_idx {
                    let conv = &tool_vars.conv;
                    src_grid.draw_outline_clamped(ui, ix as i32 + conv.left, iy as i32 + conv.up, ix as i32 + conv.right, iy as i32 + conv.down);
                    if src_grid.pressed {
                        let mut sum = 0.0;
                        for y_offset in conv.up..=conv.down {
                            for x_offset in conv.left..=conv.right {
                                let x_conv_idx = (x_offset - conv.left) as usize;
                                let y_conv_idx = (y_offset - conv.up) as usize;
                                sum += conv.mask[y_conv_idx][x_conv_idx] * src_grid.get_clamped(ix as i32 + x_offset, iy as i32 + y_offset) as f32;
                            }
                        }
                        let color = sum.clamp(0.0, 255.0) as u8;
                        dst_grid.try_set(ix as i32, iy as i32, color);
                    }
                }
            },
            Tool::Cpy => {
                if let Some((ix, iy)) = src_grid.hovered_idx {
                    src_grid.draw_outline(ui, ix, iy, ix, iy);
                    let color = src_grid.get(ix, iy);
                    dst_grid.try_draw_rect_at_idx(ui, ix as i32, iy as i32, color);
                    if src_grid.pressed {
                        dst_grid.try_set(ix as i32, iy as i32, src_grid.get_clamped(ix as i32, iy as i32));
                    }
                }
            },
        }
    }
}

struct Convolution {
    zero_centered: bool,
    left: i32,
    right: i32,
    up: i32,
    down: i32,
    mask: [[f32; 3]; 3],
}

struct PixGrid {
    width: u32,
    height: u32,
    pixels: Vec<Vec<u8>>,

    // ui stuff
    pix_size: u32,
    margin: u32,
    hovered_idx: Option<(u32, u32)>,
    pressed: bool,
    rect: Rect,
}

impl PixGrid {
    fn new(width: u32, height: u32, init_color: u8, pix_size: u32) -> Self {
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
        for y in 0..height {
            let mut row_vec = Vec::with_capacity(width as usize);
            for x in 0..width {
                row_vec.push(init_color);
            }
            s.pixels.push(row_vec);
        }
        s
    }

    /// get the value without any tests
    fn get(&self, x: u32, y: u32) -> u8 {
        self.pixels[y as usize][x as usize]
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

    fn draw_outline(&self, ui: &mut egui::Ui, from_ix: u32, from_iy: u32, to_ix: u32, to_iy: u32) {
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

    fn draw_outline_clamped(&self, ui: &mut egui::Ui, from_ix: i32, from_iy: i32, to_ix: i32, to_iy: i32) {
        //println!("before: {} {}, {} {} (as: {} {})", from_ix, from_iy, to_ix, to_iy, from_ix as u32, from_iy as u32);
        let from_ix = from_ix.clamp(0, self.width as i32 - 1) as u32;
        let from_iy = from_iy.clamp(0, self.height as i32 - 1) as u32;
        let to_ix = to_ix.clamp(0, self.width as i32 - 1) as u32;
        let to_iy = to_iy.clamp(0, self.height as i32 - 1) as u32;
        //println!("after: {} {}, {} {}", from_ix, from_iy, to_ix, to_iy);
        self.draw_outline(ui, from_ix, from_iy, to_ix, to_iy);
    }

    fn try_draw_rect_at_idx(&self, ui: &mut egui::Ui, ix: i32, iy: i32, color: u8) {
        if ix >= 0 && (ix as u32) < self.width && iy >= 0 && (iy as u32) < self.height {
            self.draw_rect_at_idx(ui, ix as u32, iy as u32, color);
        }
    }

    fn draw_rect_at_idx(&self, ui: &mut egui::Ui, ix: u32, iy: u32, color: u8) {
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
    fn draw(&mut self, ui: &mut egui::Ui) {
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
}


pub struct ImgProcDemo {
    src_grid: PixGrid,
    dst_grid: PixGrid,
    tool: Tool,
    tool_vars: ToolVars,
}

impl ImgProcDemo {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self {
            src_grid: PixGrid::new(20, 12, 200, 16),
            dst_grid: PixGrid::new(20, 12, 100, 16),
            tool: Tool::Pen,
            tool_vars: ToolVars {
                pen_color: 50,
                conv: Convolution {
                    zero_centered: false, left: -1, right: 1, up: -1, down: 1,
                    mask: [ // binomial filter
                        [1.0/16.0, 2.0/16.0, 1.0/16.0],
                        [2.0/16.0, 4.0/16.0, 2.0/16.0],
                        [1.0/16.0, 2.0/16.0, 1.0/16.0],
                    ],
                }
            },
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
            self.dst_grid.draw(ui);
            self.tool.interact(ui, &mut self.tool_vars, &mut self.src_grid, &mut self.dst_grid);

            ui.selectable_value(&mut self.tool, Tool::Pen, "Pen");
            ui.selectable_value(&mut self.tool, Tool::Cpy, "Copy");
            ui.selectable_value(&mut self.tool, Tool::Conv, "Convolution");
        });
    }
}

