//! Fork-only (Lars-/solidcraft, branch `solidworks-style`): one Interface choice that sets
//! SolidCraft's own options the way someone coming from SolidWorks expects them. Everything it
//! sets stays an ordinary preference that can be changed on its own afterwards; upstream offers
//! each of those options separately (#6489, #6490, #6491, #6492, #6707-#6710, #7243, #7246, #7247,
//! #7904, #7905). It also gives the SolidWorks default keys to the commands SolidCraft has.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::SolveApp;
use crate::dialog_column::DialogPlace;
use crate::prefs::{DimensionInput, HistorySketches, OriginPlaneNames, Prefs};
use crate::ui_kit::{self, DialogButtons};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiStyle {
    #[default]
    Standard,
    Solidworks,
}

impl UiStyle {
    pub const ALL: [Self; 2] = [Self::Standard, Self::Solidworks];

    fn message(self) -> &'static str {
        match self {
            Self::Standard => "prefs.ui_style_standard",
            Self::Solidworks => "prefs.ui_style_solidworks",
        }
    }
}

/// Picks `style`. SolidWorks-style sets the mouse, the History, the plane names, where dialogs
/// and their OK and Cancel sit, the status bar, the dimension window, the Line tool's drag, the
/// Sketch tab, the cut buttons, the view bar at the top and two keys; Standard puts those same
/// options back to their defaults. Options outside this list are left alone.
pub fn choose(app: &mut SolveApp, style: UiStyle) -> Result<(), String> {
    let sw = style == UiStyle::Solidworks;
    // The History position first: changing the dock unfolds the browser.
    app.run("view.history_layout", json!({ "dock": if sw { "under_browser" } else { "bottom" } }))?;
    let d = Prefs::default();
    let p = &mut app.preferences;
    if sw {
        crate::prefs::choose_nav_preset(p, crate::prefs::nav_preset("solidworks"));
    } else {
        p.nav = d.nav;
        p.zoom_reverse = d.zoom_reverse;
        p.middle_double_click = d.middle_double_click;
        p.right_drag_orbits = d.right_drag_orbits;
    }
    p.timeline_left_browser_collapsed = if sw { true } else { d.timeline_left_browser_collapsed };
    p.history_sketches = if sw { HistorySketches::UnderFeature } else { d.history_sketches };
    p.timeline_origin_bodies = if sw { true } else { d.timeline_origin_bodies };
    p.origin_plane_names = if sw { OriginPlaneNames::TopFrontRight } else { d.origin_plane_names };
    p.dialog_place = if sw { DialogPlace::LeftColumn } else { d.dialog_place };
    p.dialog_buttons = if sw { DialogButtons::Top } else { d.dialog_buttons };
    p.view_confirm_corner = if sw { true } else { d.view_confirm_corner };
    p.status_bar = if sw { true } else { d.status_bar };
    p.dimension_input = if sw { DimensionInput::Window } else { d.dimension_input };
    p.line_drag_draws_one = if sw { true } else { d.line_drag_draws_one };
    p.sketch_tab_always = if sw { true } else { d.sketch_tab_always };
    p.cut_buttons = if sw { true } else { d.cut_buttons };
    // The view bar at the top centre of the view.
    p.navigation_bar.edge = if sw { crate::navigation_bar::Edge::Top } else { d.navigation_bar.edge };
    p.navigation_bar.position = if sw { crate::navigation_bar::Position::Centre } else { d.navigation_bar.position };
    p.ui_style = style;
    keys(app, sw)
}

/// SolidWorks' default keys for the commands SolidCraft has: F fits the view, Enter repeats the
/// last command. Most of its other defaults (rebuild, zoom steps, previous view, recent files, full
/// screen) have no SolidCraft command yet; S stays SolidCraft's own command search.
const SOLIDWORKS_KEYS: [(&str, &str); 2] = [("view.fit", "F"), ("edit.repeat", "Enter")];

/// SolidWorks-style binds [`SOLIDWORKS_KEYS`], taking each key from the command that had it;
/// Standard gives those commands, and the ones that lost a key to them, their own keys back.
/// Other keys the user changed are left alone.
fn keys(app: &mut SolveApp, sw: bool) -> Result<(), String> {
    for (id, key) in SOLIDWORKS_KEYS {
        if sw {
            crate::keymap::bind(app, id, key, true)?;
            continue;
        }
        crate::keymap::reset(app, Some(id));
        let lost: Vec<String> = app
            .keymap
            .custom
            .iter()
            .filter(|(other, now)| now.is_empty() && solidcraft_engine::find_command(other).and_then(|c| c.shortcut) == Some(key))
            .map(|(other, _)| other.clone())
            .collect();
        for other in lost {
            crate::keymap::reset(app, Some(&other));
        }
    }
    Ok(())
}

