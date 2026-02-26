// traclusdl_gui.rs - GUI rendering (sections and widgets)
// This file is mainly AI generated (Claude.ai)
// It's made to provide a minimal working GUI for users

use eframe::egui;
use eframe::egui::{RichText, ScrollArea, TextEdit, Vec2};
use rfd::FileDialog;

use crate::gui::style::*;
use crate::gui::traclusdl_app::TraclusDLApp;
use crate::gui::view_model::{ComputeMode, ParameterSet};

// ─────────────────────────────────────────────
// App Update (main render loop)
// ─────────────────────────────────────────────

impl eframe::App for TraclusDLApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events();

        // Request a repaint on next frame while a task is running so the
        // progress display stays live without user interaction
        if self.runner.is_running() {
            ctx.request_repaint();
        }

        // Apply dark theme overrides
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = COLOR_BACKGROUND;
        visuals.window_fill = COLOR_SECTION_BG;
        visuals.extreme_bg_color = COLOR_OUTPUT_BG;
        visuals.faint_bg_color = COLOR_SECTION_BG;
        visuals.widgets.inactive.fg_stroke.color = COLOR_TEXT;
        visuals.widgets.hovered.fg_stroke.color = COLOR_TEXT;
        visuals.widgets.active.fg_stroke.color = COLOR_TEXT;
        visuals.widgets.noninteractive.fg_stroke.color = COLOR_TEXT;
        visuals.widgets.open.fg_stroke.color = COLOR_TEXT;
        ctx.set_visuals(visuals);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(SECTION_SPACING);

            // All sections are placed inside a centered column of exactly CONTAINER_WIDTH.
            // We derive the left offset so every section starts at the same x coordinate.
            let panel_width = ui.available_width();
            let left_offset = ((panel_width - CONTAINER_WIDTH) / 2.0).max(0.0);

            let mut render = |ui: &mut egui::Ui| {
                render_file_section(ui, self);
                ui.add_space(SECTION_SPACING);

                render_parameters_section(ui, self);
                ui.add_space(SECTION_SPACING);

                render_computing_mode_section(ui, self);
                ui.add_space(SECTION_SPACING);

                ui.add(egui::Separator::default().horizontal().spacing(6.0));
                ui.add_space(4.0);

                render_output_section(ui, self);
                ui.add_space(SECTION_SPACING);

                render_action_bar(ui, self);
            };

            // Place a child UI of exactly CONTAINER_WIDTH at the computed left offset.
            // This guarantees that every section rendered inside shares the same x bounds.
            ui.horizontal(|ui| {
                ui.add_space(left_offset);
                ui.vertical(|ui| {
                    ui.set_width(CONTAINER_WIDTH);
                    render(ui);
                });
            });
        });
    }
}

// ─────────────────────────────────────────────
// Shared: container frame used by every section
// ─────────────────────────────────────────────

// Frame overhead = border (1 px each side) + inner margin (each side).
// The inner width passed to set_min_width must subtract this so the
// outer edge of every framed section lands at the same x.
const FRAME_OVERHEAD: f32 = (INNER_MARGIN + 1.0) * 2.0;
const INNER_WIDTH: f32 = CONTAINER_WIDTH - FRAME_OVERHEAD;

fn container_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(COLOR_SECTION_BG)
        .stroke(egui::Stroke::new(1.0, COLOR_BORDER))
        .rounding(CONTAINER_ROUNDING)
        .inner_margin(egui::Margin::same(INNER_MARGIN))
}

// ─────────────────────────────────────────────
// Section: File Input
// ─────────────────────────────────────────────

