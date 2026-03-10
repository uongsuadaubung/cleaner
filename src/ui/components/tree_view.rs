use crate::file_info::{FileEntry, SortCriteria, SortDirection, SortState};
use crate::lang::Lang;
use crate::ui::colors;
use crate::ui::theme;
use crate::utils::{format_date, format_size};
use eframe::egui;
use std::path::PathBuf;

/// Hành động trả về từ tree view
pub enum TreeViewAction {
    /// Người dùng click header column để sort
    Sort(SortCriteria),
    /// Người dùng double-click vào thư mục → điều hướng tới đó
    NavigateTo(PathBuf),
}

/// Render tree view cho danh sách file
pub fn render_tree_view(
    ui: &mut egui::Ui,
    entries: &mut Vec<FileEntry>,
    sort_state: Option<SortState>,
    lang: &Lang,
    scroll_to_top: &mut bool,
) -> Option<TreeViewAction> {
    let t = &theme::DEFAULT;
    let scroll_bar_width = ui.spacing().scroll.bar_width;
    let total_width = ui.available_width();
    let content_width = total_width - scroll_bar_width - 4.0;

    let sort_click = render_header(ui, content_width, entries, sort_state, lang, t);
    ui.separator();

    let mut navigate_to: Option<PathBuf> = None;

    let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([false, false]);

    if *scroll_to_top {
        scroll_area = scroll_area.scroll_offset(egui::Vec2::ZERO);
        *scroll_to_top = false;
    }

    scroll_area.show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for entry in entries.iter_mut() {
            if let Some(path) = render_entry(ui, entry, 0, content_width, t) {
                navigate_to = Some(path);
            }
        }
    });

    if let Some(path) = navigate_to {
        return Some(TreeViewAction::NavigateTo(path));
    }

    sort_click.map(TreeViewAction::Sort)
}

fn get_sort_icon(criteria: SortCriteria, sort_state: Option<SortState>) -> String {
    if let Some(state) = sort_state
        && state.criteria == criteria
    {
        return match state.direction {
            SortDirection::Asc => " ⏶".to_string(),
            SortDirection::Desc => " ⏷".to_string(),
        };
    }
    "".to_string()
}

fn render_header(
    ui: &mut egui::Ui,
    width: f32,
    entries: &mut Vec<FileEntry>,
    sort_state: Option<SortState>,
    lang: &Lang,
    t: &theme::Theme,
) -> Option<SortCriteria> {
    let name_width = calculate_name_width(width, t);
    let x_offsets = calculate_x_offsets(width, t);
    let mut clicked_criteria = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        ui.allocate_ui(
            egui::vec2(
                t.tree_checkbox_width + t.tree_col_spacing + name_width,
                20.0,
            ),
            |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(t.tree_checkbox_width, 20.0), |ui| {
                        ui.centered_and_justified(|ui| {
                            let mut all_selected =
                                !entries.is_empty() && entries.iter().all(|e| e.selected);
                            if ui.checkbox(&mut all_selected, "").clicked() {
                                for entry in entries.iter_mut() {
                                    entry.set_selected_recursive(all_selected);
                                }
                            }
                        });
                    });

                    ui.add_space(t.tree_col_spacing);

                    let label = format!(
                        "{}{}",
                        lang.col_name,
                        get_sort_icon(SortCriteria::Name, sort_state)
                    );
                    if ui
                        .selectable_label(
                            false,
                            egui::RichText::new(label)
                                .strong()
                                .color(colors::accent(ui.visuals().dark_mode)),
                        )
                        .clicked()
                    {
                        clicked_criteria = Some(SortCriteria::Name);
                    }
                });
            },
        );

        ui.add_space(x_offsets[2] - (ui.cursor().min.x - ui.max_rect().left()));
        ui.allocate_ui(egui::vec2(t.tree_date_col, 20.0), |ui| {
            let label = format!(
                "{}{}",
                lang.col_created,
                get_sort_icon(SortCriteria::Created, sort_state)
            );
            if ui
                .selectable_label(
                    false,
                    egui::RichText::new(label)
                        .strong()
                        .color(colors::accent(ui.visuals().dark_mode)),
                )
                .clicked()
            {
                clicked_criteria = Some(SortCriteria::Created);
            }
        });

        ui.add_space(x_offsets[3] - (ui.cursor().min.x - ui.max_rect().left()));
        ui.allocate_ui(egui::vec2(t.tree_date_col, 20.0), |ui| {
            let label = format!(
                "{}{}",
                lang.col_modified,
                get_sort_icon(SortCriteria::Modified, sort_state)
            );
            if ui
                .selectable_label(
                    false,
                    egui::RichText::new(label)
                        .strong()
                        .color(colors::accent(ui.visuals().dark_mode)),
                )
                .clicked()
            {
                clicked_criteria = Some(SortCriteria::Modified);
            }
        });

        ui.add_space(x_offsets[4] - (ui.cursor().min.x - ui.max_rect().left()));
        ui.allocate_ui(egui::vec2(t.tree_size_col, 20.0), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = format!(
                    "{}{}",
                    lang.col_size,
                    get_sort_icon(SortCriteria::Size, sort_state)
                );
                if ui
                    .selectable_label(
                        false,
                        egui::RichText::new(label)
                            .strong()
                            .color(colors::accent(ui.visuals().dark_mode)),
                    )
                    .clicked()
                {
                    clicked_criteria = Some(SortCriteria::Size);
                }
            });
        });
    });

    clicked_criteria
}

