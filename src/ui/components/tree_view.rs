use crate::file_info::{FileEntry, SortCriteria, SortDirection, SortState};
use crate::ui::colors;
use crate::utils::{format_date, format_size};
use eframe::egui;

const CHECKBOX_WIDTH: f32 = 30.0;
const SIZE_WIDTH: f32 = 90.0;
const DATE_WIDTH: f32 = 140.0;
const COLUMN_SPACING: f32 = 12.0;
const ROW_HEIGHT: f32 = 28.0;

/// Render tree view cho danh sách file, trả về tiêu chí cần sort nếu người dùng click header
pub fn render_tree_view(
    ui: &mut egui::Ui,
    entries: &mut Vec<FileEntry>,
    sort_state: Option<SortState>,
) -> Option<SortCriteria> {
    let scroll_bar_width = ui.spacing().scroll.bar_width;
    let total_width = ui.available_width();
    let content_width = total_width - scroll_bar_width - 4.0;

    // 1. Render Header
    let sort_click = render_header(ui, content_width, entries, sort_state);
    ui.separator();

    // 2. Scroll area cho danh sách file
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for entry in entries.iter_mut() {
                render_entry(ui, entry, 0, content_width);
            }
        });

    sort_click
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

/// Render header cột
fn render_header(
    ui: &mut egui::Ui,
    width: f32,
    entries: &mut Vec<FileEntry>,
    sort_state: Option<SortState>,
) -> Option<SortCriteria> {
    let name_width = calculate_name_width(width);
    let x_offsets = calculate_x_offsets(width);
    let mut clicked_criteria = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;

        // Cột Tên (bao gồm checkbox Select All)
        ui.allocate_ui(
            egui::vec2(CHECKBOX_WIDTH + COLUMN_SPACING + name_width, 20.0),
            |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(CHECKBOX_WIDTH, 20.0), |ui| {
                        ui.centered_and_justified(|ui| {
                            let mut all_selected =
                                !entries.is_empty() && entries.iter().all(|e| e.selected);
                            if ui.checkbox(&mut all_selected, "").clicked() {
                                for entry in entries {
                                    entry.set_selected_recursive(all_selected);
                                }
                            }
                        });
                    });

                    ui.add_space(COLUMN_SPACING);

                    let label = format!(
                        "Tên file / Thư mục{}",
                        get_sort_icon(SortCriteria::Name, sort_state)
                    );
                    if ui
                        .selectable_label(
                            false,
                            egui::RichText::new(label).strong().color(colors::ACCENT),
                        )
                        .clicked()
                    {
                        clicked_criteria = Some(SortCriteria::Name);
                    }
                });
            },
        );

        // Cột Ngày tạo
        ui.add_space(x_offsets[2] - (ui.cursor().min.x - ui.max_rect().left()));
        ui.allocate_ui(egui::vec2(DATE_WIDTH, 20.0), |ui| {
            let label = format!(
                "Ngày tạo{}",
                get_sort_icon(SortCriteria::Created, sort_state)
            );
            if ui
                .selectable_label(
                    false,
                    egui::RichText::new(label).strong().color(colors::ACCENT),
                )
                .clicked()
            {
                clicked_criteria = Some(SortCriteria::Created);
            }
        });

        // Cột Ngày sửa
        ui.add_space(x_offsets[3] - (ui.cursor().min.x - ui.max_rect().left()));
        ui.allocate_ui(egui::vec2(DATE_WIDTH, 20.0), |ui| {
            let label = format!(
                "Ngày sửa{}",
                get_sort_icon(SortCriteria::Modified, sort_state)
            );
            if ui
                .selectable_label(
                    false,
                    egui::RichText::new(label).strong().color(colors::ACCENT),
                )
                .clicked()
            {
                clicked_criteria = Some(SortCriteria::Modified);
            }
        });

        // Cột Dung lượng
        ui.add_space(x_offsets[4] - (ui.cursor().min.x - ui.max_rect().left()));
        ui.allocate_ui(egui::vec2(SIZE_WIDTH, 20.0), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = format!(
                    "Dung lượng{}",
                    get_sort_icon(SortCriteria::Size, sort_state)
                );
                if ui
                    .selectable_label(
                        false,
                        egui::RichText::new(label).strong().color(colors::ACCENT),
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

fn calculate_name_width(total_width: f32) -> f32 {
    let meta_parts = SIZE_WIDTH + (DATE_WIDTH * 2.0) + (COLUMN_SPACING * 4.0);
    (total_width - CHECKBOX_WIDTH - meta_parts).max(100.0)
}

fn calculate_x_offsets(total_width: f32) -> [f32; 5] {
    let name_width = calculate_name_width(total_width);
    let x0 = 0.0;
    let x1 = x0 + CHECKBOX_WIDTH + COLUMN_SPACING;
    let x2 = x1 + name_width + COLUMN_SPACING;
    let x3 = x2 + DATE_WIDTH + COLUMN_SPACING;
    let x4 = x3 + DATE_WIDTH + COLUMN_SPACING;
    [x0, x1, x2, x3, x4]
}

/// Render một entry (file hoặc folder) trong tree
fn render_entry(ui: &mut egui::Ui, entry: &mut FileEntry, depth: usize, width: f32) {
    let x_offsets = calculate_x_offsets(width);
    let name_width = calculate_name_width(width);
    let indent = depth as f32 * 18.0;

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
        ui.allocate_exact_size(egui::vec2(width, ROW_HEIGHT), egui::Sense::click());

    let mut new_selected = entry.selected;
    let mut toggle_expand = false;

    // Nền hover
    if ui.rect_contains_pointer(rect.shrink2(egui::vec2(0.0, 0.5))) {
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_white_alpha(15));
    }

    // Double click
    if response.double_clicked() {
        if is_dir {
            toggle_expand = true;
        } else {
            let _ = open::that(&entry.path);
        }
    }

    // Đường kẻ phân cách
    ui.painter().hline(
        rect.left()..=rect.right(),
        rect.bottom(),
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    let start_x = rect.left();
    let row_y = rect.top() + (ROW_HEIGHT - 20.0) / 2.0;

    // 1. Cột Checkbox
    let cb_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[0], row_y),
        egui::vec2(CHECKBOX_WIDTH, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(cb_rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.checkbox(&mut new_selected, "");
        });
    });

    // 2. Cột Tên
    let name_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[1], row_y),
        egui::vec2(name_width, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(name_rect), |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
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
                            .color(colors::FILE_SELECTED),
                    )
                    .selectable(false),
                );
            } else {
                ui.add_space(18.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!("{} {}", icon, name))
                            .color(colors::FILE_NORMAL),
                    )
                    .selectable(false),
                );
            }
        });
    });

    // 3. Cột Ngày tạo
    let c_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[2], row_y),
        egui::vec2(DATE_WIDTH, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(c_rect), |ui| {
        ui.add_space(2.0);
        ui.label(egui::RichText::new(format_date(created)).color(colors::TEXT_SECONDARY));
    });

    // 4. Cột Ngày sửa
    let m_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[3], row_y),
        egui::vec2(DATE_WIDTH, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(m_rect), |ui| {
        ui.add_space(2.0);
        ui.label(egui::RichText::new(format_date(modified)).color(colors::TEXT_SECONDARY));
    });

    // 5. Cột Dung lượng
    let s_rect = egui::Rect::from_min_size(
        egui::pos2(start_x + x_offsets[4], row_y),
        egui::vec2(SIZE_WIDTH, 20.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(s_rect), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(format_size(size)).color(colors::TEXT_SECONDARY));
        });
    });

    // Lưu state
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

    // Render con
    if is_dir && entry.expanded {
        for child in &mut entry.children {
            render_entry(ui, child, depth + 1, width);
        }
    }
}
