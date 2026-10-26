use crate::{
    formats,
    model::{Status, Table, lookup},
};
use eframe::egui::{self, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

/// Header coordinates include the top-left label at (0, 0), matching clipboard blocks.
pub fn editor(
    ui: &mut egui::Ui,
    id: &str,
    table: &mut Table,
    status: Option<&[Vec<Status>]>,
    read_only: bool,
) -> Result<bool, String> {
    table.validate()?;
    let key = ui.make_persistent_id(id);
    let active = egui::Id::new("openetv_active_grid");
    let mut selected = ui
        .ctx()
        .data_mut(|d| d.get_temp::<(usize, usize)>(key))
        .unwrap_or((1, 1));
    selected.0 = selected.0.min(table.rpm.len());
    selected.1 = selected.1.min(table.axis.len());
    let mut draft = table.clone();
    let mut changed = false;
    let axes_editable = !read_only && status.is_none();
    let tools_id = key.with("tools");
    let (mut scope, mut operation, mut amount) = ui
        .ctx()
        .data_mut(|d| d.get_temp::<(usize, usize, f64)>(tools_id))
        .unwrap_or((0, 0, 0.0));
    let mut tools_error = None;
    ui.collapsing("Table tools",|ui|{
        ui.horizontal_wrapped(|ui| {
            if ui.button("Copy table").clicked() {
                let mut text=String::from("RPM\\Grip[%]");
                for x in &table.axis {text.push_str(&format!("\t{x}"));}
                for (rpm,row) in table.rpm.iter().zip(&table.values) {text.push_str(&format!("\n{rpm}"));for x in row{text.push_str(&format!("\t{x}"));}}
                ui.ctx().copy_text(text);
            }
            if axes_editable {
                if ui.add_enabled((table.rpm.len()+1)*table.axis.len()<=20_000,egui::Button::new("+ RPM row")).clicked(){
                    let step=if table.rpm.len()>1{table.rpm[table.rpm.len()-1]-table.rpm[table.rpm.len()-2]}else{1000.0};
                    draft.rpm.push(table.rpm.last().unwrap()+step);draft.values.push(table.values.last().unwrap().clone());changed=true;
                }
                if ui.add_enabled(table.rpm.len()*(table.axis.len()+1)<=20_000,egui::Button::new("+ Column")).clicked(){match insert_column(table){Ok(t)=>{draft=t;changed=true},Err(e)=>tools_error=Some(e)}}
                if ui.add_enabled(table.rpm.len()>1&&selected.0>0,egui::Button::new("Remove selected row")).clicked(){draft.rpm.remove(selected.0-1);draft.values.remove(selected.0-1);changed=true;}
                if ui.add_enabled(table.axis.len()>1&&selected.1>0,egui::Button::new("Remove selected column")).clicked(){draft.axis.remove(selected.1-1);for row in &mut draft.values{row.remove(selected.1-1);}changed=true;}
            }
        });
        if !read_only {
            ui.horizontal_wrapped(|ui|{
                ui.label("Apply to");
                egui::ComboBox::from_id_salt((id,"scope")).selected_text(["Selected cell","Selected row","Selected column"][scope]).show_ui(ui,|ui|{for(i,label)in ["Selected cell","Selected row","Selected column"].iter().enumerate(){ui.selectable_value(&mut scope,i,*label);}});
                egui::ComboBox::from_id_salt((id,"operation")).selected_text(["Fill","Add offset","Multiply","Interpolate endpoints"][operation]).show_ui(ui,|ui|{for(i,label)in ["Fill","Add offset","Multiply","Interpolate endpoints"].iter().enumerate(){ui.selectable_value(&mut operation,i,*label);}});
                if operation!=3{ui.label("Value");ui.add(egui::DragValue::new(&mut amount).speed(0.1));}
                if ui.button("Apply").clicked(){match bulk(table,selected,scope,operation,amount){Ok(t)=>{draft=t;changed=true},Err(e)=>tools_error=Some(e)}}
            });
            ui.weak("Click a value or header to select its row/column. Paste tab-separated cells at the selection. Interpolation keeps both endpoints.");
        }
    });
    ui.ctx()
        .data_mut(|d| d.insert_temp(tools_id, (scope, operation, amount)));
    if let Some(error) = tools_error {
        return Err(error);
    }
    // Structural changes are committed before drawing so indices remain consistent.
    if changed {
        *table = Table::new(draft.rpm, draft.axis, draft.values)?;
        return Ok(true);
    }
    egui::ScrollArea::both().id_salt((id,"scroll")).max_height(340.0).show(ui,|ui|{
        egui::Grid::new((id,"cells")).striped(true).spacing([5.0,4.0]).show(ui,|ui|{
            let corner=ui.selectable_label(selected==(0,0),"RPM / %"); if corner.clicked(){selected=(0,0);corner.request_focus();ui.ctx().data_mut(|d|d.insert_temp(active,(key,corner.id)));}
            for j in 0..draft.axis.len(){cell(ui,&mut draft.axis[j],(0,j+1),&mut selected,key,active,!axes_editable,&format!("Column {} breakpoint percent",j+1),&mut changed);}
            ui.end_row();
            for i in 0..draft.rpm.len(){
                cell(ui,&mut draft.rpm[i],(i+1,0),&mut selected,key,active,!axes_editable,&format!("Row {} RPM",i+1),&mut changed);
                for j in 0..draft.axis.len(){
                    ui.vertical(|ui|{
                        cell(ui,&mut draft.values[i][j],(i+1,j+1),&mut selected,key,active,read_only,&format!("Value at {} RPM, {} percent",table.rpm[i],table.axis[j]),&mut changed);
                        if let Some(s)=status.and_then(|rows|rows.get(i)).and_then(|row|row.get(j)) {let (symbol,tip)=match s{Status::Ok=>("·","Torque request reached"),Status::Saturated=>("▲","Torque saturated"),Status::BelowMin=>("▽","Below closed-throttle torque"),Status::NonMonotonic=>("!","Engine torque falls below the request at a larger opening")};ui.small(symbol).on_hover_text(tip);}
                    });
                }
                ui.end_row();
            }
        });
    });
    ui.ctx().data_mut(|d| d.insert_temp(key, selected));
    let owner = ui
        .ctx()
        .data_mut(|d| d.get_temp::<(egui::Id, egui::Id)>(active));
    let owns_paste = owner
        .is_some_and(|(grid, cell)| grid == key && ui.ctx().memory(|m| m.focused()) == Some(cell));
    if !read_only && owns_paste {
        let paste = ui.input_mut(|input| {
            let pos = input
                .events
                .iter()
                .position(|e| matches!(e, egui::Event::Paste(_)));
            pos.and_then(|i| match input.events.remove(i) {
                egui::Event::Paste(s) => Some(s),
                _ => None,
            })
        });
        if let Some(text) = paste {
            if !axes_editable && (selected.0 == 0 || selected.1 == 0) {
                return Err("Result breakpoints are controlled by output settings.".into());
            }
            draft = formats::paste_block(table, selected.0, selected.1, &text)?;
            changed = true;
        }
    }
    if changed {
        *table = Table::new(draft.rpm, draft.axis, draft.values)?;
    }
    Ok(changed)
}
#[allow(clippy::too_many_arguments)]
fn cell(
    ui: &mut egui::Ui,
    value: &mut f64,
    position: (usize, usize),
    selected: &mut (usize, usize),
    key: egui::Id,
    active: egui::Id,
    read_only: bool,
    label: &str,
    changed: &mut bool,
) {
    let response = if read_only {
        ui.selectable_label(*selected == position, format!("{value:.2}"))
    } else {
        ui.add(egui::DragValue::new(value).speed(0.1).max_decimals(4))
    };
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::DragValue, !read_only, label));
    let response = response.on_hover_text(label);
    if response.clicked() || response.gained_focus() {
        *selected = position;
        response.request_focus();
        ui.ctx()
            .data_mut(|d| d.insert_temp(active, (key, response.id)));
    }
    if response.changed() {
        *changed = true;
    }
}

