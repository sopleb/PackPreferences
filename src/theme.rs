use eframe::egui::{self, Color32, Rounding, Stroke, Style, Visuals};

// Color palette based on menthol cigarette pack aesthetic
#[allow(dead_code)]
pub mod colors {
    use super::Color32;

    // Primary teal/cyan tones
    pub const DARK_TEAL: Color32 = Color32::from_rgb(15, 45, 55);
    pub const DEEP_TEAL: Color32 = Color32::from_rgb(20, 60, 70);
    pub const TEAL: Color32 = Color32::from_rgb(30, 90, 100);
    pub const BRIGHT_TEAL: Color32 = Color32::from_rgb(40, 140, 150);

    // Electric green accents
    pub const ELECTRIC_GREEN: Color32 = Color32::from_rgb(0, 255, 150);
    pub const NEON_GREEN: Color32 = Color32::from_rgb(50, 255, 100);
    pub const LIME: Color32 = Color32::from_rgb(150, 255, 100);

    // Cyan highlights
    pub const CYAN: Color32 = Color32::from_rgb(0, 220, 220);
    pub const BRIGHT_CYAN: Color32 = Color32::from_rgb(100, 255, 255);

    // Text colors
    pub const TEXT_WHITE: Color32 = Color32::from_rgb(240, 255, 250);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(150, 180, 175);

    // Widget colors
    pub const WIDGET_BG: Color32 = Color32::from_rgb(25, 70, 80);
    pub const WIDGET_BG_HOVER: Color32 = Color32::from_rgb(35, 95, 105);
    pub const WIDGET_BG_ACTIVE: Color32 = Color32::from_rgb(40, 120, 130);

    // Selection/highlight
    pub const SELECTION: Color32 = Color32::from_rgb(0, 180, 120);
    pub const SELECTION_DIM: Color32 = Color32::from_rgb(0, 100, 80);
}

pub fn apply_pack_theme(ctx: &egui::Context) {
    let mut style = Style::default();

    // Customize visuals
    let mut visuals = Visuals::dark();

    // Window/panel backgrounds
    visuals.panel_fill = colors::DARK_TEAL;
    visuals.window_fill = colors::DEEP_TEAL;
    visuals.extreme_bg_color = colors::DARK_TEAL;
    visuals.faint_bg_color = colors::WIDGET_BG;

    // Widget styling
    visuals.widgets.noninteractive.bg_fill = colors::WIDGET_BG;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT_DIM);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors::TEAL);
    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);

    visuals.widgets.inactive.bg_fill = colors::WIDGET_BG;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::TEXT_WHITE);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors::BRIGHT_TEAL);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);

    visuals.widgets.hovered.bg_fill = colors::WIDGET_BG_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, colors::ELECTRIC_GREEN);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, colors::ELECTRIC_GREEN);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);

    visuals.widgets.active.bg_fill = colors::WIDGET_BG_ACTIVE;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, colors::NEON_GREEN);
    visuals.widgets.active.bg_stroke = Stroke::new(2.0, colors::NEON_GREEN);
    visuals.widgets.active.rounding = Rounding::same(4.0);

    visuals.widgets.open.bg_fill = colors::WIDGET_BG_ACTIVE;
    visuals.widgets.open.fg_stroke = Stroke::new(1.5, colors::CYAN);
    visuals.widgets.open.bg_stroke = Stroke::new(1.5, colors::CYAN);
    visuals.widgets.open.rounding = Rounding::same(4.0);

    // Selection color
    visuals.selection.bg_fill = colors::SELECTION;
    visuals.selection.stroke = Stroke::new(1.0, colors::ELECTRIC_GREEN);

    // Hyperlinks
    visuals.hyperlink_color = colors::BRIGHT_CYAN;

    // Window styling
    visuals.window_rounding = Rounding::same(8.0);
    visuals.window_stroke = Stroke::new(2.0, colors::BRIGHT_TEAL);
    visuals.window_shadow.color = Color32::from_black_alpha(100);

    // Popup styling
    visuals.popup_shadow.color = Color32::from_black_alpha(120);

    // Separator
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors::TEAL);

    // Override text color
    visuals.override_text_color = Some(colors::TEXT_WHITE);

    style.visuals = visuals;

    // Spacing adjustments
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);

    ctx.set_style(style);
}

/// Returns the app title with styled colors for the header
pub fn styled_title(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("PACK")
                .size(28.0)
                .strong()
                .color(colors::TEXT_WHITE),
        );
        ui.add_space(-4.0);
        ui.label(
            egui::RichText::new("PREFERENCES")
                .size(16.0)
                .strong()
                .color(colors::ELECTRIC_GREEN),
        );
    });
    ui.label(
        egui::RichText::new("EVE Online Settings Manager")
            .size(11.0)
            .color(colors::TEXT_DIM),
    );
}