fn render_file_section(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    container_frame().show(ui, |ui| {
        ui.set_min_width(INNER_WIDTH);

        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("Input file name").color(COLOR_LABEL));
            ui.add_space(FILE_INPUT_WIDTH - 90.0 + SPACE_BETWEEN_FIELD * 2.0);
            ui.label(RichText::new("Number DL").color(COLOR_LABEL));
            ui.add_space(NUM_DL_WIDTH - 62.0 + SPACE_BETWEEN_FIELD * 2.0);
            ui.label(RichText::new("% correlation").color(COLOR_LABEL));
        });

        ui.add_space(WIDGET_SPACING);

        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut app.vm.input_name)
                    .desired_width(FILE_INPUT_WIDTH)
                    .clip_text(true)
                    .interactive(false)
                    .text_color(COLOR_TEXT),
            );
            ui.add_space(SPACE_BETWEEN_FIELD);

            let mut num_str = app.vm.num_dl.to_string();
            ui.add(
                TextEdit::singleline(&mut num_str)
                    .desired_width(NUM_DL_WIDTH)
                    .interactive(false)
                    .text_color(COLOR_TEXT),
            );
            ui.add_space(SPACE_BETWEEN_FIELD);

            let mut pct_str = app.vm.percent_correlation.to_string();
            ui.add(
                TextEdit::singleline(&mut pct_str)
                    .desired_width(PERCENT_CORR_WIDTH)
                    .interactive(false)
                    .text_color(COLOR_TEXT),
            );
            ui.add_space(SPACE_BETWEEN_FIELD);

            if ui
                .add_sized(
                    [BROWSE_BTN_WIDTH, BROWSE_BTN_HEIGHT],
                    egui::Button::new("Browse File"),
                )
                .clicked()
            {
                if let Some(path) = FileDialog::new()
                    .add_filter("Text file", &["txt"])
                    .pick_file()
                {
                    app.on_browse_done(path);
                }
            }
        });
    });
}

// ─────────────────────────────────────────────
// Section: Parameters
// ─────────────────────────────────────────────

fn render_parameters_section(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    container_frame().show(ui, |ui| {
        ui.set_min_width(INNER_WIDTH);
        ui.set_max_width(INNER_WIDTH);

        // Exact pixel budget:
        // INNER_WIDTH = param_col + separator(~6) + add_col
        // add_col holds a 36px button with 4px padding each side.
        let add_col_width: f32 = 44.0;
        let sep_width: f32 = 6.0;
        let param_col_width = INNER_WIDTH - add_col_width - sep_width;

        ui.horizontal(|ui| {
            // ── Left: headers + stacked parameter rows ───────────────────
            ui.vertical(|ui| {
                ui.set_width(param_col_width);

                // Header labels sized exactly like their corresponding fields
                ui.horizontal(|ui| {
                    ui.add_space(26.0); // aligns with "#N " prefix below
                    for label in &["MAX ANGLE", "MIN DENSITY", "MAX DISTANCE", "SEG SIZE"] {
                        ui.add_sized(
                            [PARAM_FIELD_WIDTH, 16.0],
                            egui::Label::new(
                                RichText::new(*label).small().color(COLOR_LABEL).strong(),
                            ),
                        );
                        ui.add_space(WIDGET_SPACING);
                    }
                });

                ui.separator();

                let mut to_remove: Option<usize> = None;
                let set_count = app.vm.parameter_sets.len();
                for (idx, param) in app.vm.parameter_sets.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("#{}", idx + 1)).color(COLOR_TEXT));
                        ui.add_space(4.0);

                        // max_angle -> f64 [0.0, 22.5]; stored as tenths-of-degree: [0, 225]
                        commit_on_focus_loss(
                            ui,
                            &mut param.buf_angle,
                            &mut param.max_angle,
                            PARAM_FIELD_WIDTH,
                            0,
                            225,
                        );
                        ui.add_space(WIDGET_SPACING);

                        // min_density: u32 >= 1
                        commit_on_focus_loss(
                            ui,
                            &mut param.buf_density,
                            &mut param.min_density,
                            PARAM_FIELD_WIDTH,
                            1,
                            i32::MAX,
                        );
                        ui.add_space(WIDGET_SPACING);

                        // max_distance: u32 >= 0
                        commit_on_focus_loss(
                            ui,
                            &mut param.buf_distance,
                            &mut param.max_distance,
                            PARAM_FIELD_WIDTH,
                            0,
                            i32::MAX,
                        );
                        ui.add_space(WIDGET_SPACING);

                        // seg_size: u32 >= 1
                        commit_on_focus_loss(
                            ui,
                            &mut param.buf_seg,
                            &mut param.seg_size,
                            PARAM_FIELD_WIDTH,
                            1,
                            i32::MAX,
                        );

                        if set_count > 1 && ui.small_button(" - ").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                }
                if let Some(idx) = to_remove {
                    app.vm.parameter_sets.remove(idx);
                }
            });

            ui.separator();

            // ── Right: + button in a hard-budgeted column ────────────────
            ui.allocate_ui(Vec2::new(add_col_width, ui.available_height()), |ui| {
                ui.centered_and_justified(|ui| {
                    if ui.add_sized([36.0, 36.0], egui::Button::new("+")).clicked() {
                        app.vm.parameter_sets.push(ParameterSet::default());
                    }
                });
            });
        });
    });
}