pub fn chart(
    ui: &mut egui::Ui,
    table: &Table,
    row: &mut usize,
    overlay: Option<&Table>,
    unit: &str,
) {
    if table.validate().is_err() {
        return;
    }
    *row = (*row).min(table.rpm.len() - 1);
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt((ui.id(), "rpm", unit))
            .selected_text(format!("{} RPM", table.rpm[*row]))
            .show_ui(ui, |ui| {
                for (i, rpm) in table.rpm.iter().enumerate() {
                    ui.selectable_value(row, i, format!("{rpm} RPM"));
                }
            });
    });
    let extra = overlay.filter(|t| t.validate().is_ok()).map(|t| {
        table
            .axis
            .iter()
            .map(|&x| lookup(t, table.rpm[*row], x))
            .collect::<Vec<_>>()
    });
    let primary = &table.values[*row];
    let low = primary
        .iter()
        .chain(extra.iter().flatten())
        .copied()
        .fold(0.0, f64::min);
    let high = primary
        .iter()
        .chain(extra.iter().flatten())
        .copied()
        .fold(0.0, f64::max);
    let span = (high - low).max(1.0);
    let y_min = low - span * 0.05;
    let y_max = high + span * 0.05;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width().max(150.0), 220.0),
        Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 5.0, Color32::from_rgb(15, 22, 32));
    let plot = Rect::from_min_max(
        rect.min + Vec2::new(52.0, 15.0),
        rect.max - Vec2::new(16.0, 32.0),
    );
    let x_min = table.axis[0];
    let x_max = *table.axis.last().unwrap();
    let x_span = (x_max - x_min).max(1.0);
    let point = |x: f64, y: f64| {
        Pos2::new(
            plot.left() + ((x - x_min) / x_span) as f32 * plot.width(),
            plot.bottom() - ((y - y_min) / (y_max - y_min)) as f32 * plot.height(),
        )
    };
    for i in 0..=4 {
        let f = i as f64 / 4.0;
        let y = y_min + f * (y_max - y_min);
        let p = point(x_min, y);
        painter.line_segment(
            [Pos2::new(plot.left(), p.y), Pos2::new(plot.right(), p.y)],
            Stroke::new(1.0, Color32::from_gray(45)),
        );
        painter.text(
            Pos2::new(plot.left() - 5.0, p.y),
            egui::Align2::RIGHT_CENTER,
            format!("{y:.1}"),
            FontId::proportional(10.0),
            Color32::LIGHT_GRAY,
        );
    }
    for (values, color) in [
        (Some(primary), Color32::from_rgb(64, 200, 230)),
        (extra.as_ref(), Color32::from_rgb(250, 185, 75)),
    ] {
        if let Some(values) = values {
            let points: Vec<_> = table
                .axis
                .iter()
                .zip(values)
                .map(|(&x, &y)| point(x, y))
                .collect();
            for pair in points.windows(2) {
                painter.line_segment([pair[0], pair[1]], Stroke::new(2.0, color));
            }
            for p in points {
                painter.circle_filled(p, 2.5, color);
            }
        }
    }
    painter.text(
        Pos2::new(rect.left() + 5.0, rect.top() + 5.0),
        egui::Align2::LEFT_TOP,
        unit,
        FontId::proportional(11.0),
        Color32::LIGHT_GRAY,
    );
    painter.text(
        Pos2::new(plot.center().x, rect.bottom() - 5.0),
        egui::Align2::CENTER_BOTTOM,
        format!("Grip / throttle %   {x_min} … {x_max}"),
        FontId::proportional(11.0),
        Color32::LIGHT_GRAY,
    );
    response.on_hover_text(format!(
        "RPM {}. {} values: {:?}",
        table.rpm[*row], unit, primary
    ));
}