/// The Interface page's "Interface style" section.
pub fn section(app: &mut SolveApp, ui: &mut egui::Ui) {
    ui_kit::settings_section(ui, "ui_style", app.tr("prefs.ui_style"), |ui| {
        let labels: Vec<&str> = UiStyle::ALL.iter().map(|style| crate::i18n::tr(style.message())).collect();
        let mut selected = UiStyle::ALL.iter().position(|style| *style == app.preferences.ui_style).unwrap_or(0);
        if ui_kit::settings_segments(ui, app.tr("prefs.ui_style"), &mut selected, &labels)
            && let Some(style) = UiStyle::ALL.get(selected)
            && let Err(e) = choose(app, *style)
        {
            app.set_status(format!("Interface style: {e}"), true);
        }
        for (style, key) in UiStyle::ALL.iter().zip(["standard", "solidworks"]) {
            if let Some(at) = crate::scenario::handle_at(&format!("dialog:{}:{}", app.tr("prefs.ui_style"), crate::i18n::tr(style.message()))) {
                crate::scenario::publish_handle(&format!("prefs:ui_style:{key}"), at);
            }
        }
        ui_kit::text(ui, app.tr("prefs.ui_style_note"), ui_kit::Text::Hint("ui_style"));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solidworks_style_sets_its_options_and_standard_puts_them_back() {
        let mut app = SolveApp::new(solidcraft_engine::Session::default(), Default::default());
        app.preferences.theme = "light".into();
        choose(&mut app, UiStyle::Solidworks).unwrap();
        let p = &app.preferences;
        assert_eq!((p.nav.as_str(), p.zoom_reverse, p.middle_double_click.as_str(), p.right_drag_orbits), ("solidworks", true, "fit", false));
        assert!(p.timeline_left && p.timeline_under_browser && p.timeline_left_browser_collapsed);
        assert_eq!((p.history_sketches, p.timeline_origin_bodies), (HistorySketches::UnderFeature, true));
        assert_eq!((p.origin_plane_names, p.ui_style), (OriginPlaneNames::TopFrontRight, UiStyle::Solidworks));
        assert_eq!((p.dialog_place, p.dialog_buttons, p.view_confirm_corner), (DialogPlace::LeftColumn, DialogButtons::Top, true));
        assert_eq!((p.status_bar, p.dimension_input, p.line_drag_draws_one), (true, DimensionInput::Window, true));
        assert!(p.sketch_tab_always && p.cut_buttons);
        assert_eq!(p.navigation_bar.edge, crate::navigation_bar::Edge::Top);
        let key = |app: &SolveApp, id: &str| crate::keymap::effective(app, id);
        assert_eq!((key(&app, "view.fit").as_deref(), key(&app, "edit.repeat").as_deref()), (Some("F"), Some("Enter")));
        assert_eq!(key(&app, "solid.fillet"), None, "F was Fillet's");
        assert_eq!(p.theme, "light", "options outside the style are left alone");

        choose(&mut app, UiStyle::Standard).unwrap();
        let (p, d) = (&app.preferences, Prefs::default());
        assert_eq!(
            (p.nav.as_str(), p.zoom_reverse, p.middle_double_click.as_str()),
            (d.nav.as_str(), d.zoom_reverse, d.middle_double_click.as_str())
        );
        assert!(p.right_drag_orbits);
        assert_eq!((p.timeline_left, p.timeline_under_browser, p.timeline_left_browser_collapsed), (false, false, false));
        assert_eq!((p.history_sketches, p.timeline_origin_bodies, p.origin_plane_names), (d.history_sketches, false, d.origin_plane_names));
        assert_eq!((p.dialog_place, p.dialog_buttons, p.view_confirm_corner), (d.dialog_place, d.dialog_buttons, d.view_confirm_corner));
        assert_eq!((p.status_bar, p.dimension_input, p.line_drag_draws_one), (d.status_bar, d.dimension_input, d.line_drag_draws_one));
        assert_eq!((p.sketch_tab_always, p.cut_buttons), (d.sketch_tab_always, d.cut_buttons));
        assert_eq!(p.navigation_bar.edge, d.navigation_bar.edge);
        assert!(app.keymap.custom.is_empty(), "every key back: {:?}", app.keymap.custom);
        assert_eq!(crate::keymap::effective(&app, "solid.fillet").as_deref(), Some("F"));
        assert_eq!((p.ui_style, p.theme.as_str()), (UiStyle::Standard, "light"));
    }
}