fn calculate_name_width(total_width: f32, t: &theme::Theme) -> f32 {
    let meta_parts = t.tree_size_col + (t.tree_date_col * 2.0) + (t.tree_col_spacing * 4.0);
    (total_width - t.tree_checkbox_width - meta_parts).max(100.0)
}

fn calculate_x_offsets(total_width: f32, t: &theme::Theme) -> [f32; 5] {
    let name_width = calculate_name_width(total_width, t);
    let x0 = 0.0;
    let x1 = x0 + t.tree_checkbox_width + t.tree_col_spacing;
    let x2 = x1 + name_width + t.tree_col_spacing;
    let x3 = x2 + t.tree_date_col + t.tree_col_spacing;
    let x4 = x3 + t.tree_date_col + t.tree_col_spacing;
    [x0, x1, x2, x3, x4]
}

/// Trả về `Some(path)` nếu người dùng double-click vào folder để điều hướng
fn render_entry(
    ui: &mut egui::Ui,
    entry: &mut FileEntry,
    depth: usize,
    width: f32,
    t: &theme::Theme,
) -> Option<PathBuf> {
    let x_offsets = calculate_x_offsets(width, t);
    let name_width = calculate_name_width(width, t);
    let indent = depth as f32 * t.tree_indent_per_level;

    let is_dir = entry.is_dir;
    let icon = entry.category.icon();
    let name = entry.name.clone();
    let created = entry.created;
    let modified = entry.modified;
    let size = if is_dir {
        entry.total_size()
    } else {
        entry.size
    };

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, t.tree_row_height), egui::Sense::click());

    let mut new_selected = entry.selected;
    let mut toggle_expand = false;
    let mut navigate_to: Option<PathBuf> = None;

    if ui.rect_contains_pointer(rect.shrink2(egui::vec2(0.0, 0.5))) {
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_white_alpha(15));
    }

    if response.double_clicked() {
        if is_dir {
            // Double-click folder → điều hướng tới thư mục đó
            navigate_to = Some(entry.path.clone());
        } else {
            let _ = open::that(&entry.path);
        }
    }

    ui.painter().hline(
        rect.left()..=rect.right(),
        rect.bottom(),
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    let start_x = rect.left();
    let row_y = rect.top() + (t.tree_row_height - 20.0) / 2.0;

    // Checkbox column
    let cb_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[0], row_y),
        egui::vec2(t.tree_checkbox_width, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(cb_rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.checkbox(&mut new_selected, "");
        });
    });

    // Name column
    let name_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[1], row_y),
        egui::vec2(name_width, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(name_rect), |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = t.space_sm;
            ui.add_space(indent);

            if ui.rect_contains_pointer(name_rect) {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            if is_dir {
                let arrow = if entry.expanded { "⏷" } else { "⏵" };
                if ui.selectable_label(false, arrow).clicked() {
                    toggle_expand = true;
                }
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!("{} {}", icon, name))
                            .strong()
                            .color(colors::file_selected(ui.visuals().dark_mode)),
                    )
                    .selectable(false),
                );
            } else {
                ui.add_space(18.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!("{} {}", icon, name))
                            .color(colors::file_normal(ui.visuals().dark_mode)),
                    )
                    .selectable(false),
                );
            }
        });
    });

    // Created date column
    let c_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[2], row_y),
        egui::vec2(t.tree_date_col, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(c_rect), |ui| {
        ui.add_space(t.tree_col_spacing / 6.0);
        ui.label(
            egui::RichText::new(format_date(created))
                .color(colors::text_secondary(ui.visuals().dark_mode)),
        );
    });

    // Modified date column
    let m_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[3], row_y),
        egui::vec2(t.tree_date_col, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(m_rect), |ui| {
        ui.add_space(t.tree_col_spacing / 6.0);
        ui.label(
            egui::RichText::new(format_date(modified))
                .color(colors::text_secondary(ui.visuals().dark_mode)),
        );
    });

    // Size column
    let s_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[4], row_y),
        egui::vec2(t.tree_size_col, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(s_rect), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format_size(size))
                    .color(colors::text_secondary(ui.visuals().dark_mode)),
            );
        });
    });

    if new_selected != entry.selected {
        if is_dir {
            entry.set_selected_recursive(new_selected);
        } else {
            entry.selected = new_selected;
        }
    }
    if toggle_expand {
        entry.expanded = !entry.expanded;
    }

    // Nếu đã navigate, không cần render children (sẽ load page mới)
    if navigate_to.is_some() {
        return navigate_to;
    }

    if is_dir && entry.expanded {
        for child in &mut entry.children {
            if let Some(path) = render_entry(ui, child, depth + 1, width, t) {
                return Some(path);
            }
        }
    }

    navigate_to
}
