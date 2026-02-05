use eframe::egui::{
    self, Color32, ColorImage, FontId, Pos2, Rect, Rounding, Stroke, TextureHandle, TextureOptions,
    Vec2,
};
use std::time::Instant;

use crate::theme::colors;

// E logo SVG
const E_SVG: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg width="159" height="127" viewBox="0 0 159 127" xmlns="http://www.w3.org/2000/svg">
  <path fill="#00FF96" d="m 28.68,90.67 c 0,0 -28.68,0 -28.68,0 0,0 0,36.41 0,36.41 0,0 158.65,0 158.65,0 0,0 0,-27.03 0,-27.03 0,0 -125.93,0 -129.97,0 0,-2.53 0,-9.38 0,-9.38 z"/>
  <path fill="#00FF96" d="m 0,36.3 c 0,0 28.68,0 28.68,0 0,0 0,-6.76 0,-9.27 4.04,0 129.97,0 129.97,0 0,0 0,-27.03 0,-27.03 C 158.65,0 0,0 0,0 Z"/>
  <path fill="#00FF96" d="m 0,77.06 c 0,0 158.65,0 158.65,0 0,0 0,-27.02 0,-27.02 0,0 -158.65,0 -158.65,0 z"/>
</svg>"##;

pub struct AboutScreen {
    pub open: bool,
    start_time: Instant,
    logo_texture: Option<TextureHandle>,
}

impl AboutScreen {
    pub fn new() -> Self {
        Self {
            open: false,
            start_time: Instant::now(),
            logo_texture: None,
        }
    }

