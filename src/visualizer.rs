use eframe::egui;
use egui_plot::{Plot, PlotPoints, Points};
use std::sync::{Arc, Mutex};

use crate::point_cloud::CloudPoint;
use crate::order_book::Side;

use crate::tda::compute_betti;

// use egui_plot::{PlotPoints, Line};

#[derive(Clone)]
pub struct Visualizer {
    pub clouds: std::collections::HashMap<String, Vec<CloudPoint>>,
}

impl Visualizer {
    pub fn update_cloud(&mut self, symbol: &str, cloud: Vec<CloudPoint>) {
        const MAX_POINTS: usize = 10_000; // adjust as you like

        let entry = self.clouds.entry(symbol.to_string()).or_default();
        entry.extend(cloud);

        if entry.len() > MAX_POINTS {
            let drop = entry.len() - MAX_POINTS;
            entry.drain(0..drop);
        }
    }
}

pub struct GuiApp {
    shared: Arc<Mutex<Visualizer>>,
    active_symbol: String,
}

impl GuiApp {
    pub fn new(shared: Arc<Mutex<Visualizer>>) -> Self {
        Self { 
            shared,
            active_symbol: String::new(), 
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let guard = self.shared.lock().unwrap();
        let clouds = &guard.clouds;

        // Initialize active symbol once
        if self.active_symbol.is_empty() {
            if let Some(first) = clouds.keys().next() {
                self.active_symbol = first.clone();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // --- Tab bar ---
            ui.horizontal(|ui| {
                for symbol in clouds.keys() {
                    let selected = *symbol == self.active_symbol;
                    let label = if selected {
                        format!("[{}]", symbol)
                    } else {
                        symbol.clone()
                    };

                    if ui.button(label).clicked() {
                        self.active_symbol = symbol.clone();
                    }
                }
            });

            ui.separator();

            // --- Draw ONLY the active symbol ---
            if let Some(cloud) = clouds.get(&self.active_symbol) {
                ui.heading(format!("Point Cloud: {}", self.active_symbol));

                // Convert CloudPoint to Vector2
                let pts: Vec<_> = cloud.iter().map(|p| to_vec2(p)).collect();

                // epsilon scale
                let eps = 0.02;

                // Compute Betti numbers
                let (betti0, betti1) = compute_betti(&pts, eps);

                // Show them in the UI
                ui.label(format!("Betti-0 (components): {}", betti0));
                ui.label(format!("Betti-1 (cycles): {}", betti1));
                ui.separator();

                Plot::new(&self.active_symbol)
                    .allow_zoom(true)
                    .allow_drag(true)
                    .show(ui, |plot_ui| {
                        let min_ts = cloud.iter().map(|p| p.ts).min().unwrap_or(0);
                        let max_ts = cloud.iter().map(|p| p.ts).max().unwrap_or(0);
                        let span = (max_ts - min_ts).max(1) as f32;

                        for p in cloud {
                            let base_color = match p.side {
                                Side::Bid => egui::Color32::BLUE,
                                Side::Ask => egui::Color32::RED,
                            };

                            let age = (max_ts - p.ts) as f32 / span;
                            let alpha = 0.1 + 0.9 * (1.0 - age);
                            let color = base_color.linear_multiply(alpha);

                            let radius = 2.0 + (p.depth as f32 * 0.15);
                            let pts = PlotPoints::from_iter([[p.x, p.y]]);

                            plot_ui.points(
                                Points::new(pts)
                                    .color(color)
                                    .radius(radius),
                            );
                        }
                    });
            }
        });

        ctx.request_repaint();
    }
}

fn to_vec2(p: &CloudPoint) -> nalgebra::Vector2<f64> {
    nalgebra::Vector2::new(p.x as f64, p.y as f64)
}
