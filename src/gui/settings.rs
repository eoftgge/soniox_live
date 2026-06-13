use crate::gui::state::{PendingState, StateManager};
use crate::settings::SettingsApp;
use crate::transcription::device::MappableAvailableDevices;
use crate::stt::languages::LanguageHint;
use crate::logger::LEVELS;
use eframe::egui::{self, Button, Checkbox, ComboBox, DragValue, Grid, RichText, ScrollArea, Slider, TextEdit, Ui, vec2, Color32};
use egui_notify::Toasts;
use std::time::Duration;
use std::path::PathBuf;
use crate::stt::adapters::types::{ProviderType, SonioxSettings, WhisperSettings};

pub fn show_settings_window(
    ui: &mut Ui,
    settings: &mut SettingsApp,
    manager: &mut StateManager,
    toasts: &mut Toasts,
    devices: &mut MappableAvailableDevices,
) {
    ui_bottom_panel(ui, settings, manager, toasts);

    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ui.ctx().global_style()).inner_margin(15.0))
        .show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 12.0);
            ui.heading("Settings");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                ui_section_app(ui, settings, devices);
                ui_section_provider(ui, settings);
                ui_section_position(ui, settings);
                ui_section_appearance(ui, settings);
                ui.allocate_space(vec2(0.0, 60.0));
            });
        });
}

fn ui_bottom_panel(
    ui: &mut Ui,
    settings: &mut SettingsApp,
    manager: &mut StateManager,
    toasts: &mut Toasts,
) {
    egui::Panel::bottom("settings_bottom_panel")
        .resizable(false)
        .min_size(60.0)
        .show_inside(ui, |ui| {
            ui.add_space(15.0);
            ui.columns(2, |cols| {
                cols[0].vertical_centered_justified(|ui| {
                    if ui
                        .add(Button::new("💾 Save").min_size(vec2(0.0, 40.0)))
                        .clicked()
                    {
                        match settings.save() {
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
                        match settings.provider_type {
                            ProviderType::Soniox => {
                                if settings.soniox.api_key.trim().is_empty() {
                                    toasts.warning("No API key provided for Soniox!").closable(false);
                                    return;
                                }
                            }
                            ProviderType::Whisper => {
                                if settings.whisper.path.as_os_str().is_empty() {
                                    toasts.warning("No model path provided for Whisper!").closable(false);
                                    return;
                                }
                            }
                        }
                        manager.switch(PendingState::Overlay);
                        toasts.info("Starting subtitles overlay...").closable(false);
                    }
                });
            });
            ui.add_space(10.0);
        });
}

fn ui_section_provider(ui: &mut Ui, settings: &mut SettingsApp) {
    ui.collapsing("Speech Engine (STT)", |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut settings.provider_type, ProviderType::Soniox, "☁ Soniox (Cloud)");
            ui.selectable_value(&mut settings.provider_type, ProviderType::Whisper, "💻 Whisper (Offline)");
        });

        ui.separator();

        match settings.provider_type {
            ProviderType::Soniox => ui_soniox_settings(ui, &mut settings.soniox),
            ProviderType::Whisper => ui_whisper_settings(ui, &mut settings.whisper),
        }
    });
}

fn ui_soniox_settings(ui: &mut Ui, soniox: &mut SonioxSettings) {
    Grid::new("soniox_grid")
        .num_columns(2)
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            ui.add(egui::Label::new("API Key:").extend());
            ui.add(TextEdit::singleline(&mut soniox.api_key).password(true));
            ui.end_row();

            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.label("Languages:");
            });
            ui.vertical(|ui| {
                let mut to_remove = None;
                for (i, hint) in soniox.language_hints.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", i + 1));
                        ui_language_searchable_combo(ui, format!("hint_{}", i), hint);
                        if ui.button("🗑").clicked() {
                            to_remove = Some(i);
                        }
                    });
                }
                if let Some(i) = to_remove {
                    soniox.language_hints.remove(i);
                }
                if ui.button("➕ Add").clicked() {
                    soniox.language_hints.push(LanguageHint::English);
                }
            });
            ui.end_row();

            ui.add(egui::Label::new("Translation:").extend());
            ui.checkbox(&mut soniox.enable_translate, "Enable");

            if soniox.enable_translate {
                ui.end_row();
                ui.add(egui::Label::new("Target language:").extend());
                ui_language_searchable_combo(ui, "target_lang", &mut soniox.target_language);
            }
            ui.end_row();

            ui.add(egui::Label::new("Context:").extend());
            ui.add(TextEdit::multiline(&mut soniox.context).desired_rows(2));
            ui.end_row();

            ui.add(egui::Label::new("Options:").extend());
            ui.checkbox(&mut soniox.enable_speakers, "Enable Speakers ID");
            ui.end_row();
        });
}

fn ui_whisper_settings(ui: &mut Ui, whisper: &mut WhisperSettings) {
    Grid::new("whisper_grid")
        .num_columns(2)
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            ui.add(egui::Label::new("Model File:").extend());
            ui.horizontal(|ui| {
                let mut path_str = whisper.path.display().to_string();
                if ui.add(TextEdit::singleline(&mut path_str).desired_width(200.0)).changed() {
                    whisper.path = PathBuf::from(path_str);
                }

                if ui.button("📂 Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Whisper Models", &["bin"])
                        .pick_file()
                    {
                        whisper.path = path;
                    }
                }
            });
            ui.end_row();

            ui.label("");
            ui.label(RichText::new("Download ggml-*.bin models from HuggingFace").color(Color32::GRAY).small());
            ui.end_row();
        });
}

