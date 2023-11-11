use eframe::egui;
use crate::pix_grid::*;

#[derive(PartialEq)]
enum Tool {
    Pen,
    Conv,
    Cpy,
    Boolean,
}

// the tools' parameters are stored in an own struct rather than inside the tool enum to make them
// persistent over tool changes (the pen should keep its color after switching to another tool...)
pub struct ToolVars {
    pen_color: u8,
    conv: Convolution,
    boolean_mask: [[bool; 3];3],
    boolean_dilation: bool,
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
                    src_grid.draw_outline_clamped(
                        ui,
                        ix as i32 + conv.left,
                        iy as i32 + conv.up,
                        ix as i32 + conv.right,
                        iy as i32 + conv.down);
                    dst_grid.draw_outline_clamped(ui, ix as i32, iy as i32, ix as i32, iy as i32);

                    let color = Tool::convolution(ix, iy, &conv, &src_grid);

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
            Tool::Boolean => {
                if let Some((ix, iy)) = src_grid.hovered_idx() {
                    let ix = ix as i32;
                    let iy = iy as i32;
                    for y_off in -1..=1 {
                        for x_off in -1..=1 {
                            if tool_vars.boolean_mask[(y_off + 1) as usize][(x_off + 1) as usize] {
                                src_grid.draw_outline_clamped(ui, ix + x_off, iy + y_off, ix + x_off, iy + y_off);
                            }
                        }
                    }
                    dst_grid.draw_outline_clamped(ui, ix, iy, ix, iy);
                    let color = Tool::bool_op(ix, iy, &tool_vars, &src_grid);
                    dst_grid.try_draw_rect_at_idx(ui, ix, iy, color);
                    if src_grid.pressed() {
                        dst_grid.try_set(ix, iy, color);
                    }
                }
            },
        }
    }

    fn bool_op(ix: i32, iy: i32, tool_vars: &ToolVars, src_grid: &PixGrid) -> u8 {
        // for erosion, we start with true and only stay true if all the values are
        // true, for dilation, we start with false and go true if any of the values is
        // true
        let mut b = !tool_vars.boolean_dilation;
        let threshold = 127;
        for y_off in -1..=1 {
            for x_off in -1..=1 {
                if tool_vars.boolean_mask[(y_off + 1) as usize][(x_off + 1) as usize] {
                    let s_val = src_grid.get_clamped(ix + x_off, iy + y_off);
                    if tool_vars.boolean_dilation {
                        b |= s_val > threshold;
                    } else {
                        b &= s_val > threshold;
                    }
                }
            }
        }
        if b {255} else {0}
    }

    fn convolution(ix: u32, iy: u32, conv: &Convolution, src_grid: &PixGrid) -> u8 {
        let mut sum = 0.0;
        if conv.zero_centered {
            sum = 127.0
        }
        for y_offset in conv.up..=conv.down {
            for x_offset in conv.left..=conv.right {
                let x_conv_idx = (x_offset - conv.left) as usize;
                let y_conv_idx = (y_offset - conv.up) as usize;
                sum += conv.mask[y_conv_idx][x_conv_idx] *
                    src_grid.get_clamped(ix as i32 + x_offset, iy as i32 + y_offset) as f32;
            }
        }
        sum.clamp(0.0, 255.0) as u8
    }

    fn apply_to_whole_image(&self, tool_vars: &ToolVars, src_grid: &mut PixGrid, dst_grid: &mut PixGrid) {
        match self {
            Tool::Pen => {
                src_grid.reset_to_color(tool_vars.pen_color);
            },
            Tool::Cpy => {
                for iy in 0..src_grid.height() {
                    for ix in 0..src_grid.width() {
                        let color = src_grid.get(ix, iy);
                        dst_grid.try_set(ix as i32, iy as i32, color);
                    }
                }
            },
            Tool::Conv => {
                for iy in 0..src_grid.height() {
                    for ix in 0..src_grid.width() {
                        let color = Tool::convolution(ix, iy, &tool_vars.conv, &src_grid);
                        dst_grid.try_set(ix as i32, iy as i32, color);
                    }
                }
            },
            Tool::Boolean => {
                for iy in 0..src_grid.height() {
                    for ix in 0..src_grid.width() {
                        let color = Tool::bool_op(ix as i32, iy as i32, &tool_vars, &src_grid);
                        dst_grid.try_set(ix as i32, iy as i32, color);
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
            src_grid: PixGrid::new(20, 12, 180, 16),
            dst_grid: PixGrid::new(20, 12, 180, 16),
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
                },
                boolean_mask: [
                    [false, true, false],
                    [true, true, true],
                    [false, true, false]
                ],
                boolean_dilation: true,
            },
        };
        s.src_grid.try_set(5, 2, 0);
        s
    }

    fn conv_binomial(&mut self) {
        let conv = &mut self.tool_vars.conv;
        conv.left = -1;
        conv.right = 1;
        conv.up = -1;
        conv.down = 1;
        conv.zero_centered = false;
        conv.mask = vec![ // binomial filter
                        vec![1.0/16.0, 2.0/16.0, 1.0/16.0],
                        vec![2.0/16.0, 4.0/16.0, 2.0/16.0],
                        vec![1.0/16.0, 2.0/16.0, 1.0/16.0],
        ];
    }
    fn conv_v_sobel(&mut self) {
        let conv = &mut self.tool_vars.conv;
        conv.left = -1;
        conv.right = 1;
        conv.up = -1;
        conv.down = 1;
        conv.zero_centered = true;
        conv.mask = vec![ // binomial filter
                        vec![-1.0, 0.0, 1.0],
                        vec![-2.0, 0.0, 2.0],
                        vec![-1.0, 0.0, 1.0],
        ];
    }
    fn conv_h_sobel(&mut self) {
        let conv = &mut self.tool_vars.conv;
        conv.left = -1;
        conv.right = 1;
        conv.up = -1;
        conv.down = 1;
        conv.zero_centered = true;
        conv.mask = vec![ // binomial filter
                        vec![1.0, 2.0, 1.0],
                        vec![0.0, 0.0, 0.0],
                        vec![-1.0, -2.0, -1.0],
        ];
    }


    fn pen_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool, Tool::Pen, "Pen");
            let mut color_proxy = self.tool_vars.pen_color as f32;
            let slider = egui::Slider::new(&mut color_proxy, 0.0..=255.0).text("Color").clamp_to_range(true);
            if ui.add(slider).changed() {
                self.tool_vars.pen_color = color_proxy.round() as u8;
                self.tool = Tool::Pen;
            }
        });
    }

    fn conv_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool, Tool::Conv, "Convolution");

            let conv = &mut self.tool_vars.conv;
            ui.horizontal(|ui| {
                for ix in conv.left..=conv.right {
                    ui.vertical(|ui| {
                        for iy in conv.up..=conv.down {
                            let slider = egui::Slider::new(
                                &mut conv.mask[(iy - conv.up) as usize][(ix - conv.left) as usize],
                                -2.0..=2.0);
                            if ui.add(slider).changed() {
                                self.tool = Tool::Conv;
                            }
                        }
                    });
                }
            });

            if ui.toggle_value(&mut conv.zero_centered, "Zero-centered").changed() {
                self.tool = Tool::Conv;
            }

            ui.vertical(|ui| {
                ui.label("Example filters:");
                if ui.button("Binomial filter").clicked() {
                    self.conv_binomial();
                    self.tool = Tool::Conv;
                }
                if ui.button("Vertical Sobel").clicked() {
                    self.conv_v_sobel();
                    self.tool = Tool::Conv;
                }
                if ui.button("Horizontal Sobel").clicked() {
                    self.conv_h_sobel();
                    self.tool = Tool::Conv;
                }
            });
        });
    }

    fn bool_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool, Tool::Boolean, "Boolean operation");
            if ui.toggle_value(&mut self.tool_vars.boolean_dilation, "Dilation").changed() {
                self.tool = Tool::Boolean;
            }
            let mut b_proxy = !self.tool_vars.boolean_dilation;
            if ui.toggle_value(&mut b_proxy, "Erosion").changed() {
                self.tool_vars.boolean_dilation = !b_proxy;
                self.tool = Tool::Boolean;
            }

            for ix in 0..=2 {
                ui.vertical(|ui| {
                    for iy in 0..=2 {
                        if ui.checkbox(&mut self.tool_vars.boolean_mask[iy][ix], "").changed() {
                            self.tool = Tool::Boolean;
                        }
                    }
                });
            }
        });
    }
}

