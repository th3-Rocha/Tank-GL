//! Tank Studio: editor for the M-VAVE Tank-G in egui + fastframe.
//!
//! One copy per user: two would fight over the MIDI port and one would get
//! the other's replies. Opening it again brings the existing window forward.

mod app;
mod community;
mod device;
mod dialog;
mod library;
mod pedalboard;
mod theme;
mod widgets;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use fastframe_instance::{Claim, Slot};

const APP_ID: &str = "io.github.th3rocha.tankstudio";

fn main() -> eframe::Result {
    let show = Arc::new(AtomicBool::new(false));
    let ctx_slot: Arc<Mutex<Option<egui::Context>>> = Arc::default();
    let _guard = {
        let show = show.clone();
        let ctx_slot = ctx_slot.clone();
        match Slot::new(APP_ID).claim("show", move |req| {
            (req == "show").then(|| {
                show.store(true, Ordering::Relaxed);
                if let Some(ctx) = ctx_slot.lock().ok().and_then(|c| c.clone()) {
                    ctx.request_repaint();
                }
                "ok".to_owned()
            })
        }) {
            Claim::First(guard) => guard,
            Claim::Running(_) | Claim::Declined | Claim::Unanswered => {
                eprintln!("Tank Studio is already open.");
                return Ok(());
            }
        }
    };

    let options = || eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Tank Studio")
            .with_app_id(APP_ID)
            .with_inner_size([1280.0, 860.0])
            .with_min_inner_size([700.0, 600.0]),
        // With vsync, a swap on a hidden Wayland window (another workspace)
        // waits for a frame callback that never comes. The event loop stalls
        // and Hyprland shows "Application Not Responding". egui only redraws
        // when something changes, so no vsync costs nothing.
        glow_options: eframe::egui_glow::GlowConfiguration {
            vsync: false,
            ..Default::default()
        },
        ..Default::default()
    };

    fastframe_graphics::Graphics::new("Tank Studio").run_native(options, || {
        let show = show.clone();
        let ctx_slot = ctx_slot.clone();
        Box::new(move |cc| {
            setup(&cc.egui_ctx);
            if let Ok(mut c) = ctx_slot.lock() {
                *c = Some(cc.egui_ctx.clone());
            }
            Ok(Box::new(app::TankApp::new(&cc.egui_ctx, show)))
        })
    })
}

fn setup(ctx: &egui::Context) {
    let mut fonts = fastframe_fonts::FontSetup::default().definitions();
    let rendering = fastframe_text::detect();
    rendering.apply_to(&mut fonts);
    ctx.set_fonts(fonts);
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, theme::visuals());
    ctx.all_styles_mut(|style| {
        rendering.apply_to_visuals(&mut style.visuals);
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
    });
}