pub fn heatmap(ui: &mut egui::Ui, table: &Table) {
    if table.validate().is_err() {
        return;
    }
    let min = table
        .values
        .iter()
        .flatten()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let max = table
        .values
        .iter()
        .flatten()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let width = (ui.available_width() / table.axis.len() as f32).clamp(4.0, 24.0);
    let size = Vec2::new(
        width * table.axis.len() as f32,
        12.0 * table.rpm.len() as f32,
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    for (i, row) in table.values.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            let f = ((value - min) / (max - min).max(f64::EPSILON)) as f32;
            let cell = Rect::from_min_size(
                rect.min + Vec2::new(j as f32 * width, i as f32 * 12.0),
                Vec2::new(width, 12.0),
            );
            painter.rect_filled(
                cell,
                0.0,
                Color32::from_rgb(
                    (35.0 + 200.0 * f) as u8,
                    (90.0 + 70.0 * (1.0 - f)) as u8,
                    (210.0 - 160.0 * f) as u8,
                ),
            );
        }
    }
    if let Some(p) = response.hover_pos() {
        let i = ((p.y - rect.top()) / 12.0) as usize;
        let j = ((p.x - rect.left()) / width) as usize;
        if let Some(value) = table.values.get(i).and_then(|r| r.get(j)) {
            response.on_hover_text(format!(
                "{} RPM · {}% · {}",
                table.rpm[i], table.axis[j], value
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn table_views_render_without_a_window() {
        let ctx = egui::Context::default();
        let mut table = Table::new(
            vec![1000.0, 2000.0],
            vec![0.0, 50.0, 100.0],
            vec![vec![-10.0, 50.0, 90.0], vec![-5.0, 60.0, 100.0]],
        )
        .unwrap();
        let mut row = 0;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                assert!(!editor(ui, "smoke", &mut table, None, false).unwrap());
                chart(ui, &table, &mut row, Some(&table), "Nm");
                heatmap(ui, &table);
            });
        });
        assert!(!output.shapes.is_empty());
        output.textures_delta.clear();
        table.validate().unwrap();
    }
}

