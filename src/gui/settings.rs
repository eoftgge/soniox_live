use crate::gui::state::{PendingState, StateManager};
use crate::settings::SettingsApp;
use crate::types::device::MappableAvailableDevices;
use crate::types::languages::LanguageHint;
use crate::types::tracing::LEVELS;
use eframe::egui::{self, Button, ComboBox, Context, DragValue, Grid, RichText, ScrollArea, Slider, TextEdit, Ui, vec2, Checkbox};
use eframe::epaint::Color32;
use egui_notify::Toasts;
use std::time::Duration;

pub fn show_settings_window(
    ctx: &Context,
    settings: &mut SettingsApp,
    manager: &mut StateManager,
    toasts: &mut Toasts,
    devices: &mut MappableAvailableDevices,
) {
    ui_bottom_panel(ctx, settings, manager, toasts);

    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(15.0))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 12.0);
            ui.heading("Settings");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                ui_section_app(ui, settings, devices);
                ui_section_api(ui, settings);
                ui_section_position(ui, ctx, settings);
                ui_section_appearance(ui, settings);
                ui.allocate_space(vec2(0.0, 60.0));
            });
        });
}

fn ui_bottom_panel(
    ctx: &Context,
    settings: &mut SettingsApp,
    manager: &mut StateManager,
    toasts: &mut Toasts,
) {
    egui::TopBottomPanel::bottom("settings_bottom_panel")
        .resizable(false)
        .min_height(60.0)
        .show(ctx, |ui| {
            ui.add_space(15.0);
            ui.columns(2, |cols| {
                cols[0].vertical_centered_justified(|ui| {
                    if ui
                        .add(Button::new("💾 Save").min_size(vec2(0.0, 40.0)))
                        .clicked()
                    {
                        match settings.save("soniox.toml") {
                            Ok(_) => {
                                toasts
                                    .success("Settings saved successfully!")
                                    .duration(Duration::from_secs(3))
                                    .closable(false);
                            }
                            Err(e) => {
                                toasts
                                    .error(format!("Failed to save: {}", e))
                                    .duration(Duration::from_secs(5))
                                    .closable(false);
                            }
                        }
                    }
                });

                cols[1].vertical_centered_justified(|ui| {
                    if ui
                        .add(Button::new("🚀 Start").min_size(vec2(0.0, 40.0)))
                        .clicked()
                    {
                        manager.switch(PendingState::Overlay);
                        toasts.info("Starting subtitles overlay...").closable(false);
                    }
                });
            });
            ui.add_space(10.0);
        });
}

fn ui_section_app(ui: &mut Ui, settings: &mut SettingsApp, devices: &mut MappableAvailableDevices) {
    ui.collapsing("Configuration App", |ui| {
        Grid::new("app_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Log Level:");
                    ComboBox::from_id_salt("log_level")
                        .selected_text(settings.level.to_string())
                        .width(40.0)
                        .show_ui(ui, |ui| {
                            for level in LEVELS {
                                ui.selectable_value(&mut settings.level, *level, level.to_string());
                            }
                        });
                });
                ui.end_row();

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.add(egui::Label::new("Log to file:").wrap());
                });
                ui.add(Checkbox::without_text(&mut settings.log_to_file)).on_hover_text("Allow logs to be added to the file");
                ui.end_row();

                ui.horizontal(|ui| {
                    ui.label("Output Device:");

                    let default_label = "System Default";
                    let current = settings
                        .device_id()
                        .and_then(|d| devices.get(d))
                        .map(|d| d.name())
                        .unwrap_or(default_label);
                    ComboBox::from_id_salt("selector_device")
                        .selected_text(current)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut settings.device_id, None, default_label);
                            ui.separator();
                            for device in devices.iter() {
                                ui.selectable_value(
                                    &mut settings.device_id,
                                    Some(device.id().clone()),
                                    device.name(),
                                );
                            }
                        });
                })
            });
    });
}

fn ui_section_api(ui: &mut Ui, settings: &mut SettingsApp) {
    ui.collapsing("API Configuration", |ui| {
        Grid::new("api_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.add(egui::Label::new("API Key:").extend());
                });
                ui.add(TextEdit::singleline(&mut settings.api_key).password(true));
                ui.end_row();

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.add(egui::Label::new("Languages:").extend());
                });
                ui.vertical(|ui| {
                    let mut to_remove = None;
                    for (i, hint) in settings.language_hints.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}.", i + 1));
                            ui_language_searchable_combo(ui, format!("hint_{}", i), hint);
                            if ui.button("🗑").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }
                    if let Some(i) = to_remove {
                        settings.language_hints.remove(i);
                    }
                    if ui.button("➕ Add").clicked() {
                        settings.language_hints.push(LanguageHint::English);
                    }
                });
                ui.end_row();

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.add(egui::Label::new("Translation:").extend());
                });
                ui.vertical(|ui| {
                    ui.checkbox(&mut settings.enable_translate, "Enable");
                    if settings.enable_translate {
                        ui_language_searchable_combo(
                            ui,
                            "target_lang",
                            &mut settings.target_language,
                        );
                    }
                });
                ui.end_row();
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.add(egui::Label::new("Context:").extend());
                });
                ui.add(TextEdit::multiline(&mut settings.context).desired_rows(2));
                ui.end_row();
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.add(egui::Label::new("Options:").extend());
                });
                ui.checkbox(&mut settings.enable_speakers, "Enable Speakers ID");
                ui.end_row();
            });
    });
}