// Editable integer field: typing updates the buffer freely.
// Clamping and commit to the real value happen only on focus loss or Enter.
fn commit_on_focus_loss(
    ui: &mut egui::Ui,
    buf: &mut String,
    value: &mut i32,
    width: f32,
    min: i32,
    max: i32,
) {
    let response = ui.add(
        TextEdit::singleline(buf)
            .desired_width(width)
            .clip_text(true)
            .text_color(COLOR_TEXT),
    );

    let commit = response.lost_focus()
        || (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));

    if commit {
        match buf.parse::<i32>() {
            Ok(v) => {
                *value = v.clamp(min, max);
                *buf = value.to_string(); // normalise buffer to clamped value
            }
            Err(_) => {
                *buf = value.to_string(); // restore buffer to last valid value
            }
        }
    }
}

// ─────────────────────────────────────────────
// Section: Computing Mode
// ─────────────────────────────────────────────

fn render_computing_mode_section(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    container_frame().show(ui, |ui| {
        ui.set_min_width(CONTAINER_WIDTH - 5.0); // compensate for frame horizontal padding

        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.radio_value(
                &mut app.vm.compute_mode,
                ComputeMode::Serial,
                RichText::new("Serial computing").color(COLOR_TEXT),
            );
            ui.add_space(40.0);
            let parallel_label = format!("Parallel computing ({} CPU detected)", app.detected_cpus);
            ui.radio_value(
                &mut app.vm.compute_mode,
                ComputeMode::Parallel,
                RichText::new(parallel_label).color(COLOR_TEXT),
            );
        });
    });
}

// ─────────────────────────────────────────────
// Section: Outputs
// ─────────────────────────────────────────────

fn render_output_section(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    ui.label(RichText::new("OUTPUTS").color(COLOR_LABEL).strong());
    ui.add_space(4.0);

    egui::Frame::none()
        .fill(COLOR_OUTPUT_BG)
        .stroke(egui::Stroke::new(1.0, COLOR_BORDER))
        .rounding(CONTAINER_ROUNDING)
        .inner_margin(egui::Margin::same(INNER_MARGIN))
        .show(ui, |ui| {
            ui.set_min_width(INNER_WIDTH);

            ScrollArea::vertical()
                .max_height(OUTPUT_BOX_HEIGHT)
                .min_scrolled_height(OUTPUT_BOX_HEIGHT)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Subtract scrollbar width (~12 px) so text doesn't clip under it
                    ui.set_min_width(INNER_WIDTH - 12.0);
                    if app.output_text.is_empty() {
                        ui.label(
                            RichText::new("No output yet.")
                                .color(egui::Color32::GRAY)
                                .italics(),
                        );
                    } else {
                        ui.label(RichText::new(&app.output_text).color(COLOR_TEXT));
                    }
                });
        });
}

// ─────────────────────────────────────────────
// Section: Action Bar
// ─────────────────────────────────────────────

fn render_action_bar(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    // No border frame — just enforce the same CONTAINER_WIDTH so buttons
    // align with the left and right edges of every framed container above.
    ui.set_min_width(CONTAINER_WIDTH);

    ui.horizontal(|ui| {
        if ui
            .add_sized(
                [ACTION_BTN_WIDTH, ACTION_BTN_HEIGHT],
                egui::Button::new("Start\nComputation"),
            )
            .clicked()
        {
            app.on_start_computation();
        }

        ui.add_space(16.0);

        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("Estimated time:").color(COLOR_LABEL));
            ui.label(RichText::new("-- min --s").color(COLOR_TEXT));
        });

        // Stop + Create output flush to the right edge of CONTAINER_WIDTH
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_sized(
                    [ACTION_BTN_WIDTH, ACTION_BTN_HEIGHT],
                    egui::Button::new("Create output"),
                )
                .clicked()
            {
                println!("Create output clicked");
            }
            ui.add_space(8.0);
            if ui
                .add_sized([80.0, ACTION_BTN_HEIGHT], egui::Button::new("Stop"))
                .clicked()
            {
                println!("Stop clicked");
            }
        });
    });
}
