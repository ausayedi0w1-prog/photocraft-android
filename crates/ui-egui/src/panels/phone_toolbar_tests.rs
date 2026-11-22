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
    assert!(brush.top() > 700.0 && brush.bottom() <= 800.0, "the buttons sit at the bottom edge: {brush:?}");
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

/// A frame of the phone toolbar alone, at an explicit time: the long press is a clock, so the test
/// says what the clock says.
fn frame(app: &mut PhotocraftApp, ctx: &egui::Context, time: f64, events: Vec<egui::Event>) {
    let mut out = ctx.run_ui(
        egui::RawInput { time: Some(time), events, screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(400.0, 800.0))), ..Default::default() },
        |ui| phone_toolbar(app, ui),
    );
    out.textures_delta.clear();
}

/// Clickable buttons of the height the phone bar's tool buttons are, in row order.
fn bar_buttons(ctx: &egui::Context) -> Vec<Rect> {
    let mut rects: Vec<Rect> = ctx.viewport(|v| {
        v.prev_pass
            .widgets
            .layers()
            .flat_map(|(_, w)| w.iter())
            .filter(|w| (w.rect.height() - PHONE_ITEM_H).abs() < 0.5 && w.sense.senses_click())
            .map(|w| w.rect)
            .collect()
    });
    rects.sort_by(|a, b| a.left().total_cmp(&b.left()));
    rects
}

#[test]
fn a_long_press_opens_the_slots_flyout() {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    let ctx = egui::Context::default();
    PhotocraftApp::setup_context(&ctx, theme::ThemeKind::ALL[0]);
    egui_extras::install_image_loaders(&ctx);
    theme::install_fonts(&ctx);
    frame(&mut app, &ctx, 0.0, vec![]);
    frame(&mut app, &ctx, 0.1, vec![]);
    // The second slot (the marquee tools) is on screen in a 400-point window: the Brush's is
    // scrolled past the right edge.
    let index = 1;
    let at = bar_buttons(&ctx)[index].center();
    assert!(at.x < 400.0 && at.y > 700.0, "the pressed button is on screen: {at:?}");
    let pointer = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
    let key = egui::Id::unique(("phone-tool-slot", index));
    frame(&mut app, &ctx, 1.0, vec![egui::Event::PointerMoved(at), pointer(at, true)]);
    // 400 ms of holding: past the long press, which opens the slot's flyout above the bar.
    frame(&mut app, &ctx, 1.4, vec![]);
    assert_eq!(
        ctx.data(|d| d.get_temp::<(egui::Id, Rect)>(egui::Id::unique("phone-tool-flyout")).map(|(id, _)| id)),
        Some(key),
        "holding the marquee button opens its slot's flyout"
    );
    assert_eq!(app.ui.tool, Tool::RectMarquee, "holding a button selects the tool it shows");
    // The release that ends the hold is not a tap: the flyout stays open.
    frame(&mut app, &ctx, 1.45, vec![pointer(at, false)]);
    frame(&mut app, &ctx, 1.5, vec![]);
    assert!(ctx.data(|d| d.get_temp::<(egui::Id, Rect)>(egui::Id::unique("phone-tool-flyout"))).is_some(), "the flyout stays open");
    // Its rows are the slot's tools, finger-sized: the second one is the Elliptical Marquee.
    let mut rows: Vec<Rect> = ctx.viewport(|v| {
        v.prev_pass
            .widgets
            .layers()
            .flat_map(|(_, w)| w.iter())
            .filter(|w| (w.rect.height() - PHONE_FLYOUT_ROW_H).abs() < 0.5 && w.sense.senses_click())
            .map(|w| w.rect)
            .collect()
    });
    rows.sort_by(|a, b| a.top().total_cmp(&b.top()));
    assert!(rows.len() >= 2, "the flyout lists the slot's tools: {rows:?}");
    assert!(rows.iter().all(|r| r.top() < 740.0), "the flyout opens above the bar: {rows:?}");
    let row = rows[1].center();
    frame(&mut app, &ctx, 2.0, vec![egui::Event::PointerMoved(row), pointer(row, true)]);
    frame(&mut app, &ctx, 2.05, vec![pointer(row, false)]);
    assert_eq!(app.ui.tool, Tool::EllipseMarquee, "choosing a row picks that tool");
    frame(&mut app, &ctx, 2.1, vec![]);
    assert!(ctx.data(|d| d.get_temp::<(egui::Id, Rect)>(egui::Id::unique("phone-tool-flyout"))).is_none(), "and closes the flyout");
}
