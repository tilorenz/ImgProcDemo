use eframe::egui;
use crate::pix_grid::*;

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
                if let Some((ix, iy)) = src_grid.hovered_idx() {
                    src_grid.draw_outline(ui, ix, iy, ix, iy);
                    src_grid.draw_rect_at_idx(ui, ix, iy, tool_vars.pen_color);
                    if src_grid.pressed() {
                        src_grid.try_set(ix as i32, iy as i32, tool_vars.pen_color);
                    }
                }
            },
            Tool::Conv => {
                if let Some((ix, iy)) = src_grid.hovered_idx() {
                    let conv = &tool_vars.conv;
                    src_grid.draw_outline_clamped(ui, ix as i32 + conv.left, iy as i32 + conv.up, ix as i32 + conv.right, iy as i32 + conv.down);
                    dst_grid.draw_outline_clamped(ui, ix as i32, iy as i32, ix as i32, iy as i32);
                    let mut sum = 0.0;
                    if conv.zero_centered {
                        sum = 127.0
                    }
                    for y_offset in conv.up..=conv.down {
                        for x_offset in conv.left..=conv.right {
                            let x_conv_idx = (x_offset - conv.left) as usize;
                            let y_conv_idx = (y_offset - conv.up) as usize;
                            sum += conv.mask[y_conv_idx][x_conv_idx] * src_grid.get_clamped(ix as i32 + x_offset, iy as i32 + y_offset) as f32;
                        }
                    }
                    let color = sum.clamp(0.0, 255.0) as u8;
                    dst_grid.try_draw_rect_at_idx(ui, ix as i32, iy as i32, color);

                    if src_grid.pressed() {
                        dst_grid.try_set(ix as i32, iy as i32, color);
                    }
                }
            },
            Tool::Cpy => {
                if let Some((ix, iy)) = src_grid.hovered_idx() {
                    src_grid.draw_outline(ui, ix, iy, ix, iy);
                    dst_grid.draw_outline_clamped(ui, ix as i32, iy as i32, ix as i32, iy as i32);
                    let color = src_grid.get(ix, iy);
                    dst_grid.try_draw_rect_at_idx(ui, ix as i32, iy as i32, color);
                    if src_grid.pressed() {
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
    mask: Vec<Vec<f32>>,
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
                    mask: vec![ // binomial filter
                        vec![1.0/16.0, 2.0/16.0, 1.0/16.0],
                        vec![2.0/16.0, 4.0/16.0, 2.0/16.0],
                        vec![1.0/16.0, 2.0/16.0, 1.0/16.0],
                    ],
                }
            },
        };
        s.src_grid.try_set(5, 2, 0);
        s
    }

    fn pen_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool, Tool::Pen, "Pen");
            let mut color_proxy = self.tool_vars.pen_color as f32;
            let slider = egui::Slider::new(&mut color_proxy, 0.0..=255.0).text("Color").clamp_to_range(true);
            if ui.add(slider).changed() {
                self.tool_vars.pen_color = color_proxy.round() as u8;
            }
        });
    }
}

impl eframe::App for ImgProcDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Image Processing Demo");
            self.src_grid.draw(ui);
            self.dst_grid.draw(ui);
            self.tool.interact(ui, &mut self.tool_vars, &mut self.src_grid, &mut self.dst_grid);

            self.pen_row(ui);
            ui.selectable_value(&mut self.tool, Tool::Cpy, "Copy");
            ui.selectable_value(&mut self.tool, Tool::Conv, "Convolution");
        });
    }
}

