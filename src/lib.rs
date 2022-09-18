#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::wildcard_imports,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    unused,
    missing_docs
)]
#![allow(
    clippy::missing_const_for_fn,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::multiple_crate_versions
)]

//! This crate provides visualizations for Bevy game engine diagnostics.
//!
//! ## Usage
//!
//! Here's a minimal usage example:
//!
//! ```
#![doc = include_str!("../examples/minimal.rs")]
//! ```

use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics},
    prelude::*,
};
use bevy_egui::{
    egui::{
        epaint::{PathShape, RectShape},
        pos2, remap, vec2, CollapsingHeader, Color32, Rect, Rgba, Rounding, Sense, Shape, Stroke,
        TextStyle, Ui, Vec2, WidgetText, Window,
    },
    EguiContext, EguiPlugin,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Duration,
};

/// Diagnostics visualizer plugin
pub struct DiagnosticVisualizerPlugin {
    wait_duration: Duration,
    filter: DiagnosticIds,
    style: Style,
}

#[derive(Clone)]
struct Style {
    text_color: Color32,
    rectangle_stroke: Stroke,
    line_stroke: Stroke,
    width: f32,
    height: f32,
}

#[derive(Clone)]
enum DiagnosticIds {
    All,
    Include(HashSet<DiagnosticId>),
    Exclude(HashSet<DiagnosticId>),
}

impl DiagnosticIds {
    fn should_include(&self, diagnostic_id: &DiagnosticId) -> bool {
        match self {
            Self::All => true,
            Self::Include(ids) => ids.contains(diagnostic_id),
            Self::Exclude(ids) => !ids.contains(diagnostic_id),
        }
    }
}

impl Default for DiagnosticVisualizerPlugin {
    fn default() -> Self {
        Self {
            wait_duration: Duration::from_millis(20),
            filter: DiagnosticIds::All,
            style: Style::default(),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            text_color: Color32::WHITE,
            rectangle_stroke: Stroke::new(1., Color32::WHITE),
            line_stroke: Stroke::new(1., Color32::WHITE),
            width: 200.,
            height: 100.,
        }
    }
}

impl DiagnosticVisualizerPlugin {
    /// How often to update measurements
    #[must_use]
    pub fn wait_duration(mut self, wait_duration: Duration) -> Self {
        self.wait_duration = wait_duration;
        self
    }

    /// Include a specific diagnostic ID.
    #[must_use]
    pub fn include(mut self, diagnostic_id: DiagnosticId) -> Self {
        match &mut self.filter {
            DiagnosticIds::Include(hash_set) => {
                hash_set.insert(diagnostic_id);
            }
            filter => {
                let mut hash_set = HashSet::new();
                hash_set.insert(diagnostic_id);
                *filter = DiagnosticIds::Include(hash_set);
            }
        };
        self
    }

    /// Exclude a specific diagnostic ID.
    #[must_use]
    pub fn exclude(mut self, diagnostic_id: DiagnosticId) -> Self {
        match &mut self.filter {
            DiagnosticIds::Exclude(hash_set) => {
                hash_set.insert(diagnostic_id);
            }
            filter => {
                let mut hash_set = HashSet::new();
                hash_set.insert(diagnostic_id);
                *filter = DiagnosticIds::Exclude(hash_set);
            }
        };
        self
    }
}

struct State {
    timer: Timer,
    filter: DiagnosticIds,
    measurements: HashMap<DiagnosticId, VecDeque<f64>>,
    is_open: bool,
    style: Style,
}

