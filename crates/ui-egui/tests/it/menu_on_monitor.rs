//! A dropdown list in a window moved down its monitor stops at the foot of the monitor, not of the
//! window, and scrolls there (`vendor/craft-ui`, `combo::visible_area`).

use egui_kittest::{Harness, kittest::Queryable as _};

/// What the list asks for, like the Scan and OCR "Document language" list.
const ASKED: f32 = 560.0;
/// A window this tall, a list at its top.
const WINDOW: f32 = 800.0;
const MONITOR: egui::Vec2 = egui::vec2(1920.0, 1080.0);
/// The scroll area's clip may reach a little past the menu's frame.
const SLACK: f32 = 8.0;

/// The list open in a window whose top is `top` points down the monitor; the state is the part of
/// the list its rows can be seen in.
fn open_at(top: f32) -> Harness<'static, egui::Rect> {
    let mut h = Harness::builder().with_size(egui::vec2(400.0, WINDOW)).build_ui_state(
        |ui, seen: &mut egui::Rect| {
            craft_ui::ComboBox::new("language", "Language").selected_text("English").height(ASKED).show_ui(ui, |ui| {
                *seen = ui.clip_rect();
                for n in 0..100 {
                    let _ = ui.selectable_label(false, format!("Language {n}"));
                }
            });
        },
        egui::Rect::NOTHING,
    );
    let info = h.input_mut().viewports.entry(egui::ViewportId::ROOT).or_default();
    info.inner_rect = Some(egui::Rect::from_min_size(egui::pos2(100.0, top), egui::vec2(400.0, WINDOW)));
    info.monitor_size = Some(MONITOR);
    h.run();
    h.get_by_role_and_label(egui::Role::ComboBox, "Language").click();
    h.run();
    h
}

#[test]
fn a_list_in_a_window_moved_down_stops_above_the_foot_of_the_monitor() {
    let top = 580.0;
    let h = open_at(top);
    let seen = *h.state();
    // In window coordinates the monitor ends at 500: the window and the list asked for go past it.
    let monitor_foot = MONITOR.y - top;
    assert!(seen.bottom() <= monitor_foot + SLACK - 48.0, "the list ends on the monitor, above its foot: {seen:?}");
    assert!(seen.height() > 100.0, "and there is still a list to scroll: {seen:?}");
}

#[test]
fn a_list_in_a_window_on_the_monitor_keeps_the_height_it_asked_for() {
    let h = open_at(100.0);
    let seen = *h.state();
    assert!(seen.height() > ASKED - 2.0 * SLACK, "nothing to cut: {seen:?}");
}
