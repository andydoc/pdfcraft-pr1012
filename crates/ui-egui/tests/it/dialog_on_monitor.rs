//! The Scan and OCR dialog in a window moved down its monitor: it sits in the part of the window the
//! monitor shows, what it asks scrolls there, and its buttons stay on the screen
//! (craft-libs `craft_window::desktop::visible_part`, used by the dialog overlays).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::Session;
use pdfcraft_ui_egui::PdfCraftApp;

use crate::support::AppHarness as _;

/// The default window on a 1920 x 1080 monitor at 125%.
const WINDOW: egui::Vec2 = egui::vec2(1368.0, 740.0);
const MONITOR: egui::Vec2 = egui::vec2(1536.0, 864.0);
/// Tall enough for all the dialog asks, with or without the note that text recognition isn't
/// installed: tests run with or without its model files. Title to buttons took 1169 points with the
/// models and 1207 without when this was measured, so it scrolls on a 1080-line monitor.
const TALL_WINDOW: egui::Vec2 = egui::vec2(1368.0, 1300.0);
const TALL_MONITOR: egui::Vec2 = egui::vec2(1536.0, 1400.0);
/// What a window that doesn't fill its monitor leaves free at the foot (`craft_window::desktop::TASKBAR`).
const FOOT: f32 = 48.0;

fn pdf() -> Vec<u8> {
    Session::new().create_from_text("t", "A page").unwrap().to_vec()
}

/// The dialog open in a `window` whose top is `top` points down the `monitor`.
fn dialog_in(window: egui::Vec2, monitor: egui::Vec2, top: f32) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(window).build_eframe(|cc| {
        crate::support::install_runtime(&cc.egui_ctx);
        let mut app = crate::control::new_app();
        app.set_option(&cc.egui_ctx, "language", "en").unwrap();
        app.open_bytes(&cc.egui_ctx, "a.pdf", None, pdf()).unwrap();
        app
    });
    let info = h.input_mut().viewports.entry(egui::ViewportId::ROOT).or_default();
    info.inner_rect = Some(egui::Rect::from_min_size(egui::pos2(100.0, top), window));
    info.monitor_size = Some(monitor);
    h.run_steps(4);
    assert!(h.execute("ocr.recognize"));
    h.run_steps(6);
    h
}

fn dialog_at(top: f32) -> Harness<'static, PdfCraftApp> {
    dialog_in(WINDOW, MONITOR, top)
}

/// In window coordinates, where the monitor stops a window whose top is `top`.
fn monitor_foot(top: f32) -> f32 {
    MONITOR.y - FOOT - top
}

fn buttons_bottom(h: &Harness<'static, PdfCraftApp>) -> f32 {
    let recognize = h.get_by_label("Recognize text").rect();
    let cancel = h.get_by_label("Cancel").rect();
    assert_eq!(recognize.bottom(), cancel.bottom(), "the buttons share a row");
    recognize.bottom()
}

/// Whether what the dialog asks runs past its foot, so it scrolls there. The dialog's scroll bar is
/// drawn, and seen by accessibility, only while the pointer is over it: the last thing it asks is the
/// probe instead.
fn has_scroll_bar(h: &Harness<'static, PdfCraftApp>) -> bool {
    let dialog = h.get_by_role_and_label(egui::Role::Dialog, "Scan and text recognition").rect();
    h.get_by_label("Add searchable text").rect().bottom() > dialog.bottom()
}

#[test]
fn the_buttons_stay_on_the_monitor_in_a_window_moved_down_it() {
    for top in [240.0, 480.0] {
        let h = dialog_at(top);
        let foot = monitor_foot(top);
        assert!(buttons_bottom(&h) <= foot, "top {top}: the buttons end at {}, the monitor at {foot}", buttons_bottom(&h));
        assert!(
            h.get_by_role_and_label(egui::Role::Dialog, "Scan and text recognition").rect().top() >= 0.0,
            "top {top}: the title stays in the window"
        );
        assert!(has_scroll_bar(&h), "top {top}: what doesn't fit scrolls");
        // Still there to scroll to, and so is the list's field.
        h.get_by_label("Add searchable text");
        h.get_by_label("Document language");
    }
}

#[test]
fn the_buttons_still_answer_in_a_window_moved_down_the_monitor() {
    let mut h = dialog_at(240.0);
    h.get_by_label("Cancel").click();
    h.run_steps(4);
    assert!(h.query_by_label("Scan and text recognition").is_none(), "Cancel closes the dialog");
}

#[test]
fn a_default_window_on_the_monitor_keeps_the_whole_dialog_inside_it() {
    let h = dialog_at(20.0);
    assert!(h.get_by_role_and_label(egui::Role::Dialog, "Scan and text recognition").rect().top() >= 0.0, "the title");
    assert!(buttons_bottom(&h) <= WINDOW.y, "and the buttons");
}

#[test]
fn a_window_with_room_shows_the_whole_dialog_with_nothing_to_scroll() {
    let h = dialog_in(TALL_WINDOW, TALL_MONITOR, 20.0);
    assert!(!has_scroll_bar(&h), "everything fits");
    assert!(buttons_bottom(&h) > h.get_by_label("Add searchable text").rect().bottom(), "the buttons are below what it asks");
    assert!(buttons_bottom(&h) <= TALL_WINDOW.y, "and inside the window");
}

#[test]
fn a_dialog_that_had_to_scroll_grows_back_when_the_window_is_maximized() {
    let mut h = dialog_in(TALL_WINDOW, TALL_MONITOR, 600.0);
    assert!(has_scroll_bar(&h), "the window runs off the monitor");
    let info = h.input_mut().viewports.entry(egui::ViewportId::ROOT).or_default();
    // Maximizing moves the window to the top of its monitor; a maximized window is still checked by
    // where it is (craft-window `visible_part`, #3737).
    info.maximized = Some(true);
    info.inner_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, TALL_WINDOW));
    h.run_steps(4);
    assert!(!has_scroll_bar(&h), "the system keeps a maximized window on the monitor");
}
