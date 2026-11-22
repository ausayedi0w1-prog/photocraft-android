//! The phone layout's bottom tool bar: every slot in the row with its icon and name, the row
//! itself as a sideways-scrolling strip at the bottom edge, taps that pick the tool, and the long
//! press that opens a slot's flyout.

use super::*;
use egui_kittest::kittest::Queryable;

fn phone_harness() -> egui_kittest::Harness<'static, PhotocraftApp> {
    let mut ready = false;
    let mut harness = egui_kittest::Harness::builder().with_size(vec2(400.0, 800.0)).build_ui_state(
        move |ui, app: &mut PhotocraftApp| {
            if !ready {
                egui_extras::install_image_loaders(ui.ctx());
                theme::install_fonts(ui.ctx());
                theme::apply(ui.ctx(), theme::ThemeKind::ALL[0]);
                ready = true;
                return;
            }
            phone_toolbar(app, ui);
        },
        PhotocraftApp::new(photocraft_engine::Session::new(), Default::default()),
    );
    harness.run_steps(4);
    harness
}

/// The tool a slot's button shows with no history: the app starts on the Brush's slot.
fn shown_tool(slot: &[Tool]) -> Tool {
    slot_tool(Tool::Brush, &[], slot)
}

#[test]
fn every_slot_has_a_button_with_its_icon_and_name() {
    let harness = phone_harness();
    let sections = visible_sections(&[]);
    assert_eq!(sections.iter().flatten().count(), TOOL_SECTIONS.iter().flat_map(|s| s.iter()).count(), "no slot is dropped");
    for (_, slot) in sections.iter().flatten() {
        let label = phone_tool_label(shown_tool(slot));
        assert!(harness.query_by_label(label).is_some(), "no button named {label:?}");
    }
    // The tools' names, not the desktop tooltips' wordier ones.
    assert!(harness.query_by_label("Brush").is_some() && harness.query_by_label("Brush Tool").is_none());
}

#[test]
fn phone_labels_drop_only_the_tool_suffix() {
    assert_eq!(phone_tool_label(Tool::Brush), "Brush");
    assert_eq!(phone_tool_label(Tool::Eraser), "Eraser");
    assert_eq!(phone_tool_label(Tool::ColorReplacement), "Color Replacement");
    assert_eq!(phone_tool_label(Tool::RectMarquee), "Rectangular Marquee");
}

#[test]
fn the_bar_runs_along_the_bottom_edge_and_the_side_rail_is_gone() {
    let harness = phone_harness();
    // The tool row is the sideways-scrolling strip, at the window's bottom edge and as wide as the
    // window: the side rail would have been a column on the left.
    let strip = crate::scroll_strip::strips(&harness.ctx).first().copied().expect("the tool row's strip");
    assert!(strip.top() > 700.0 && strip.bottom() <= 800.5, "the tools sit at the bottom edge: {strip:?}");
    assert!(strip.left() < 120.0 && strip.width() > 250.0, "the row starts at the leading edge: {strip:?}");
    let brush = harness.get_by_label("Brush").rect();
    assert!(brush.top() > strip.top() && brush.bottom() <= strip.bottom(), "the button is inside the bar: {brush:?}");
    assert!(brush.height() >= 44.0, "finger-sized buttons: {brush:?}");
    // The chips are pinned at the leading edge, left of the scrolling tools.
    let chips = harness.get_by_label("Switch Foreground and Background Colors (X)").rect();
    assert!(chips.left() < brush.left() && chips.bottom() <= 800.5, "the chips lead the row: {chips:?}");
}

#[test]
fn the_tool_row_is_a_sideways_scrolling_strip() {
    let harness = phone_harness();
    let strips = crate::scroll_strip::strips(&harness.ctx);
    assert_eq!(strips.len(), 1, "the overflowing tool row is the strip the touch router swipes: {strips:?}");
    let strip = strips.first().copied().expect("the tool row's strip");
    assert!(strip.top() > 700.0, "a sideways swipe at the bottom edge scrolls the tools: {strip:?}");
    assert!(strip.width() > 300.0, "{strip:?}");
}

#[test]
fn tapping_a_tool_selects_it() {
    let mut harness = phone_harness();
    // The app starts on Move; the next slot's button is on screen.
    assert_eq!(harness.state().ui.tool, Tool::Move);
    harness.get_by_label("Rectangular Marquee").click();
    harness.run_steps(3);
    assert_eq!(harness.state().ui.tool, Tool::RectMarquee);
}

#[test]
fn a_long_press_opens_the_slots_flyout() {
    let mut harness = phone_harness();
    let button = harness.get_by_label("Brush").rect();
    harness.event(egui::Event::PointerMoved(button.center()));
    harness.run_steps(1);
    harness.event(egui::Event::PointerButton {
        pos: button.center(),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    // Each step is 0.25 s in the harness: four pass the long press's 350 ms.
    harness.run_steps(4);
    assert!(harness.query_by_label("Pencil Tool").is_some(), "holding the Brush button lists the slot's other tools");
    assert_eq!(harness.state().ui.tool, Tool::Brush, "holding a button selects the tool it shows");
    // Choosing a row selects that tool and closes the flyout.
    harness.get_by_label("Pencil Tool").click();
    harness.run_steps(3);
    assert_eq!(harness.state().ui.tool, Tool::Pencil);
    assert!(harness.query_by_label("Pencil Tool").is_none(), "the flyout closes on its choice");
}