fn ui_section_app(ui: &mut Ui, settings: &mut SettingsApp, devices: &mut MappableAvailableDevices) {
    ui.collapsing("Configuration App", |ui| {
        Grid::new("app_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                ui.label("Hangover Chunks:");
                ui.add(Slider::new(&mut settings.audio.hangover_chunks, 0..=50));
                ui.end_row();

                ui.label("Threshold:");
                ui.add(Slider::new(&mut settings.audio.vad_threshold, 0..=2000).logarithmic(true));
                ui.end_row();

                ui.label("Log Level:");
                ComboBox::from_id_salt("log_level")
                    .selected_text(settings.level.to_string())
                    .width(40.0)
                    .show_ui(ui, |ui| {
                        for level in LEVELS {
                            ui.selectable_value(&mut settings.level, *level, level.to_string());
                        }
                    });
                ui.end_row();

                ui.label("Log to file");
                ui.add(Checkbox::without_text(&mut settings.log_to_file))
                    .on_hover_text("Allow logs to be added to the file");
                ui.end_row();

                ui.label("Output Device:");

                let default_label = "System Default";
                let current = settings
                    .audio
                    .device_id
                    .clone()
                    .and_then(|d| devices.get(&d))
                    .map(|d| d.name())
                    .unwrap_or(default_label);
                ComboBox::from_id_salt("selector_device")
                    .selected_text(current)
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut settings.audio.device_id, None, default_label);
                        ui.separator();
                        for device in devices.iter() {
                            ui.selectable_value(
                                &mut settings.audio.device_id,
                                Some(device.id().clone()),
                                device.name(),
                            );
                        }
                    });
            });
    });
}

fn ui_section_position(ui: &mut Ui, settings: &mut SettingsApp) {
    ui.collapsing("Position", |ui| {
        Grid::new("pos_grid").spacing([10.0, 10.0]).show(ui, |ui| {
            ui.add(egui::Label::new("Offset:").extend());
            ui.horizontal(|ui| {
                ui.add(
                    DragValue::new(&mut settings.gui.offset.0)
                        .speed(1.0)
                        .prefix("X: "),
                );
                ui.add(
                    DragValue::new(&mut settings.gui.offset.1)
                        .speed(1.0)
                        .prefix("Y: "),
                );
            });
            ui.end_row();

            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.add(egui::Label::new("Snap to:").extend());
            });
            ui.vertical(|ui| {
                Grid::new("snap_buttons")
                    .spacing([5.0, 5.0])
                    .show(ui, |ui| {
                        let mut btn =
                            |ui: &mut Ui,
                             text: &str,
                             anchor_val: usize,
                             default_offset: (f32, f32)| {
                                let is_selected = settings.gui.anchor == anchor_val;
                                let button = Button::new(RichText::new(text).size(16.0))
                                    .min_size(vec2(30.0, 30.0));

                                let response =
                                    if is_selected {
                                        ui.add(button.fill(
                                            ui.ctx().global_style().visuals.selection.bg_fill,
                                        ))
                                    } else {
                                        ui.add(button)
                                    };
                                if response.clicked() {
                                    settings.gui.anchor = anchor_val;
                                    settings.gui.offset = default_offset;
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
                ui.add(Slider::new(&mut settings.audio.max_blocks, 1..=10));
                ui.end_row();

                ui.label("Font Size:");
                ui.add(Slider::new(&mut settings.gui.font_size, 10..=80));
                ui.end_row();

                ui.label("Always On Top:");
                ui.add(Checkbox::without_text(&mut settings.gui.enable_high_priority));
                ui.end_row();
            });

        ui.separator();

        Grid::new("colors_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Background Color:");
                ui.horizontal(|ui| {
                    let color = &mut settings.gui.background_color;
                    if ui.color_edit_button_rgba_unmultiplied(color).changed() {
                        settings.gui.background_color = [color[0], color[1], color[2], color[3]];
                    }
                    if ui.button("Clear").clicked() {
                        settings.gui.background_color = [0.; 4];
                    }
                });
                ui.end_row();

                ui.label("Text Color:");
                ui.horizontal(|ui| {
                    let color = &mut settings.gui.text_color;
                    if ui.color_edit_button_rgb(color).changed() {
                        settings.gui.text_color = [color[0], color[1], color[2]];
                    }
                    if ui
                        .button("Clear")
                        .on_hover_text("Reset to Yellow")
                        .clicked()
                    {
                        settings.gui.text_color = [255., 255., 0.]; // yellow
                    }
                });
                ui.end_row();
            });

        egui::Frame::new()
            .fill(settings.gui.background_color())
            .corner_radius(5.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("Preview ({:.0}px)", settings.gui.font_size))
                        .color(settings.gui.text_color())
                        .size(settings.gui.font_size as f32),
                );
            });
    });
}