impl eframe::App for ImgProcDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(egui::RichText::new("Image Processing Demo").strong().size(24.0));
            ui.horizontal(|ui| {
                // grid column
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Source Image:").size(16.0));
                    self.src_grid.draw(ui);
                    ui.label(""); // little spacer
                    ui.label(egui::RichText::new("Target Image:").size(16.0));
                    self.dst_grid.draw(ui);
                    self.tool.interact(ui, &mut self.tool_vars, &mut self.src_grid, &mut self.dst_grid);
                });

                // tools column
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Tool:").size(16.0));
                    self.pen_row(ui);
                    ui.selectable_value(&mut self.tool, Tool::Cpy, "Copy");
                    self.conv_row(ui);
                    self.bool_row(ui);

                    ui.label(egui::RichText::new("Actions:").size(16.0));
                    if ui.button("Reset").clicked() {
                        self.src_grid.reset_to_color(180);
                        self.dst_grid.reset_to_color(180);
                    }
                    if ui.button("Apply tool to whole image").clicked() {
                        self.tool.apply_to_whole_image(&self.tool_vars, &mut self.src_grid, &mut self.dst_grid);
                    }
                    if ui.button("Copy target to source").clicked() {
                        self.src_grid.copy_pixels_from(&mut self.dst_grid);
                    }
                });
            });
        });
    }
}

