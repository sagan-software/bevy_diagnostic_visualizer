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

#[cfg(feature = "bevy_egui")]
mod egui;

use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics},
    prelude::*,
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
    time::Duration,
};

/// Diagnostics visualizer plugin
pub struct DiagnosticVisualizerPlugin {
    wait_duration: Duration,
    filter: DiagnosticIds,
}

#[derive(Clone)]
enum DiagnosticIds {
    Include(HashSet<DiagnosticId>),
    Exclude(HashSet<DiagnosticId>),
}

impl DiagnosticIds {
    fn should_include(&self, diagnostic_id: &DiagnosticId) -> bool {
        match self {
            Self::Include(ids) => ids.contains(diagnostic_id),
            Self::Exclude(ids) => !ids.contains(diagnostic_id),
        }
    }
}

impl Default for DiagnosticVisualizerPlugin {
    fn default() -> Self {
        Self {
            wait_duration: Duration::from_millis(20),
            filter: DiagnosticIds::Exclude(
                vec![bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_COUNT]
                    .into_iter()
                    .collect(),
            ),
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
            filter @ DiagnosticIds::Exclude(_) => {
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
            filter @ DiagnosticIds::Include(_) => {
                let mut hash_set = HashSet::new();
                hash_set.insert(diagnostic_id);
                *filter = DiagnosticIds::Exclude(hash_set);
            }
        };
        self
    }
}

struct DiagnosticVisualizerState {
    timer: Timer,
    filter: DiagnosticIds,
    default_formatter: Arc<dyn Formatter>,
    available_formatters: Vec<AvailableFormatter>,
    diagnostic_states: HashMap<DiagnosticId, DiagnosticState>,
}

fn find_formatter(
    available_formatters: &[AvailableFormatter],
    default_formatter: &Arc<dyn Formatter>,
    diagnostic: &Diagnostic,
) -> Arc<dyn Formatter> {
    available_formatters
        .iter()
        .find(|f| f.is_match(diagnostic))
        .map_or_else(|| default_formatter.clone(), |f| f.formatter.clone())
}

fn new_state(
    available_formatters: &[AvailableFormatter],
    default_formatter: &Arc<dyn Formatter>,
    diagnostic: &Diagnostic,
) -> DiagnosticState {
    DiagnosticState {
        name: diagnostic.name.clone(),
        formatter: find_formatter(available_formatters, default_formatter, diagnostic),
        measurements: VecDeque::default(),
    }
}

struct DiagnosticState {
    name: Cow<'static, str>,
    formatter: Arc<dyn Formatter>,
    measurements: VecDeque<f64>,
}

trait Formatter: (Fn(f64) -> String) + Send + Sync {}
impl<T> Formatter for T where T: (Fn(f64) -> String) + Send + Sync {}
trait Matcher: (Fn(&Diagnostic) -> bool) + Send + Sync {}
impl<T> Matcher for T where T: (Fn(&Diagnostic) -> bool) + Send + Sync {}

fn default_formatter(value: f64) -> String {
    format!("{:.0}", value)
}

fn format_secs_as_ms(value: f64) -> String {
    format!("{:.1} ms", value * 1_000.0)
}

struct AvailableFormatter {
    matchers: Vec<Box<dyn Matcher>>,
    formatter: Arc<dyn Formatter>,
}

impl AvailableFormatter {
    fn is_match(&self, diagnostic: &Diagnostic) -> bool {
        self.matchers.iter().any(|m| (m)(diagnostic))
    }
}

impl Plugin for DiagnosticVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DiagnosticVisualizerState {
            timer: Timer::new(self.wait_duration, true),
            filter: self.filter.clone(),
            default_formatter: Arc::new(default_formatter),
            available_formatters: vec![AvailableFormatter {
                matchers: vec![Box::new(|d: &Diagnostic| {
                    d.id == bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME
                })],
                formatter: Arc::new(format_secs_as_ms),
            }],
            diagnostic_states: HashMap::default(),
        })
        .add_system_to_stage(
            CoreStage::PreUpdate,
            update_diagnostic_visualizer_state_system,
        );
        #[cfg(feature = "bevy_egui")]
        app.add_plugin(crate::egui::DiagnosticVisualizerEguiPlugin);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn update_diagnostic_visualizer_state_system(
    mut state: ResMut<'_, DiagnosticVisualizerState>,
    time: Res<'_, Time>,
    diagnostics: Res<'_, Diagnostics>,
) {
    let DiagnosticVisualizerState {
        diagnostic_states,
        available_formatters,
        default_formatter,
        filter,
        timer,
        ..
    } = state.as_mut();
    let is_tick_finished = timer.tick(time.delta()).finished();
    if is_tick_finished {
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_enabled)
            .filter(|diagnostic| filter.should_include(&diagnostic.id))
            .for_each(|diagnostic| {
                let state = diagnostic_states.entry(diagnostic.id).or_insert_with(|| {
                    new_state(available_formatters, default_formatter, diagnostic)
                });
                track_diagnostic(diagnostic, state);
            });
    }
}

fn track_diagnostic(diagnostic: &Diagnostic, state: &mut DiagnosticState) {
    if let Some(last) = diagnostic.average() {
        state.measurements.push_back(last);
        if state.measurements.len() > 100 {
            state.measurements.pop_front();
        }
        state.measurements.make_contiguous();
    }
}