fn ui_section_position(ui: &mut Ui, ctx: &Context, settings: &mut SettingsApp) {
    ui.collapsing("Position", |ui| {
        Grid::new("pos_grid").spacing([10.0, 10.0]).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("Offset:").extend());
                ui.label("X:");
                ui.add(DragValue::new(&mut settings.offset.0).speed(1.0));
                ui.label("Y:");
                ui.add(DragValue::new(&mut settings.offset.1).speed(1.0));
            });
            ui.end_row();

            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.add(egui::Label::new("Snap to:").extend());
            });
            ui.end_row();
            ui.vertical(|ui| {
                Grid::new("snap_buttons")
                    .spacing([5.0, 5.0])
                    .show(ui, |ui| {
                        let mut btn =
                            |ui: &mut Ui,
                             text: &str,
                             anchor_val: usize,
                             default_offset: (f32, f32)| {
                                let is_selected = settings.anchor == anchor_val;
                                let button = Button::new(RichText::new(text).size(16.0))
                                    .min_size(vec2(30.0, 30.0));

                                let response = if is_selected {
                                    ui.add(button.fill(ctx.style().visuals.selection.bg_fill))
                                } else {
                                    ui.add(button)
                                };
                                if response.clicked() {
                                    settings.anchor = anchor_val;
                                    settings.offset = default_offset;
                                }
                            };

                        let pad = 30.0;
                        btn(ui, "↖", 0, (pad, pad));
                        btn(ui, "⬆", 1, (0.0, pad));
                        btn(ui, "↗", 2, (-pad, pad));
                        ui.end_row();

                        btn(ui, "←", 3, (pad, 0.0));
                        btn(ui, "X", 4, (0.0, 0.0));
                        btn(ui, "→", 5, (-pad, 0.0));
                        ui.end_row();

                        btn(ui, "↙", 6, (pad, -pad));
                        btn(ui, "⬇", 7, (0.0, -pad));
                        btn(ui, "↘", 8, (-pad, -pad));
                        ui.end_row();
                    });
            });
            ui.end_row();
        });
    });
}

fn ui_language_searchable_combo(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    selected: &mut LanguageHint,
) {
    let id = ui.make_persistent_id(id_salt);
    let mut search_term = ui.data_mut(|d| d.get_temp::<String>(id).unwrap_or_default());

    ComboBox::from_id_salt(id)
        .selected_text(selected.to_string())
        .height(250.)
        .show_ui(ui, |ui| {
            ui.set_min_width(180.0);
            ui.set_min_height(250.0);
            let text_edit_response = ui.add(
                TextEdit::singleline(&mut search_term)
                    .hint_text("🔍 Search...")
                    .desired_width(f32::INFINITY),
            );

            if !text_edit_response.has_focus() {
                text_edit_response.request_focus();
            }

            ui.separator();
            let query = search_term.to_lowercase();
            for lang in LanguageHint::all() {
                let lang_name = lang.to_string();
                if (query.is_empty() || lang_name.to_lowercase().contains(&query))
                    && ui.selectable_value(selected, *lang, lang_name).clicked()
                {
                    search_term.clear();
                }
            }
        });

    ui.data_mut(|d| d.insert_temp(id, search_term));
}

fn ui_section_appearance(ui: &mut Ui, settings: &mut SettingsApp) {
    ui.collapsing("Appearance", |ui| {
        Grid::new("appearance_grid")
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                ui.label("Max Blocks:");
                ui.add(Slider::new(&mut settings.max_blocks, 1..=10));
                ui.end_row();

                ui.label("Font Size:");
                ui.add(Slider::new(&mut settings.font_size, 10..=80));
                ui.end_row();

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.add(egui::Label::new("Style:").extend());
                });
                Grid::new("font_style").spacing([10.0, 8.0]).show(ui, |ui| {
                    ui.checkbox(&mut settings.enable_background, "Background");
                    ui.end_row();
                    ui.checkbox(&mut settings.enable_high_priority, "Always On Top");
                    ui.end_row();
                });
                ui.end_row();
            });

        ui.separator();

        Grid::new("colors_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Text Color:");
                ui.horizontal(|ui| {
                    if ui
                        .button("🔄 Reset to default")
                        .on_hover_text("Reset to Yellow")
                        .clicked()
                    {
                        settings.text_color = (255, 255, 0);
                    }
                });
                ui.end_row();

                ui.label("Red:");
                ui.add(Slider::new(&mut settings.text_color.0, 0..=255));
                ui.end_row();

                ui.label("Green:");
                ui.add(Slider::new(&mut settings.text_color.1, 0..=255));
                ui.end_row();

                ui.label("Blue:");
                ui.add(Slider::new(&mut settings.text_color.2, 0..=255));
                ui.end_row();
            });

        let p_col = Color32::from_rgb(
            settings.text_color.0,
            settings.text_color.1,
            settings.text_color.2,
        );
        egui::Frame::new()
            .fill(p_col)
            .corner_radius(5.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                let text_col = if (settings.text_color.0 as u16
                    + settings.text_color.1 as u16
                    + settings.text_color.2 as u16)
                    > 380
                {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                };
                ui.label(
                    RichText::new(format!("Preview ({:.0}px)", settings.font_size))
                        .color(text_col)
                        .size(settings.font_size()),
                );
            });
    });
}