    fn load_logo_texture(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_some() {
            return;
        }

        // Parse and rasterize SVG
        if let Ok(svg) = nsvg::parse_str(E_SVG, nsvg::Units::Pixel, 96.0) {
            let scale = 2.0; // Render at 2x for better quality
            if let Ok((w, h, data)) = svg.rasterize_to_raw_rgba(scale) {
                let image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &data);
                self.logo_texture = Some(ctx.load_texture("e_logo", image, TextureOptions::LINEAR));
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        // Load texture if needed
        self.load_logo_texture(ctx);

        // Reset animation timer when opened
        ctx.request_repaint();

        let screen_rect = ctx.screen_rect();

        // Semi-transparent backdrop
        egui::Area::new(egui::Id::new("about_backdrop"))
            .fixed_pos(Pos2::ZERO)
            .order(egui::Order::Middle)
            .show(ctx, |ui| {
                let painter = ui.painter();
                painter.rect_filled(screen_rect, Rounding::ZERO, Color32::from_black_alpha(180));

                // Click backdrop to close
                if ui.input(|i| i.pointer.any_click()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        let pack_height = screen_rect.height() * 0.85;
                        let pack_width = pack_height * 0.625;
                        let pack_rect = Rect::from_center_size(
                            screen_rect.center(),
                            Vec2::new(pack_width, pack_height),
                        );
                        if !pack_rect.contains(pos) {
                            self.open = false;
                        }
                    }
                }
            });

        egui::Area::new(egui::Id::new("about_screen"))
            .fixed_pos(Pos2::ZERO)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let painter = ui.painter();

                // Calculate pack dimensions
                let pack_height = screen_rect.height() * 0.85;
                let pack_width = pack_height * 0.625;
                let pack_rect = Rect::from_center_size(
                    screen_rect.center(),
                    Vec2::new(pack_width, pack_height),
                );

                // Draw pack background + intense menthol-style splatters
                self.draw_pack_background(painter, pack_rect);

                // Central glowing circle/portal
                let circle_center = Pos2::new(
                    pack_rect.center().x,
                    pack_rect.center().y + pack_height * 0.02,
                );
                let circle_radius = pack_width * 0.38;

                // Outer glow layers
                for i in (1..=4).rev() {
                    let alpha = 30 + (i * 15) as u8;
                    let r = circle_radius + (5 - i) as f32 * 8.0;
                    painter.circle_filled(
                        circle_center,
                        r,
                        Color32::from_rgba_unmultiplied(0, 255, 150, alpha),
                    );
                }

                // Main circle with dark center
                painter.circle_filled(circle_center, circle_radius, Color32::from_rgb(10, 50, 40));
                painter.circle_stroke(
                    circle_center,
                    circle_radius,
                    Stroke::new(3.0, colors::ELECTRIC_GREEN),
                );

                // Inner glow
                painter.circle_stroke(
                    circle_center,
                    circle_radius * 0.85,
                    Stroke::new(2.0, Color32::from_rgba_unmultiplied(0, 200, 120, 100)),
                );

                // Draw EVE logo from texture
                if let Some(texture) = &self.logo_texture {
                    let logo_size = circle_radius * 1.2;
                    let aspect = texture.aspect_ratio();
                    let logo_rect = Rect::from_center_size(
                        circle_center,
                        Vec2::new(logo_size, logo_size / aspect),
                    );

                    // Apply pulsing glow effect via tint
                    let elapsed = self.start_time.elapsed().as_secs_f32();
                    let glow = 0.7 + 0.3 * (elapsed * 1.5).sin();
                    let tint = Color32::from_rgba_unmultiplied(
                        (255.0 * glow) as u8,
                        255,
                        (255.0 * glow) as u8,
                        (255.0 * glow) as u8,
                    );

                    painter.image(
                        texture.id(),
                        logo_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        tint,
                    );
                }

                // Top text area with dark backing
                let text_top = pack_rect.top() + pack_height * 0.04;
                let top_text_rect = Rect::from_min_max(
                    Pos2::new(pack_rect.left() + 10.0, text_top - 5.0),
                    Pos2::new(pack_rect.right() - 10.0, text_top + pack_width * 0.47),
                );
                painter.rect_filled(
                    top_text_rect,
                    Rounding::same(8.0),
                    Color32::from_black_alpha(160),
                );

                // "PACK" - large white text with shadow
                let pack_pos = Pos2::new(pack_rect.center().x, text_top + 5.0);
                painter.text(
                    Pos2::new(pack_pos.x + 2.0, pack_pos.y + 2.0),
                    egui::Align2::CENTER_TOP,
                    "PACK",
                    FontId::proportional(pack_width * 0.28),
                    Color32::from_black_alpha(150),
                );
                painter.text(
                    pack_pos,
                    egui::Align2::CENTER_TOP,
                    "PACK",
                    FontId::proportional(pack_width * 0.28),
                    Color32::WHITE,
                );

                // "PREFERENCES" - green text with shadow
                let pref_pos = Pos2::new(pack_rect.center().x, text_top + pack_width * 0.22);
                painter.text(
                    Pos2::new(pref_pos.x + 1.0, pref_pos.y + 1.0),
                    egui::Align2::CENTER_TOP,
                    "PREFERENCES",
                    FontId::proportional(pack_width * 0.13),
                    Color32::from_black_alpha(150),
                );
                painter.text(
                    pref_pos,
                    egui::Align2::CENTER_TOP,
                    "PREFERENCES",
                    FontId::proportional(pack_width * 0.13),
                    colors::ELECTRIC_GREEN,
                );

                // "Settings Manager" with shadow
                let settings_pos = Pos2::new(pack_rect.center().x, text_top + pack_width * 0.35);
                painter.text(
                    Pos2::new(settings_pos.x + 1.0, settings_pos.y + 1.0),
                    egui::Align2::CENTER_TOP,
                    "Settings Manager",
                    FontId::proportional(pack_width * 0.08),
                    Color32::from_black_alpha(200),
                );
                painter.text(
                    settings_pos,
                    egui::Align2::CENTER_TOP,
                    "Settings Manager",
                    FontId::proportional(pack_width * 0.08),
                    Color32::WHITE,
                );

                // Bottom section with dark backing
                let bottom_section_top = pack_rect.bottom() - pack_height * 0.24;
                let bottom_text_rect = Rect::from_min_max(
                    Pos2::new(pack_rect.left() + 10.0, bottom_section_top - 8.0),
                    Pos2::new(pack_rect.right() - 10.0, pack_rect.bottom() - 8.0),
                );
                painter.rect_filled(
                    bottom_text_rect,
                    Rounding::same(8.0),
                    Color32::from_black_alpha(180),
                );

                // Version
                painter.text(
                    Pos2::new(pack_rect.center().x, bottom_section_top),
                    egui::Align2::CENTER_TOP,
                    concat!("v", env!("CARGO_PKG_VERSION")),
                    FontId::proportional(pack_width * 0.07),
                    colors::ELECTRIC_GREEN,
                );

                // Tagline
                painter.text(
                    Pos2::new(pack_rect.center().x, bottom_section_top + pack_width * 0.10),
                    egui::Align2::CENTER_TOP,
                    "An EVE Online settings manager",
                    FontId::proportional(pack_width * 0.055),
                    colors::TEXT_WHITE,
                );

                // Credits
                painter.text(
                    Pos2::new(pack_rect.center().x, bottom_section_top + pack_width * 0.19),
                    egui::Align2::CENTER_TOP,
                    "Replicate settings across characters",
                    FontId::proportional(pack_width * 0.045),
                    colors::TEXT_DIM,
                );

                // Author
                painter.text(
                    Pos2::new(pack_rect.center().x, bottom_section_top + pack_width * 0.24),
                    egui::Align2::CENTER_TOP,
                    "By Sopleb",
                    FontId::proportional(pack_width * 0.045),
                    colors::TEXT_WHITE,
                );

                // Close hint
                painter.text(
                    Pos2::new(
                        pack_rect.center().x,
                        pack_rect.bottom() - pack_height * 0.025,
                    ),
                    egui::Align2::CENTER_BOTTOM,
                    "click outside to close",
                    FontId::proportional(pack_width * 0.04),
                    colors::BRIGHT_CYAN,
                );
            });
    }

    fn draw_pack_background(&self, painter: &egui::Painter, pack_rect: Rect) {
        let center = pack_rect.center();
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let w = pack_rect.width();
        let h = pack_rect.height();
        // Very dark teal-cyan base
        painter.rect_filled(
            pack_rect,
            Rounding::same(12.0),
            Color32::from_rgb(4, 30, 40), // darker still for higher contrast pops
        );
        // ───────────────────────────────────────────────
        // ULTRA CHAOTIC menthol explosion – with smooth color/size transitions
        // ───────────────────────────────────────────────
        let palette = [
            colors::ELECTRIC_GREEN,
            colors::CYAN,
            colors::NEON_GREEN,
            colors::BRIGHT_TEAL,
            colors::BRIGHT_CYAN,
            Color32::from_rgb(140, 255, 220), // brighter mint
            Color32::from_rgb(0, 255, 180),   // intense turquoise
            Color32::from_rgb(100, 255, 140), // wild acid green
            Color32::from_rgb(0, 200, 255),   // electric blue-cyan
        ];
        let count = 520;
        let max_spread = w.max(h) * 0.75; // increased overall reach
        let min_radius = w.max(h) * 0.18; // enforced min distance – key fix
        for i in 0..count {
            let seed = i as f32 * 0.618 + elapsed * 0.3;
            let angle = seed * std::f32::consts::PI * 0.618 + (seed * 1.1).sin() * 2.4;
            let radius_factor = (i as f32).sqrt() * 0.68 + (seed * 0.8).cos() * 0.55;
            // Core fix: add strong minimum distance
            let raw_dist = radius_factor * max_spread;
            let dist = raw_dist.max(min_radius);
            // Reduce inward turbulence strength
            let turbulence = 14.0 * (seed * 0.9).sin().powi(2); // was 18.0 → less extreme pull
            let offset_x = dist * angle.cos() + turbulence * (seed * 1.3).cos();
            let offset_y = dist * angle.sin() + turbulence * (seed * 1.7).sin();
            let pos = Pos2::new(center.x + offset_x, center.y + offset_y);
            // Wilder radius variation – but stronger taper for inner positions
            let base_radius = 130.0 - (dist / max_spread) * 110.0; // taper based on actual dist
            let size_breath = 0.75 + 0.25 * (elapsed * 1.1 + seed * 1.4).sin().abs().powf(1.6); // smooth breathing
            let radius_var = 24.0 * (seed * 2.3 + elapsed * 0.65).cos().abs(); // independent variation
            let radius = (base_radius * size_breath + radius_var).max(6.0);
            // Smooth color lerping instead of stepped index
            let hue_phase = (elapsed * 0.14 + seed * 0.08) % 1.0;
            let idx_a = (hue_phase * palette.len() as f32) as usize;
            let idx_b = (idx_a + 1) % palette.len();
            let t = (hue_phase * palette.len() as f32).fract();
            let col_a = palette[idx_a];
            let col_b = palette[idx_b];
            let r = (col_a.r() as f32 * (1.0 - t) + col_b.r() as f32 * t) as u8;
            let g = (col_a.g() as f32 * (1.0 - t) + col_b.g() as f32 * t) as u8;
            let b = (col_a.b() as f32 * (1.0 - t) + col_b.b() as f32 * t) as u8;
            // Volatile alpha with smoothed pulsing
            let base_alpha = 70.0 + 190.0 * (1.0 - radius_factor * 0.55);
            let alpha_breath = 0.60 + 0.40 * (elapsed * 1.3 + seed * 2.3).sin().abs().powf(1.5);
            let alpha = (base_alpha * alpha_breath).clamp(45.0, 250.0) as u8;
            let fill = Color32::from_rgba_unmultiplied(r, g, b, alpha);
            painter.circle_filled(pos, radius, fill);
        }
        // ───────────────────────────────────────────────
        // Smooth orbiting droplets / sparks
        // ───────────────────────────────────────────────
        for i in 0..220 {
            let seed = i as f32 * 3.1;
            let t = elapsed * 0.6 + seed;
            // Spread evenly around circle with golden angle
            let angle = i as f32 * 2.399 + t * 0.35;
            // Push droplets farther out
            let min_droplet_dist = w.max(h) * 0.22;
            let raw_dist = 50.0 + ((i % 70) as f32 * 4.5) + (t * 0.9).sin().abs() * 45.0;
            let dist = raw_dist.max(min_droplet_dist);
            let droplet_pos =
                Pos2::new(center.x + dist * angle.cos(), center.y + dist * angle.sin());
            // Smooth size variation using sin
            let droplet_size = 5.0 + 12.0 * (t * 0.8 + seed * 0.3).sin().abs();
            // Smooth alpha variation
            let alpha = (80.0 + 100.0 * (t * 0.5 + seed * 0.2).sin().abs()) as u8;
            let col = match i % 5 {
                0 => colors::BRIGHT_CYAN,
                1 => Color32::WHITE,
                2 => colors::NEON_GREEN,
                3 => colors::ELECTRIC_GREEN,
                _ => Color32::from_rgb(255, 255, 200),
            };
            painter.circle_filled(
                droplet_pos,
                droplet_size,
                Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), alpha),
            );
        }
        // Intense vignette – deepens the chaos at edges
        let vignette_alpha = 90 + (40.0 * (elapsed * 0.5).sin().abs()) as u8;
        let vignette = Color32::from_black_alpha(vignette_alpha);
        painter.rect_filled(pack_rect, Rounding::same(12.0), vignette);
        // Pulsing outer rim with more variation – safe multiply
        let rim_intensity = (0.35 + 0.3 * (elapsed * 1.2).sin().powi(2)).clamp(0.0, 1.0);
        painter.rect_stroke(
            pack_rect,
            Rounding::same(12.0),
            Stroke::new(5.0, colors::BRIGHT_TEAL.linear_multiply(rim_intensity)),
        );
    }
}
