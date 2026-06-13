use crate::terrain::HeightCurve;
use bevy_egui::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

const CANVAS_HEIGHT: f32 = 200.0;
const MARGIN_L: f32 = 40.0;
const MARGIN_R: f32 = 10.0;
const MARGIN_T: f32 = 14.0;
const MARGIN_B: f32 = 26.0;

pub fn height_curve_editor(ui: &mut egui::Ui, curve: &mut HeightCurve) -> bool {
    let mut changed = false;

    let available = ui.available_width();
    let canvas_size = Vec2::new(available.max(120.0), CANVAS_HEIGHT);

    let (response, painter) = ui.allocate_painter(canvas_size, Sense::click_and_drag());
    let rect = response.rect;

    let plot_rect = Rect::from_min_max(
        Pos2::new(rect.left() + MARGIN_L, rect.top() + MARGIN_T),
        Pos2::new(rect.right() - MARGIN_R, rect.bottom() - MARGIN_B),
    );

    let point_radius = 5.0;
    let selected_point_radius = 7.0;
    let click_threshold = selected_point_radius + 4.0;

    let to_screen = |input: f32, output: f32| -> Pos2 {
        Pos2::new(
            plot_rect.left() + input * plot_rect.width(),
            plot_rect.bottom() - output * plot_rect.height(),
        )
    };

    // ---- Drawing (immutable borrow of curve) ----
    {
        let points = &curve.points;

        // Background
        painter.rect_filled(plot_rect, 0.0, Color32::from_gray(26));

        // Grid lines
        for i in 0..=4 {
            let t = i as f32 / 4.0;
            let x = plot_rect.left() + t * plot_rect.width();
            painter.line_segment(
                [Pos2::new(x, plot_rect.top()), Pos2::new(x, plot_rect.bottom())],
                Stroke::new(1.0, Color32::from_gray(46)),
            );
            let y = plot_rect.bottom() - t * plot_rect.height();
            painter.line_segment(
                [Pos2::new(plot_rect.left(), y), Pos2::new(plot_rect.right(), y)],
                Stroke::new(1.0, Color32::from_gray(46)),
            );
        }

        // Axis border
        painter.rect_stroke(
            plot_rect,
            0.0,
            Stroke::new(1.0, Color32::from_gray(90)),
            egui::StrokeKind::Outside,
        );

        // Axis labels
        let font_id = egui::FontId::proportional(11.0);
        let label_color = Color32::from_gray(150);

        for &(t, text) in &[(0.0, "0"), (0.5, "0.5"), (1.0, "1")] {
            let x = plot_rect.left() + t * plot_rect.width();
            painter.text(
                Pos2::new(x, rect.bottom() - 3.0),
                egui::Align2::CENTER_TOP,
                text,
                font_id.clone(),
                label_color,
            );
            let y = plot_rect.bottom() - t * plot_rect.height();
            painter.text(
                Pos2::new(rect.left() + 5.0, y),
                egui::Align2::LEFT_CENTER,
                text,
                font_id.clone(),
                label_color,
            );
        }

        // Curve line
        if points.len() >= 2 {
            let mut prev = to_screen(points[0].input, points[0].output);
            for point in points.iter().skip(1) {
                let curr = to_screen(point.input, point.output);
                painter.line_segment([prev, curr], Stroke::new(2.5, Color32::from_rgb(76, 175, 80)));
                prev = curr;
            }
        }

        // Control points
        let state_id = ui.id().with("curve_state");
        let selected_index: usize = ui.data(|d| d.get_temp::<usize>(state_id).unwrap_or(usize::MAX));

        for (i, point) in points.iter().enumerate() {
            let pos = to_screen(point.input, point.output);
            let is_selected = i == selected_index;
            let is_endpoint = i == 0 || i == points.len() - 1;
            let r = if is_selected {
                selected_point_radius
            } else {
                point_radius
            };

            let fill = if is_selected {
                Color32::from_rgb(255, 213, 79)
            } else if is_endpoint {
                Color32::from_rgb(100, 181, 246)
            } else {
                Color32::WHITE
            };

            painter.circle_filled(pos, r, fill);
            painter.circle_stroke(pos, r, Stroke::new(1.0, Color32::from_gray(170)));
        }

        // Axis titles
        painter.text(
            Pos2::new(plot_rect.center().x, rect.bottom() - 1.0),
            egui::Align2::CENTER_BOTTOM,
            "Input",
            egui::FontId::proportional(11.0),
            Color32::from_gray(120),
        );
    }

    // ---- Interaction (mutable borrow of curve) ----
    let state_id = ui.id().with("curve_state");
    let mut selected_index: usize = ui.data(|d| d.get_temp::<usize>(state_id).unwrap_or(usize::MAX));

    // Click / drag-start: select or deselect a point
    if response.drag_started() || response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let nearest = curve
                .points
                .iter()
                .enumerate()
                .map(|(i, p)| (i, to_screen(p.input, p.output)))
                .filter(|(_, screen_pos)| screen_pos.distance(pos) <= click_threshold)
                .min_by(|(_, a), (_, b)| a.distance(pos).total_cmp(&b.distance(pos)));

            match nearest {
                Some((idx, _)) => selected_index = idx,
                None => selected_index = usize::MAX,
            }
        }
        ui.data_mut(|d| d.insert_temp(state_id, selected_index));
    }

    // Drag: move selected point
    if response.dragged() && selected_index < curve.points.len() {
        let delta = response.drag_delta();
        let input_per_pixel = 1.0 / plot_rect.width();
        let output_per_pixel = 1.0 / plot_rect.height();

        let current_input = curve.points[selected_index].input;
        let current_output = curve.points[selected_index].output;

        if selected_index > 0 && selected_index < curve.points.len() - 1 {
            changed |=
                curve.set_point_input(selected_index, current_input + delta.x * input_per_pixel);
        }
        changed |=
            curve.set_point_output(selected_index, current_output - delta.y * output_per_pixel);
    }

    // ---- Bottom controls ----
    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Add Point").clicked() {
            changed |= curve.add_point();
        }

        let can_remove = selected_index < curve.points.len()
            && selected_index > 0
            && selected_index < curve.points.len() - 1;
        if ui
            .add_enabled(can_remove, egui::Button::new("Remove Point"))
            .clicked()
        {
            changed |= curve.remove_point(selected_index);
            selected_index = usize::MAX;
            ui.data_mut(|d| d.insert_temp(state_id, selected_index));
        }

        if ui.button("Reset").clicked() {
            changed |= curve.reset();
            selected_index = usize::MAX;
            ui.data_mut(|d| d.insert_temp(state_id, selected_index));
        }
    });

    // Selected point details
    if selected_index < curve.points.len() {
        ui.horizontal(|ui| {
            ui.label("X:");
            changed |= ui
                .add(egui::DragValue::new(&mut curve.points[selected_index].input).speed(0.005).range(0.0..=1.0))
                .changed();
            ui.label("Y:");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut curve.points[selected_index].output)
                        .speed(0.005)
                        .range(0.0..=1.0),
                )
                .changed();
        });

        if selected_index == 0 {
            ui.colored_label(Color32::from_gray(140), "Start point — input fixed at 0");
        } else if selected_index == curve.points.len() - 1 {
            ui.colored_label(Color32::from_gray(140), "End point — input fixed at 1");
        }
    }

    changed
}