fn insert_column(table: &Table) -> Result<Table, String> {
    let x = if table.axis.len() == 1 {
        if table.axis[0] > 0.0 { 0.0 } else { 100.0 }
    } else {
        let pair = table
            .axis
            .windows(2)
            .max_by(|a, b| (a[1] - a[0]).total_cmp(&(b[1] - b[0])))
            .unwrap();
        pair[0] + (pair[1] - pair[0]) / 2.0
    };
    let mut axis = table.axis.clone();
    let j = axis.partition_point(|v| *v < x);
    axis.insert(j, x);
    let values = table
        .rpm
        .iter()
        .zip(&table.values)
        .map(|(&rpm, row)| {
            let mut row = row.clone();
            row.insert(j, lookup(table, rpm, x));
            row
        })
        .collect();
    Table::new(table.rpm.clone(), axis, values)
}
fn bulk(
    table: &Table,
    selected: (usize, usize),
    scope: usize,
    operation: usize,
    amount: f64,
) -> Result<Table, String> {
    if !amount.is_finite() {
        return Err("The operation value must be finite.".into());
    }
    let (r, c) = selected;
    if (scope != 2 && r == 0) || (scope != 1 && c == 0) {
        return Err("Select a value, or the header of the row/column to change.".into());
    }
    if operation == 3 && scope == 0 {
        return Err("Choose a row or column to interpolate.".into());
    }
    let mut out = table.clone();
    let rows = if scope == 2 {
        0..table.rpm.len()
    } else {
        r - 1..r
    };
    let cols = if scope == 1 {
        0..table.axis.len()
    } else {
        c - 1..c
    };
    for i in rows {
        for j in cols.clone() {
            out.values[i][j] = match operation {
                0 => amount,
                1 => table.values[i][j] + amount,
                2 => table.values[i][j] * amount,
                3 => {
                    let (axis, index, a, b) = if scope == 1 {
                        (
                            &table.axis,
                            j,
                            table.values[i][0],
                            *table.values[i].last().unwrap(),
                        )
                    } else {
                        (
                            &table.rpm,
                            i,
                            table.values[0][j],
                            table.values.last().unwrap()[j],
                        )
                    };
                    if axis.len() == 1 {
                        a
                    } else {
                        a + (axis[index] - axis[0]) / (axis.last().unwrap() - axis[0]) * (b - a)
                    }
                }
                _ => return Err("Unknown table operation.".into()),
            };
        }
    }
    out.validate()?;
    Ok(out)
}

#[cfg(test)]
mod operation_tests {
    use super::*;
    fn sample() -> Table {
        Table::new(
            vec![1000., 2000., 4000.],
            vec![0., 20., 100.],
            vec![
                vec![0., 80., 100.],
                vec![10., 30., 60.],
                vec![20., 40., 80.],
            ],
        )
        .unwrap()
    }
    #[test]
    fn column_insertion_interpolates_and_stays_in_bounds() {
        let t = insert_column(&sample()).unwrap();
        assert_eq!(t.axis, vec![0., 20., 60., 100.]);
        assert_eq!(t.values[0], vec![0., 80., 90., 100.]);
    }
    #[test]
    fn bulk_operations_only_change_selected_scope() {
        let t = sample();
        assert_eq!(
            bulk(&t, (2, 2), 0, 0, 7.).unwrap().values[1],
            vec![10., 7., 60.]
        );
        assert_eq!(
            bulk(&t, (2, 0), 1, 1, 5.).unwrap().values[1],
            vec![15., 35., 65.]
        );
        let a = bulk(&t, (0, 2), 2, 2, 2.).unwrap();
        assert_eq!(a.values[0], vec![0., 160., 100.]);
        assert_eq!(a.values[2], vec![20., 80., 80.]);
        assert_eq!(t.values[0][1], 80.);
    }
    #[test]
    fn interpolation_uses_numeric_breakpoints() {
        let t = sample();
        assert_eq!(
            bulk(&t, (1, 0), 1, 3, 0.).unwrap().values[0],
            vec![0., 20., 100.]
        );
        let t = bulk(&t, (0, 1), 2, 3, 0.).unwrap();
        assert!((t.values[1][0] - 20. / 3.).abs() < 1e-12);
        assert!(bulk(&t, (1, 1), 0, 3, 0.).is_err());
        assert!(bulk(&t, (1, 1), 1, 2, f64::MAX).is_err());
    }
}
