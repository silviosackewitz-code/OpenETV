//! Read-only, orthographic projection of the actual map points.
use crate::{look, model::Table};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Camera {
    yaw: f32,
    pitch: f32,
    zoom: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: -0.7,
            pitch: 0.65,
            zoom: 1.,
        }
    }
}
impl Camera {
    fn project(self, p: [f32; 3], rect: Rect) -> (Pos2, f32) {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let depth = p[0] * sy + p[1] * cy;
        let scale = (rect.width() / 4.2).min(rect.height() / 3.3) * self.zoom;
        let center = rect.center() + Vec2::new(0., 0.8 * cp * scale);
        (
            center + Vec2::new(p[0] * cy - p[1] * sy, depth * sp - p[2] * cp) * scale,
            depth * cp + p[2] * sp,
        )
    }
}
fn fraction(value: f64, low: f64, high: f64) -> f32 {
    if high == low {
        return 0.5;
    }
    // Scaling first keeps finite, extreme imported values from overflowing the span.
    let scale = low.abs().max(high.abs()).max(1.);
    ((value / scale - low / scale) / (high / scale - low / scale)) as f32
}
fn coordinates(table: &Table, low: f64, high: f64) -> Vec<[f32; 3]> {
    table
        .rpm
        .iter()
        .enumerate()
        .flat_map(|(i, &rpm)| {
            table.axis.iter().enumerate().map(move |(j, &axis)| {
                [
                    2. * fraction(rpm, table.rpm[0], *table.rpm.last().unwrap()) - 1.,
                    2. * fraction(axis, table.axis[0], *table.axis.last().unwrap()) - 1.,
                    1.6 * fraction(table.values[i][j], low, high),
                ]
            })
        })
        .collect()
}