impl Plugin for DiagnosticVisualizerPlugin {
    fn build(&self, app: &mut App) {
        if !app.world.contains_resource::<EguiContext>() {
            app.add_plugin(EguiPlugin);
        }

        app.insert_resource(State {
            timer: Timer::new(self.wait_duration, true),
            filter: self.filter.clone(),
            measurements: HashMap::default(),
            is_open: true,
            style: self.style.clone(),
        })
        .add_system_to_stage(CoreStage::PostUpdate, plot_diagnostics_system);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn plot_diagnostics_system(
    mut state: ResMut<'_, State>,
    time: Res<'_, Time>,
    diagnostics: Res<'_, Diagnostics>,
    mut egui_context: ResMut<'_, EguiContext>,
) {
    let State {
        is_open,
        measurements,
        filter,
        style,
        timer,
        ..
    } = state.as_mut();

    if !*is_open {
        return;
    }

    let is_tick_finished = timer.tick(time.delta()).finished();

    Window::new("Diagnostics")
        .open(is_open)
        .default_width(250.0)
        .default_height(diagnostics.iter().count() as f32 * 170.0)
        .vscroll(true)
        .show(egui_context.ctx_mut(), |ui| {
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.is_enabled)
                .filter(|diagnostic| filter.should_include(&diagnostic.id))
                .for_each(|diagnostic| {
                    if is_tick_finished {
                        track_diagnostic(measurements, diagnostic);
                    }
                    plot_diagnostic(measurements, diagnostic, ui, style);
                });
        });
}

fn track_diagnostic(
    measurements: &mut HashMap<DiagnosticId, VecDeque<f64>>,
    diagnostic: &Diagnostic,
) {
    let measurements = measurements.entry(diagnostic.id).or_default();
    if let Some(last) = diagnostic.average() {
        measurements.push_back(last);
        if measurements.len() > 100 {
            measurements.pop_front();
        }
    }
    measurements.make_contiguous();
}

fn plot_diagnostic(
    measurements: &mut HashMap<DiagnosticId, VecDeque<f64>>,
    diagnostic: &Diagnostic,
    ui: &mut Ui,
    style: &Style,
) {
    let values = measurements.entry(diagnostic.id).or_default().as_slices().0;
    CollapsingHeader::new(diagnostic.name.as_ref())
        .default_open(true)
        .show(ui, |ui| show_graph(ui, style, values));
}

fn show_graph(ui: &mut Ui, style: &Style, values: &[f64]) {
    if values.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        let last_value = values.last().unwrap();

        let min = 0.0;
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        let spacing_x = ui.spacing().item_spacing.x;

        let last_text: WidgetText = format!("{:.2}", last_value).into();
        let galley = last_text.into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
        let (outer_rect, _) = ui.allocate_exact_size(
            Vec2::new(style.width + galley.size().x + spacing_x, style.height),
            Sense::hover(),
        );
        let rect = Rect::from_min_size(outer_rect.left_top(), vec2(style.width, style.height));
        let text_pos = rect.right_center() + vec2(spacing_x / 2.0, -galley.size().y / 2.);
        galley.paint_with_fallback_color(
            &ui.painter().with_clip_rect(outer_rect),
            text_pos,
            style.text_color,
        );

        let body = Shape::Rect(RectShape {
            rect,
            rounding: Rounding::none(),
            fill: Rgba::TRANSPARENT.into(),
            stroke: style.rectangle_stroke,
        });
        ui.painter().add(body);
        let init_point = rect.left_bottom();

        let size = values.len();
        let points = values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let x = remap(i as f32, 0.0..=size as f32, 0.0..=style.width);
                let y = remap(
                    (*value) as f32,
                    (min as f32)..=(max as f32),
                    0.0..=style.height,
                );

                pos2(x + init_point.x, init_point.y - y)
            })
            .collect();

        let path = PathShape::line(points, style.line_stroke);
        ui.painter().add(path);

        {
            let text: WidgetText = format!("{:.0}", max).into();
            let galley = text.into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
            let text_pos =
                rect.left_top() + Vec2::new(0.0, galley.size().y / 2.) + vec2(spacing_x, 0.0);
            galley.paint_with_fallback_color(
                &ui.painter().with_clip_rect(rect),
                text_pos,
                style.text_color,
            );
        }
        {
            let text: WidgetText = format!("{:.0}", min).into();
            let galley = text.into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
            let text_pos =
                rect.left_bottom() - Vec2::new(0.0, galley.size().y * 1.5) + vec2(spacing_x, 0.0);
            galley.paint_with_fallback_color(
                &ui.painter().with_clip_rect(rect),
                text_pos,
                style.text_color,
            );
        }
    });
}
