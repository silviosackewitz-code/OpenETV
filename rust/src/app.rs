use crate::{
    formats::{self, DssOptions, ImportedTable},
    look,
    model::{self, EtvResult, Shape, Status, Table},
};
use eframe::egui::{self, RichText};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub rpm: String,
    pub grip: String,
    pub threshold: f64,
    pub tolerance: f64,
    pub ramp: f64,
    pub shape: usize,
    pub exponent: f64,
    pub center: f64,
    pub steepness: f64,
    pub max_percent: f64,
    pub per_rpm: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub version: u32,
    pub engine: Table,
    pub request: Table,
    pub engine_name: String,
    pub request_name: String,
    pub settings: Settings,
    pub result: Option<EtvResult>,
    pub raw: Option<Table>,
    pub calculated_from: Option<String>,
    #[serde(default)]
    pub dss: DssOptions,
}
fn axis_text(values: &[f64]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}
pub fn parse_axis(text: &str, percent: bool) -> Result<Vec<f64>, String> {
    let mut values = text
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<f64>()
                .map_err(|_| format!("Invalid breakpoint: {s}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty()
        || values
            .iter()
            .any(|x| !x.is_finite() || *x < 0. || (percent && *x > 100.))
    {
        return Err("Use finite breakpoints: RPM ≥ 0; pedal 0–100%.".into());
    }
    values.sort_by(f64::total_cmp);
    values.dedup();
    if values.len() > 1024 {
        return Err("Too many breakpoints (maximum 1024).".into());
    }
    Ok(values)
}
fn checked_table(table: &Table, throttle: bool) -> Result<(), String> {
    table.validate()?;
    if table.rpm.len() * table.axis.len() > 20_000 {
        return Err("This table exceeds the 20,000-cell editing limit.".into());
    }
    if table.rpm.iter().any(|r| *r < 0.) || table.axis.iter().any(|x| !(0. ..=100.).contains(x)) {
        return Err("RPM must be nonnegative and pedal/throttle breakpoints within 0–100%.".into());
    }
    if throttle
        && table
            .values
            .iter()
            .flatten()
            .any(|x| !(0. ..=100.).contains(x))
    {
        return Err("ETV throttle values must be within 0–100%.".into());
    }
    Ok(())
}
impl Session {
    pub fn sample() -> Self {
        let engine = formats::read_csv(include_str!("../assets/engine_torque_map.csv"))
            .expect("bundled engine table");
        let request = formats::read_csv(include_str!("../assets/demand_map.csv"))
            .expect("bundled request table");
        let settings = Settings {
            rpm: axis_text(&request.rpm),
            grip: axis_text(&request.axis),
            threshold: engine.rpm[engine.rpm.len() / 2],
            tolerance: 0.3,
            ramp: 20.,
            shape: 1,
            exponent: 1.8,
            center: 60.,
            steepness: 8.,
            max_percent: 100.,
            per_rpm: false,
        };
        Self {
            version: 1,
            engine,
            request,
            engine_name: "Example engine — synthetic data".into(),
            request_name: "Example request — synthetic data".into(),
            settings,
            result: None,
            raw: None,
            calculated_from: None,
            dss: DssOptions::default(),
        }
    }
    fn input_key(&self) -> String {
        serde_json::to_string(&(
            &self.engine,
            &self.request,
            &self.settings.rpm,
            &self.settings.grip,
            self.settings.threshold,
            self.settings.tolerance,
        ))
        .expect("validated finite inputs")
    }
    pub fn fresh(&self) -> bool {
        self.result.is_some() && self.calculated_from.as_deref() == Some(self.input_key().as_str())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported OpenETV project version.".into());
        }
        checked_table(&self.engine, false)?;
        checked_table(&self.request, false)?;
        if self.dss.table_unit != "%" || self.dss.range_min != 0. || self.dss.range_max != 100. {
            return Err("ETV export units must be % with range 0–100.".into());
        }
        let s = &self.settings;
        let rpm = parse_axis(&s.rpm, false)?;
        let grip = parse_axis(&s.grip, true)?;
        if rpm.len() * grip.len() > 20_000 {
            return Err("Output table exceeds 20,000 cells.".into());
        }
        if [
            s.threshold,
            s.tolerance,
            s.ramp,
            s.exponent,
            s.center,
            s.steepness,
            s.max_percent,
        ]
        .iter()
        .any(|x| !x.is_finite())
            || s.tolerance < 0.
            || !(0. ..=100.).contains(&s.ramp)
            || !(0. ..=100.).contains(&s.max_percent)
            || s.shape > 4
        {
            return Err("Invalid calculation or generator settings.".into());
        }
        self.shape().validate()?;
        if let Some(r) = &self.result {
            checked_table(&r.table, true)?;
            if r.status.len() != r.table.rpm.len()
                || r.status.iter().any(|row| row.len() != r.table.axis.len())
                || r.outside.iter().any(|x| !x.is_finite())
            {
                return Err("Invalid result diagnostics in project.".into());
            }
        }
        if let Some(t) = &self.raw {
            checked_table(t, true)?;
        }
        Ok(())
    }
    pub fn shape(&self) -> Shape {
        match self.settings.shape {
            0 => Shape::Power { n: 0.6 },
            1 => Shape::Power { n: 1. },
            2 => Shape::Power { n: 1.8 },
            3 => Shape::Power {
                n: self.settings.exponent,
            },
            _ => Shape::SCurve {
                center: self.settings.center,
                steepness: self.settings.steepness,
            },
        }
    }
    pub fn calculate(&mut self) -> Result<(), String> {
        self.validate()?;
        let rpm = parse_axis(&self.settings.rpm, false)?;
        let grip = parse_axis(&self.settings.grip, true)?;
        let result = model::calculate(
            &self.engine,
            &self.request,
            &rpm,
            &grip,
            self.settings.threshold,
            self.settings.tolerance,
        )?;
        checked_table(&result.table, true)?;
        self.raw = Some(result.table.clone());
        self.result = Some(result);
        self.calculated_from = Some(self.input_key());
        Ok(())
    }
    pub fn generate(&mut self) -> Result<(), String> {
        self.validate()?;
        let s = &self.settings;
        let table = model::generate_request(
            &self.engine,
            &parse_axis(&s.rpm, false)?,
            &parse_axis(&s.grip, true)?,
            self.shape(),
            s.max_percent,
            s.per_rpm,
        )?;
        checked_table(&table, false)?;
        self.request = table;
        self.request_name = "Generated torque request".into();
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let bytes = read_small(path)?;
        Self::from_bytes(&bytes)
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut session: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        session.validate()?;
        // Verify the saved baseline, retaining deliberate result edits only for matching inputs.
        let matched = session.fresh();
        session.calculated_from = None;
        if matched {
            let baseline = model::calculate(
                &session.engine,
                &session.request,
                &parse_axis(&session.settings.rpm, false)?,
                &parse_axis(&session.settings.grip, true)?,
                session.settings.threshold,
                session.settings.tolerance,
            )?;
            if session.raw.as_ref() == Some(&baseline.table)
                && session.result.as_ref().is_some_and(|r| {
                    r.table.rpm == baseline.table.rpm
                        && r.table.axis == baseline.table.axis
                        && r.status == baseline.status
                        && r.outside == baseline.outside
                })
            {
                session.calculated_from = Some(session.input_key());
            }
        }
        Ok(session)
    }
    pub fn save_checked(&self, path: &Path, expected: &[u8]) -> Result<(), String> {
        if read_small(path)? != expected {
            return Err(
                "Project changed outside OpenETV. Use Save as to keep your work separately.".into(),
            );
        }
        self.save(path)
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        formats::atomic_write(
            path,
            &serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?,
        )
    }
}
fn read_small(path: &Path) -> Result<Vec<u8>, String> {
    if path.metadata().map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
        return Err("File exceeds the 16 MB limit.".into());
    }
    {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err("File exceeds the 16 MB limit.".into());
        }
        Ok(bytes)
    }
}
#[derive(Clone, Copy, PartialEq)]
enum ImportRole {
    Engine,
    Request,
    Reverse,
}
struct ImportChoice {
    tables: Vec<ImportedTable>,
    role: ImportRole,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Appearance {
    theme: String,
    accent: String,
    font: String,
    size: f32,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            accent: "#d40000".into(),
            font: "plex".into(),
            size: 90.,
        }
    }
}
impl Appearance {
    fn rgb(&self) -> [u8; 3] {
        let n = u32::from_str_radix(self.accent.trim_start_matches('#'), 16).unwrap_or(0xd40000);
        [(n >> 16) as u8, (n >> 8) as u8, n as u8]
    }
}
pub struct App {
    pub session: Session,
    pub step: usize,
    pub message: String,
    undo: Vec<Session>,
    redo: Vec<Session>,
    dirty: bool,
    project: Option<PathBuf>,
    project_bytes: Option<Vec<u8>>,
    import: Option<ImportChoice>,
    row: usize,
    format: usize,
    appearance: Appearance,
    appearance_open: bool,
    manual_open: bool,
    manual_topic: usize,
    result_view: usize,
    principle_open: bool,
    probe_rpm: f64,
    probe_grip: f64,
    confirm: Option<&'static str>,
    close_allowed: bool,
}
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
impl App {
    pub fn new() -> Self {
        Self {
            session: Session::sample(),
            step: 0,
            message: String::new(),
            undo: vec![],
            redo: vec![],
            dirty: false,
            project: None,
            project_bytes: None,
            import: None,
            row: 0,
            format: 0,
            appearance: Appearance::default(),
            appearance_open: false,
            manual_open: false,
            manual_topic: 0,
            result_view: 0,
            principle_open: false,
            probe_rpm: 7000.,
            probe_grip: 50.,
            confirm: None,
            close_allowed: false,
        }
    }
    pub fn start(ctx: &egui::Context) -> Self {
        let mut app = Self::new();
        if let Ok(bytes) = std::fs::read(prefs_path())
            && let Ok(p) = serde_json::from_slice(&bytes)
        {
            app.appearance = p;
        }
        look::fonts(ctx);
        look::configure(
            ctx,
            &app.appearance.theme,
            app.appearance.rgb(),
            &app.appearance.font,
            app.appearance.size,
        );
        app
    }
    fn record(&mut self, before: Session) {
        self.undo.push(before);
        if self.undo.len() > 40 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
    }
    pub fn undo(&mut self) {
        if let Some(old) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.session, old));
            self.dirty = true;
        }
    }
    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.session, next));
            self.dirty = true;
        }
    }
    fn operation(&mut self, op: impl FnOnce(&mut Session) -> Result<(), String>) {
        let before = self.session.clone();
        match op(&mut self.session) {
            Ok(()) => {
                self.record(before);
                self.message.clear();
            }
            Err(e) => {
                self.session = before;
                self.message = e;
            }
        }
    }
    fn import_file(&mut self, role: ImportRole) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Calibration table", &["csv", "xlsx", "xlsm", "dss"])
            .pick_file()
        {
            match formats::read_file(&path) {
                Ok(tables) => self.import = Some(ImportChoice { tables, role }),
                Err(e) => self.message = e,
            }
        }
    }
    fn accept_import(&mut self, index: usize) {
        if let Some(choice) = self.import.take() {
            let t = &choice.tables[index];
            let checked = if choice.role == ImportRole::Reverse {
                if !t.unit.is_empty() && t.unit.trim() != "%" {
                    Err("An ETV map must contain throttle in %.".into())
                } else {
                    checked_table(&t.table, true)
                }
            } else {
                t.require_torque()
                    .and_then(|()| checked_table(&t.table, false))
            };
            if let Err(e) = checked {
                self.message = e;
                self.import = Some(choice);
                return;
            }
            let before = self.session.clone();
            if choice.role != ImportRole::Engine && !t.rpm_path.is_empty() {
                self.session.dss.rpm_path = t.rpm_path.clone();
                self.session.dss.other_path = t.other_path.clone();
                self.session.dss.rpm_unit = t.rpm_unit.clone();
                self.session.dss.other_unit = t.other_unit.clone();
                if choice.role == ImportRole::Reverse {
                    self.session.dss.table_path = t.path.clone();
                }
            }
            match choice.role {
                ImportRole::Engine => {
                    self.session.engine = t.table.clone();
                    self.session.engine_name = t.path.clone();
                    if !t.table.rpm.contains(&self.session.settings.threshold) {
                        self.session.settings.threshold = t.table.rpm[t.table.rpm.len() / 2];
                    }
                }
                ImportRole::Request => {
                    self.session.request = t.table.clone();
                    self.session.request_name = t.path.clone();
                }
                ImportRole::Reverse => {
                    match model::request_from_etv(&self.session.engine, &t.table) {
                        Ok(request) => {
                            self.session.request = request;
                            self.session.request_name = format!("Recovered from {}", t.path);
                        }
                        Err(e) => {
                            self.message = e;
                            return;
                        }
                    }
                }
            }
            if choice.role != ImportRole::Engine {
                self.session.settings.rpm = axis_text(&self.session.request.rpm);
                self.session.settings.grip = axis_text(&self.session.request.axis);
            }
            self.record(before);
            self.row = 0;
            self.message = if choice.role == ImportRole::Reverse {
                "Torque recovered using the engine model. Outside its RPM or throttle range, edge values are held; inspect the model before use.".into()
            } else {
                "Table imported.".into()
            };
        }
    }
    fn save_project(&mut self, save_as: bool) -> bool {
        let path = if !save_as { self.project.clone() } else { None }.or_else(|| {
            rfd::FileDialog::new()
                .add_filter("OpenETV project", &["openetv"])
                .set_file_name("calibration.openetv")
                .save_file()
        });
        let Some(path) = path else { return false };
        let result = if self.project.as_ref() == Some(&path) {
            self.project_bytes
                .as_ref()
                .ok_or_else(|| {
                    "Project snapshot unavailable. Use Save as with another filename.".to_string()
                })
                .and_then(|expected| self.session.save_checked(&path, expected))
        } else {
            self.session.save(&path)
        };
        match result {
            Ok(()) => {
                self.message = format!("Saved {}", path.display());
                self.project_bytes = serde_json::to_vec_pretty(&self.session).ok();
                self.project = Some(path);
                self.dirty = false;
                true
            }
            Err(e) => {
                self.message = e;
                false
            }
        }
    }
    fn open_project(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("OpenETV project", &["openetv"])
            .pick_file()
        {
            match read_small(&path)
                .and_then(|bytes| Session::from_bytes(&bytes).map(|s| (s, bytes)))
            {
                Ok((s, bytes)) => {
                    self.project_bytes = Some(bytes);
                    self.session = s;
                    self.project = Some(path);
                    self.undo.clear();
                    self.redo.clear();
                    self.dirty = false;
                    self.row = 0;
                    self.message = if self.session.fresh() { "Project opened. Saved result and corrections verified against the inputs." } else { "Project opened. Inputs or calculation baseline differ; recalculate before exporting." }.into();
                }
                Err(e) => self.message = e,
            }
        }
    }
    fn export(&mut self) {
        if !self.session.fresh() {
            self.message = "Recalculate after changing inputs before exporting.".into();
            return;
        }
        let Some(result) = &self.session.result else {
            return;
        };
        if let Err(e) = checked_table(&result.table, true) {
            self.message = e;
            return;
        }
        let ext = ["csv", "xlsx", "dss"][self.format];
        let bytes = match formats::export_bytes(&result.table, ext, &self.session.dss) {
            Ok(b) => b,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ETV map", &[ext])
            .set_file_name(format!("ETV-map.{ext}"))
            .save_file()
        {
            match formats::atomic_write(&path, &bytes) {
                Ok(()) => self.message = format!("Exported {}", path.display()),
                Err(e) => self.message = e,
            }
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        look::configure(
            &ctx,
            &self.appearance.theme,
            self.appearance.rgb(),
            &self.appearance.font,
            self.appearance.size,
        );
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && !self.close_allowed {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm = Some("close");
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.save_project(false);
        }
        if !ctx.egui_wants_keyboard_input() {
            if ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Z,
                )
            }) {
                self.redo();
            } else if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) {
                self.undo();
            }
        }
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("OpenETV");
                ui.weak(concat!("v", env!("CARGO_PKG_VERSION")));
                ui.separator();
                if ui.button("Open project…").clicked() {
                    if self.dirty {
                        self.confirm = Some("open")
                    } else {
                        self.open_project();
                    }
                }
                if ui.button("Save").clicked() {
                    self.save_project(false);
                }
                if ui.button("Save as…").clicked() {
                    self.save_project(true);
                }
                if ui
                    .add_enabled(!self.undo.is_empty(), egui::Button::new("Undo"))
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_enabled(!self.redo.is_empty(), egui::Button::new("Redo"))
                    .clicked()
                {
                    self.redo();
                }
                if self.dirty {
                    ui.label("• Unsaved");
                }
            });
        });
        egui::Panel::bottom("status").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(if self.message.is_empty() {
                    "Local calculation · no ECU connection"
                } else {
                    &self.message
                });
            });
        });
        egui::Panel::left("steps")
            .resizable(false)
            .exact_size(192.)
            .show(ui, |ui| {
                ui.add_space(12.);
                ui.label(RichText::new("THROTTLE MAP BUILDER").small());
                ui.add_space(16.);
                for (n, label) in [
                    "1   Engine torque",
                    "2   Torque request",
                    "3   Calculation",
                    "4   ETV map",
                    "5   Export",
                ]
                .iter()
                .enumerate()
                {
                    if ui
                        .add_sized(
                            [172., 36.],
                            egui::Button::new(*label).selected(self.step == n),
                        )
                        .clicked()
                    {
                        self.step = n;
                        self.row = 0;
                    }
                }
                ui.add_space(24.);
                ui.separator();
                if ui.button("How it works").clicked() {
                    self.principle_open = true;
                }
                if ui
                    .button("Manual")
                    .on_hover_text("Shows how OpenETV works.")
                    .clicked()
                {
                    self.manual_topic = 0;
                    self.manual_open = true;
                }
                if ui.button("Appearance").clicked() {
                    self.appearance_open = true;
                }
                ui.add_space(12.);
                ui.small("Engine torque → Torque request → Throttle target");
                if self.session.result.is_some() {
                    ui.add_space(12.);
                    ui.label(if self.session.fresh() {
                        "Result up to date"
                    } else {
                        "Inputs changed — recalculate"
                    });
                }
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(18))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("page")
                    .show(ui, |ui| match self.step {
                        0 => self.engine_page(ui),
                        1 => self.request_page(ui),
                        2 => self.calculation_page(ui),
                        3 => self.result_page(ui),
                        _ => self.export_page(ui),
                    });
            });
        self.windows(&ctx);
    }
    fn page_heading(&mut self, ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.horizontal(|ui| {
            ui.heading(title);
            if ui
                .button("?")
                .on_hover_text(format!("Help for {title}"))
                .clicked()
            {
                self.manual_topic = self.step + 1;
                self.manual_open = true;
            }
        });
        ui.label(subtitle);
        ui.add_space(8.);
        let (bar, _) = ui.allocate_exact_size(egui::vec2(64., 3.), egui::Sense::hover());
        let color = look::theme::applied(ui.ctx())
            .map(|look| look.palette().accent)
            .unwrap_or(ui.visuals().hyperlink_color);
        ui.painter().rect_filled(bar, 1., color);
        ui.add_space(10.);
    }
    fn engine_page(&mut self, ui: &mut egui::Ui) {
        self.page_heading(
            ui,
            "Engine torque",
            "RPM × throttle position → torque in Nm",
        );
        ui.weak(&self.session.engine_name);
        ui.add_space(10.);
        ui.horizontal(|ui| {
            if ui.button("Import engine table…").clicked() {
                self.import_file(ImportRole::Engine);
            }
            if ui.button("Next: torque request →").clicked() {
                self.step = 1;
                self.row = 0;
            }
        });
        self.edit(ui, false);
        ui.add_space(14.);
        ui.strong("Torque curve");
        crate::grid::chart(ui, &self.session.engine, &mut self.row, None, "Nm");
    }
    fn request_page(&mut self, ui: &mut egui::Ui) {
        self.page_heading(
            ui,
            "Torque request",
            "RPM × pedal position → requested torque in Nm",
        );
        ui.weak(&self.session.request_name);
        ui.add_space(10.);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Import request…").clicked() {
                self.import_file(ImportRole::Request);
            }
            if ui.button("Recover request from ETV…").clicked() {
                self.import_file(ImportRole::Reverse);
            }
        });
        let before = self.session.clone();
        let s = &mut self.session.settings;
        let mut generate = false;
        egui::CollapsingHeader::new("Generate a torque request").show(ui,|ui|{
            ui.horizontal_wrapped(|ui|{egui::ComboBox::from_label("Shape").selected_text(["Cable-like","Linear","Corner exit","Custom power","S-curve"][s.shape.min(4)]).show_ui(ui,|ui|{for(i,label)in ["Cable-like","Linear","Corner exit","Custom power","S-curve"].iter().enumerate(){ui.selectable_value(&mut s.shape,i,*label);}});
            if s.shape==3{ui.add(egui::DragValue::new(&mut s.exponent).range(0.05..=10.).speed(0.05).prefix("Exponent "));}
            if s.shape==4{ui.add(egui::DragValue::new(&mut s.center).range(1. ..=99.).prefix("Fine zone % "));ui.add(egui::DragValue::new(&mut s.steepness).range(0.5..=50.).speed(0.1).prefix("Sharpness "));}
            ui.add(egui::DragValue::new(&mut s.max_percent).range(0. ..=100.).suffix("% of maximum torque"));});
            ui.checkbox(&mut s.per_rpm,"Scale to maximum at each RPM (torque follows engine speed)");
            ui.small("Unchecked: the same pedal asks for the same Nm, capped by available torque. Cable-like is an illustrative curve, not a measured cable model.");
            ui.label("Generator RPM breakpoints");ui.text_edit_singleline(&mut s.rpm);ui.label("Pedal breakpoints (%)");ui.text_edit_singleline(&mut s.grip);
            generate=ui.button("Generate request").clicked();
        });
        if self.session.settings != before.settings {
            self.record(before);
        }
        let shape = self.session.shape();
        let axis: Vec<f64> = (0..=100).step_by(5).map(f64::from).collect();
        let preview = Table {
            rpm: vec![0.],
            axis: axis.clone(),
            values: vec![
                axis.iter()
                    .map(|x| shape.value(x / 100.) * self.session.settings.max_percent)
                    .collect(),
            ],
        };
        egui::CollapsingHeader::new("Shape preview").show(ui, |ui| {
            ui.small("Requested share of maximum torque (%)");
            crate::grid::chart(ui, &preview, &mut 0, None, "%");
        });
        if generate {
            self.operation(Session::generate);
        }
        self.edit(ui, true);
        ui.add_space(14.);
        ui.strong("Request and available torque");
        let max = available(&self.session.engine, &self.session.request);
        ui.small("Cyan: requested torque · amber: available engine torque");
        crate::grid::chart(ui, &self.session.request, &mut self.row, Some(&max), "Nm");
        if ui.button("Next: calculation →").clicked() {
            self.step = 2;
        }
    }
    fn edit(&mut self, ui: &mut egui::Ui, request: bool) {
        let before = self.session.clone();
        let table = if request {
            &mut self.session.request
        } else {
            &mut self.session.engine
        };
        match crate::grid::editor(
            ui,
            if request { "request" } else { "engine" },
            table,
            None,
            false,
        ) {
            Ok(true) => {
                if let Err(e) = checked_table(table, false) {
                    self.session = before;
                    self.message = e;
                } else {
                    self.record(before);
                }
            }
            Err(e) => {
                self.session = before;
                self.message = e;
            }
            _ => {}
        }
    }
    fn calculation_page(&mut self, ui: &mut egui::Ui) {
        self.page_heading(
            ui,
            "Calculation",
            "Choose the output grid and calculate the throttle target.",
        );
        let before = self.session.clone();
        let s = &mut self.session.settings;
        ui.add_space(12.);
        ui.label("Output RPM breakpoints");
        ui.text_edit_singleline(&mut s.rpm);
        ui.label("Output pedal breakpoints (%)");
        ui.text_edit_singleline(&mut s.grip);
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::DragValue::new(&mut s.threshold)
                    .range(0. ..=100000.)
                    .speed(100.)
                    .prefix("RPM calc method "),
            );
            ui.add(
                egui::DragValue::new(&mut s.tolerance)
                    .range(0. ..=1000.)
                    .speed(0.1)
                    .prefix("Max torque tolerance ")
                    .suffix(" Nm"),
            );
        });
        ui.label("Up to the threshold, saturation uses the first throttle breakpoint reaching maximum torque minus tolerance; above it, the last. Between breakpoints, both tables interpolate linearly.");
        ui.label("All-zero engine rows are omitted. Outside its RPM range the nearest usable row is held and reported.");
        if self.session.settings != before.settings {
            self.record(before);
        }
        if ui.button("Calculate ETV map").clicked() {
            self.operation(Session::calculate);
            if self.message.is_empty() {
                self.step = 3;
                self.row = 0;
            }
        }
        ui.add_space(24.);
        self.principle(ui);
    }
    fn result_page(&mut self, ui: &mut egui::Ui) {
        self.page_heading(ui, "ETV map", "RPM × pedal position → throttle target in %");
        if self.session.result.is_none() {
            ui.label("Calculate a map first.");
            if ui.button("Go to calculation").clicked() {
                self.step = 2;
            }
            return;
        }
        let fresh = self.session.fresh();
        if !fresh {
            ui.colored_label(
                look::AMBER,
                "Inputs changed. Recalculate before editing or exporting this result.",
            );
        }
        let mut action = None;
        let before_ramp = self.session.clone();
        ui.add_enabled_ui(fresh, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::DragValue::new(&mut self.session.settings.ramp)
                        .range(0. ..=100.)
                        .prefix("Ramp to ")
                        .suffix("% pedal"),
                );
                for (key, label) in [
                    ("zero", "Zero-gas fix"),
                    ("flat", "Flat-spot fix"),
                    ("dips", "Remove dips"),
                    ("reset", "Reset to calculated"),
                ] {
                    if ui.button(label).clicked() {
                        action = Some(key);
                    }
                }
            });
        });
        if self.session.settings.ramp != before_ramp.settings.ramp {
            self.record(before_ramp);
        }
        if let Some(action) = action {
            self.operation(|s| {
                let r = s.result.as_mut().ok_or("No result")?;
                r.table = match action {
                    "zero" => model::zero_gas_fix(&r.table, s.settings.ramp)?,
                    "flat" => model::flat_spot_fix(&r.table),
                    "dips" => model::monotonic_fix(&r.table),
                    _ => s.raw.clone().ok_or("No raw result")?,
                };
                checked_table(&r.table, true)
            });
        }
        let before = self.session.clone();
        let r = self.session.result.as_mut().unwrap();
        match crate::grid::editor(ui, "result", &mut r.table, Some(&r.status), !fresh) {
            Ok(true) => {
                if r.table.rpm != before.result.as_ref().unwrap().table.rpm
                    || r.table.axis != before.result.as_ref().unwrap().table.axis
                {
                    self.session = before;
                    self.message="Result axes belong to the calculation. Change output breakpoints and recalculate.".into();
                } else if let Err(e) = checked_table(&r.table, true) {
                    self.session = before;
                    self.message = e;
                } else {
                    self.record(before);
                }
            }
            Err(e) => {
                self.session = before;
                self.message = e;
            }
            _ => {}
        }
        let r = self.session.result.as_ref().unwrap();
        ui.label(format!(
            "Original calculation: ▲ {} saturated · ▽ {} below minimum · ! {} non-monotonic",
            count(r, Status::Saturated),
            count(r, Status::BelowMin),
            count(r, Status::NonMonotonic)
        ));
        if !r.outside.is_empty() {
            ui.label(format!(
                "Engine RPM outside range: {}",
                axis_text(&r.outside)
            ));
        }
        ui.small("Cell symbols describe the original inversion. Corrections may change the torque delivered.");
        ui.add_space(14.);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.result_view, 0, "Throttle curve");
            ui.selectable_value(&mut self.result_view, 1, "Torque comparison");
        });
        if self.result_view == 0 {
            crate::grid::chart(ui, &r.table, &mut self.row, None, "% throttle");
        }
        if self.result_view == 1
            && let Ok(actual) = model::request_from_etv(&self.session.engine, &r.table)
        {
            let mut worst = 0f64;
            for (i, rpm) in actual.rpm.iter().enumerate() {
                for (j, grip) in actual.axis.iter().enumerate() {
                    worst = worst.max(
                        (actual.values[i][j] - model::lookup(&self.session.request, *rpm, *grip))
                            .abs(),
                    );
                }
            }
            ui.label(format!(
                "Modelled torque after corrections · largest difference from request: {worst:.2} Nm"
            ));
            ui.small("Cyan: modelled torque · amber: requested torque. This is a model comparison, not a dyno measurement.");
            crate::grid::chart(
                ui,
                &actual,
                &mut self.row,
                Some(&self.session.request),
                "Nm",
            );
        }
        if ui.button("Next: export →").clicked() {
            self.step = 4;
        }
    }
    fn export_page(&mut self, ui: &mut egui::Ui) {
        self.page_heading(
            ui,
            "Export",
            "Save the throttle target for your ECU workflow.",
        );
        ui.label("A calculated map is a first attempt. Verify it on the dyno, pedal position by pedal position, before anyone rides it.");
        ui.horizontal(|ui| {
            for (i, name) in ["CSV", "Excel (.xlsx)", "Mectronik (.dss)"]
                .iter()
                .enumerate()
            {
                ui.selectable_value(&mut self.format, i, *name);
            }
        });
        let before_export = self.session.clone();
        if self.format == 2 {
            egui::Grid::new("dss-options")
                .num_columns(2)
                .show(ui, |ui| {
                    for (label, value) in [
                        ("Table path", &mut self.session.dss.table_path),
                        ("RPM axis path", &mut self.session.dss.rpm_path),
                        ("Pedal axis path", &mut self.session.dss.other_path),
                        ("RPM unit", &mut self.session.dss.rpm_unit),
                        ("Pedal unit", &mut self.session.dss.other_unit),
                    ] {
                        ui.label(label);
                        ui.text_edit_singleline(value);
                        ui.end_row();
                    }
                });
            ui.small("Value unit: % · range: 0–100. Confirm the ECU table and axis paths before importing.");
        }
        if serde_json::to_value(&self.session.dss).ok()
            != serde_json::to_value(&before_export.dss).ok()
        {
            self.record(before_export);
        }
        if !self.session.fresh() {
            ui.label("No current result. Calculate the map before exporting.");
        }
        if ui
            .add_enabled(self.session.fresh(), egui::Button::new("Export ETV map…"))
            .clicked()
        {
            self.export();
        }
    }
    fn principle(&mut self, ui: &mut egui::Ui) {
        ui.heading("How the map works");
        ui.label("The grip requests torque. The engine table tells us which throttle opening can produce it. OpenETV combines both into the throttle target table read by the ECU.");
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::Slider::new(&mut self.probe_rpm, 0. ..=20000.).text("RPM"));
            ui.add(egui::Slider::new(&mut self.probe_grip, 0. ..=100.).text("Pedal %"));
        });
        let requested = model::lookup(&self.session.request, self.probe_rpm, self.probe_grip);
        let (row, outside) = model::engine_row(&self.session.engine, self.probe_rpm);
        let max = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        match model::calculate(
            &self.session.engine,
            &self.session.request,
            &[self.probe_rpm],
            &[self.probe_grip],
            self.session.settings.threshold,
            self.session.settings.tolerance,
        ) {
            Ok(r) => {
                let throttle = r.table.values[0][0];
                let slice = Table {
                    rpm: vec![self.probe_rpm],
                    axis: self.session.engine.axis.clone(),
                    values: vec![row],
                };
                let actual = model::lookup(&slice, self.probe_rpm, throttle);
                ui.horizontal_wrapped(|ui| {
                    for (title, value) in [
                        ("PEDAL", format!("{:.1}%", self.probe_grip)),
                        ("TORQUE REQUEST", format!("{requested:.2} Nm")),
                        ("THROTTLE TARGET", format!("{throttle:.1}%")),
                        ("MODELLED TORQUE", format!("{actual:.2} Nm")),
                    ] {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.set_min_width(140.);
                                ui.small(title);
                                ui.label(RichText::new(value).size(21.));
                            });
                        });
                    }
                });
                ui.label(format!(
                    "Available at this RPM: {max:.2} Nm. Calculation status: {:?}.",
                    r.status[0][0]
                ));
                if outside {
                    ui.label("This RPM is outside the measured engine range; an edge row is used.");
                }
                if self.session.fresh()
                    && let Some(result) = &self.session.result
                {
                    let final_tps = model::lookup(&result.table, self.probe_rpm, self.probe_grip);
                    let final_nm = model::lookup(&slice, self.probe_rpm, final_tps);
                    ui.label(format!("Finished map at this point: {final_tps:.1}% throttle → {final_nm:.2} Nm, including corrections and output-table interpolation."));
                }
            }
            Err(e) => {
                ui.label(e);
            }
        }
        ui.label("At a fixed pedal position, absolute-torque mode holds the requested Nm as revs change, until the engine reaches its limit. Per-RPM mode instead follows the engine's torque curve.");
        ui.label("Zero-gas, plateau and dip corrections deliberately change the result. The modelled-torque comparison makes those changes visible. Engine braking, cylinder cut and ECU control loops are separate functions.");
    }
    fn windows(&mut self, ctx: &egui::Context) {
        if self.import.is_some() {
            let mut open = true;
            let mut selected = None;
            egui::Window::new("Choose table")
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Select a table and check its unit.");
                    let choice = self.import.as_ref().unwrap();
                    for (i, t) in choice.tables.iter().enumerate() {
                        let valid = if choice.role == ImportRole::Reverse {
                            t.unit.is_empty() || t.unit.trim() == "%"
                        } else {
                            t.require_torque().is_ok()
                        };
                        if ui
                            .add_enabled(
                                valid,
                                egui::Button::new(format!(
                                    "{} · {} · {} × {}",
                                    t.path,
                                    t.unit,
                                    t.table.rpm.len(),
                                    t.table.axis.len()
                                )),
                            )
                            .on_hover_text(if valid {
                                "Import this table"
                            } else {
                                "Wrong units for this input"
                            })
                            .clicked()
                        {
                            selected = Some(i);
                        }
                    }
                });
            if let Some(i) = selected {
                self.accept_import(i);
            } else if !open {
                self.import = None;
            }
        }
        if self.manual_open {
            egui::Window::new("OpenETV manual")
                .open(&mut self.manual_open)
                .default_size([720., 640.])
                .min_width(520.)
                .show(ctx, |ui| crate::manual::show(ui, &mut self.manual_topic));
        }
        if self.principle_open {
            let mut open = true;
            egui::Window::new("How it works")
                .open(&mut open)
                .default_width(780.)
                .show(ctx, |ui| {
                    self.principle(ui);
                });
            self.principle_open = open;
        }
        if self.appearance_open {
            let mut open = true;
            egui::Window::new("Appearance")
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        for theme in ["system", "dark", "light", "industrial", "terminal"] {
                            ui.selectable_value(&mut self.appearance.theme, theme.into(), theme);
                        }
                    });
                    let mut rgb = self.appearance.rgb();
                    if ui.color_edit_button_srgb(&mut rgb).changed() {
                        self.appearance.accent =
                            format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]);
                    }
                    ui.horizontal(|ui| {
                        for font in crate::look::fonts::Font::ALL {
                            ui.selectable_value(
                                &mut self.appearance.font,
                                font.key().into(),
                                font.name(),
                            );
                        }
                    });
                    ui.add(
                        egui::Slider::new(&mut self.appearance.size, 70. ..=150.).text("Size (%)"),
                    );
                    look::configure(
                        ctx,
                        &self.appearance.theme,
                        self.appearance.rgb(),
                        &self.appearance.font,
                        self.appearance.size,
                    );
                    if ui.button("Save appearance").clicked() {
                        let path = prefs_path();
                        let result = (|| {
                            std::fs::create_dir_all(path.parent().ok_or("No preference folder")?)
                                .map_err(|e| e.to_string())?;
                            let mut data: serde_json::Value = std::fs::read(&path)
                                .ok()
                                .and_then(|b| serde_json::from_slice(&b).ok())
                                .unwrap_or(serde_json::json!({}));
                            if !data.is_object() {
                                return Err(
                                    "Existing preferences are not an object; kept unchanged."
                                        .to_string(),
                                );
                            }
                            data["theme"] = self.appearance.theme.clone().into();
                            data["accent"] = self.appearance.accent.clone().into();
                            data["font"] = self.appearance.font.clone().into();
                            data["size"] = self.appearance.size.into();
                            formats::atomic_write(
                                &path,
                                &serde_json::to_vec_pretty(&data).map_err(|e| e.to_string())?,
                            )
                        })();
                        match result {
                            Ok(()) => {
                                self.appearance_open = false;
                                self.message = "Appearance saved.".into();
                            }
                            Err(e) => self.message = e,
                        }
                    }
                });
            self.appearance_open &= open;
        }
        if let Some(action) = self.confirm {
            egui::Window::new("Unsaved project")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Save your input tables, settings and result before leaving?");
                    ui.horizontal(|ui| {
                        let mut proceed = false;
                        if ui.button("Save").clicked() {
                            proceed = self.save_project(false);
                        }
                        if ui.button("Discard").clicked() {
                            proceed = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm = None;
                        }
                        if proceed {
                            self.confirm = None;
                            if action == "close" {
                                self.close_allowed = true;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            } else {
                                self.open_project();
                            }
                        }
                    });
                });
        }
    }
}
fn prefs_path() -> PathBuf {
    std::env::var_os("OPENETV_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(
                std::env::var_os("HOME")
                    .or_else(|| std::env::var_os("USERPROFILE"))
                    .unwrap_or_default(),
            )
            .join(".openetv")
        })
        .join("prefs.json")
}
fn count(r: &EtvResult, s: Status) -> usize {
    r.status.iter().flatten().filter(|&&x| x == s).count()
}
fn available(engine: &Table, request: &Table) -> Table {
    Table {
        rpm: request.rpm.clone(),
        axis: request.axis.clone(),
        values: request
            .rpm
            .iter()
            .map(|rpm| {
                let (row, _) = model::engine_row(engine, *rpm);
                vec![row.into_iter().fold(f64::NEG_INFINITY, f64::max); request.axis.len()]
            })
            .collect(),
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}