pub fn show(ui: &mut egui::Ui, id: &str, table: &Table, low: f64, high: f64, unit: &str) {
    if table.validate().is_err() {
        return;
    }
    let key = ui.make_persistent_id((id, "3d camera"));
    let mut camera = ui
        .ctx()
        .data_mut(|d| d.get_temp::<Camera>(key))
        .unwrap_or_default();
    ui.horizontal_wrapped(|ui| {
        if ui.button("Reset view").clicked() {
            camera = Camera::default();
        }
        ui.add(
            egui::Slider::new(
                &mut camera.yaw,
                -std::f32::consts::PI..=std::f32::consts::PI,
            )
            .text("Rotation")
            .show_value(false),
        );
        ui.add(
            egui::Slider::new(&mut camera.pitch, 0.15..=1.4)
                .text("Tilt")
                .show_value(false),
        );
        ui.add(
            egui::Slider::new(&mut camera.zoom, 0.6..=1.8)
                .text("Zoom")
                .suffix("×"),
        );
    });
    ui.small("Drag to rotate · scroll to zoom · hover a point for its value · edit in Table");
    let height = (ui.ctx().content_rect().height() - 330.).clamp(300., 560.);
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::drag());
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Other, true, "3D map surface"));
    if response.dragged() {
        let delta = ui.input(|i| i.pointer.delta());
        camera.yaw = (camera.yaw + delta.x * 0.01 + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        camera.pitch = (camera.pitch - delta.y * 0.01).clamp(0.15, 1.4);
    }
    if response.hovered() {
        let scroll = ui.input_mut(|i| {
            let scroll = i.smooth_scroll_delta.y;
            i.smooth_scroll_delta = Vec2::ZERO;
            scroll
        });
        camera.zoom = (camera.zoom * (scroll * 0.003).exp()).clamp(0.6, 1.8);
    }
    ui.ctx().data_mut(|d| d.insert_temp(key, camera));
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 6., ui.visuals().extreme_bg_color);
    let ink = ui.visuals().text_color();
    let grid = Stroke::new(1., ui.visuals().widgets.noninteractive.bg_stroke.color);
    let project = |p| camera.project(p, rect).0;
    for n in 0..=4 {
        let t = n as f32 * 0.5 - 1.;
        painter.line_segment([project([t, -1., 0.]), project([t, 1., 0.])], grid);
        painter.line_segment([project([-1., t, 0.]), project([1., t, 0.])], grid);
    }
    let points = coordinates(table, low, high);
    let projected: Vec<_> = points.iter().map(|&p| camera.project(p, rect)).collect();
    let colors: Vec<_> = table
        .values
        .iter()
        .flatten()
        .map(|&v| look::map_color(v, low, high))
        .collect();
    let cols = table.axis.len();
    let mut triangles = Vec::new();
    for i in 0..table.rpm.len() - 1 {
        for j in 0..cols - 1 {
            let a = i * cols + j;
            for indices in [[a, a + cols, a + cols + 1], [a, a + cols + 1, a + 1]] {
                let depth = indices.iter().map(|&k| projected[k].1).sum::<f32>() / 3.;
                triangles.push((indices, depth));
            }
        }
    }
    triangles.sort_by(|a, b| a.1.total_cmp(&b.1));
    for (indices, _) in triangles {
        let mut mesh = egui::Mesh::default();
        for k in indices {
            mesh.colored_vertex(projected[k].0, colors[k]);
        }
        mesh.add_triangle(0, 1, 2);
        painter.add(egui::Shape::mesh(mesh));
        // The diagonal is only a rendering seam; draw only original grid edges.
        for edge in [
            [indices[0], indices[1]],
            [indices[1], indices[2]],
            [indices[2], indices[0]],
        ] {
            if edge[0].abs_diff(edge[1]) == 1 || edge[0].abs_diff(edge[1]) == cols {
                painter.line_segment(
                    edge.map(|k| projected[k].0),
                    Stroke::new(0.8, Color32::from_black_alpha(100)),
                );
            }
        }
    }
    if table.rpm.len() == 1 || cols == 1 {
        ui.small("A surface needs at least two RPM and two position points. This map is shown as a line or point.");
        for pair in projected.windows(2) {
            painter.line_segment([pair[0].0, pair[1].0], Stroke::new(2., ink));
        }
        for (i, p) in projected.iter().enumerate() {
            painter.circle_filled(p.0, 3., colors[i]);
        }
    }
    let x_front = if camera.yaw.sin() >= 0. { 1. } else { -1. };
    let y_front = if camera.yaw.cos() >= 0. { 1. } else { -1. };
    let z_corner = [[-1., -1., 0.], [-1., 1., 0.], [1., -1., 0.], [1., 1., 0.]]
        .into_iter()
        .min_by(|a, b| project(*a).x.total_cmp(&project(*b).x))
        .unwrap();
    let position_label = if id == "engine" {
        "Throttle (%)"
    } else {
        "Pedal (%)"
    };
    for (a, b, title, from, to) in [
        (
            [-1., y_front, 0.],
            [1., y_front, 0.],
            "RPM",
            table.rpm[0],
            *table.rpm.last().unwrap(),
        ),
        (
            [x_front, -1., 0.],
            [x_front, 1., 0.],
            position_label,
            table.axis[0],
            *table.axis.last().unwrap(),
        ),
        (z_corner, [z_corner[0], z_corner[1], 1.6], unit, low, high),
    ] {
        painter.line_segment([project(a), project(b)], Stroke::new(1.2, ink));
        let vertical = a[2] != b[2];
        let mid = project(std::array::from_fn(|k| (a[k] + b[k]) * 0.5));
        let offset = if vertical {
            Vec2::new(-40., 0.)
        } else {
            (mid - project([0., 0., 0.])).normalized() * 42.
        };
        painter.text(
            if vertical {
                project(b) + Vec2::new(0., -24.)
            } else {
                mid + offset
            },
            Align2::CENTER_CENTER,
            title,
            FontId::proportional(12.),
            ink,
        );
        for n in 0..=4 {
            if from == to && n != 2 {
                continue;
            }
            let f = n as f32 / 4.;
            let p = project(std::array::from_fn(|k| a[k] + (b[k] - a[k]) * f));
            painter.circle_filled(p, 2., ink);
            let label = from * (1. - f64::from(f)) + to * f64::from(f);
            let offset = if vertical {
                Vec2::new(-8., 0.)
            } else {
                (mid - project([0., 0., 0.])).normalized() * 16.
            };
            painter.text(
                p + offset,
                if vertical {
                    Align2::RIGHT_CENTER
                } else {
                    Align2::CENTER_CENTER
                },
                format!("{label:.0}"),
                FontId::proportional(10.),
                ink,
            );
        }
    }
    if let Some(pointer) = response.hover_pos()
        && let Some((k, p)) = projected
            .iter()
            .enumerate()
            .filter(|(_, p)| p.0.distance(pointer) < 12.)
            .min_by(|(_, a), (_, b)| {
                a.0.distance_sq(pointer)
                    .total_cmp(&b.0.distance_sq(pointer))
            })
    {
        painter.circle_stroke(p.0, 5., Stroke::new(2., ink));
        response.on_hover_text(format!(
            "{} RPM · {} {}\n{} {unit}",
            table.rpm[k / cols],
            table.axis[k % cols],
            position_label,
            table.values[k / cols][k % cols]
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geometry_preserves_breakpoint_spacing_and_signed_values() {
        let t = Table::new(
            vec![1000., 2000., 5000.],
            vec![0., 20., 100.],
            vec![vec![-10., 0., 30.]; 3],
        )
        .unwrap();
        let p = coordinates(&t, -10., 30.);
        assert_eq!(p[0], [-1., -1., 0.]);
        assert_eq!(p[3][0], -0.5);
        assert!((p[1][1] + 0.6).abs() < 1e-6);
        assert_eq!(p[8], [1., 1., 1.6]);
        let single = Table::new(vec![3000.], vec![50.], vec![vec![7.]]).unwrap();
        assert_eq!(coordinates(&single, 7., 7.), vec![[0., 0., 0.8]]);
        assert_eq!(fraction(0., -f64::MAX, f64::MAX), 0.5);
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800., 500.));
        let a = Camera::default().project(p[0], rect);
        let b = Camera {
            yaw: 1.,
            ..Camera::default()
        }
        .project(p[0], rect);
        assert!(a.0.is_finite() && b.0.is_finite());
        assert_ne!(a, b);
    }
}
