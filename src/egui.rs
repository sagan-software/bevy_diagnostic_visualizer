use crate::{DiagnosticState, DiagnosticVisualizerState};
use bevy::prelude::*;
use bevy_egui::{
    egui::{
        epaint::{PathShape, RectShape},
        pos2, remap, vec2, CollapsingHeader, Color32, Rect, Rgba, Rounding, Sense, Shape, Stroke,
        TextStyle, Ui, Vec2, WidgetText, Window,
    },
    EguiContext, EguiPlugin,
};

pub struct DiagnosticVisualizerEguiPlugin;

impl Plugin for DiagnosticVisualizerEguiPlugin {
    fn build(&self, app: &mut App) {
        if !app.world.contains_resource::<EguiContext>() {
            app.add_plugin(EguiPlugin);
        }
        app.insert_resource(Style::default())
            .insert_resource(IsOpenState(true))
            .add_system_to_stage(CoreStage::PostUpdate, plot_diagnostics_system);
    }
}

struct Style {
    text_color: Color32,
    rectangle_stroke: Stroke,
    line_stroke: Stroke,
    width: f32,
    height: f32,
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

struct IsOpenState(bool);

#[allow(clippy::needless_pass_by_value)]
fn plot_diagnostics_system(
    state: Res<'_, DiagnosticVisualizerState>,
    style: Res<'_, Style>,
    mut is_open_state: ResMut<'_, IsOpenState>,
    mut egui_context: ResMut<'_, EguiContext>,
) {
    Window::new("Diagnostics")
        .open(&mut is_open_state.0)
        .vscroll(true)
        .show(egui_context.ctx_mut(), |ui| {
            for diagnostic_state in state.diagnostic_states.values() {
                plot_diagnostic(diagnostic_state, ui, &style);
            }
        });
}

fn plot_diagnostic(diagnostic_state: &DiagnosticState, ui: &mut Ui, style: &Style) {
    CollapsingHeader::new(diagnostic_state.name.as_ref())
        .default_open(true)
        .show(ui, |ui| show_graph(ui, style, diagnostic_state));
}

fn show_graph(ui: &mut Ui, style: &Style, state: &DiagnosticState) {
    let DiagnosticState {
        formatter,
        measurements,
        ..
    } = state;

    let values = measurements.as_slices().0;

    if values.is_empty() {
        return;
    }

    ui.vertical(|ui| {
        let last_value = values.last().unwrap();

        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        let spacing_x = ui.spacing().item_spacing.x;

        let last_text: WidgetText = formatter(*last_value).into();
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
                let y = remap((*value) as f32, 0.0..=(max as f32), 0.0..=style.height);

                pos2(x + init_point.x, init_point.y - y)
            })
            .collect();

        let path = PathShape::line(points, style.line_stroke);
        ui.painter().add(path);

        // Max value
        {
            let text: WidgetText = format!("max: {}", formatter(max)).into();
            let galley = text.into_galley(ui, Some(false), f32::INFINITY, TextStyle::Button);
            let text_pos =
                rect.left_top() + Vec2::new(0.0, galley.size().y / 2.) + vec2(spacing_x, 0.0);
            galley.paint_with_fallback_color(
                &ui.painter().with_clip_rect(rect),
                text_pos,
                style.text_color,
            );
        }

        // Min value
        {
            let text: WidgetText = format!("min: {}", formatter(min)).into();
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
