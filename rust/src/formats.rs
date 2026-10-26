use crate::model::Table;
use calamine::Reader;
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::Path,
};

#[derive(Clone, Debug)]
pub struct ImportedTable {
    pub path: String,
    pub unit: String,
    pub table: Table,
    pub rpm_path: String,
    pub other_path: String,
    pub rpm_unit: String,
    pub other_unit: String,
}
impl ImportedTable {
    pub fn require_torque(&self) -> Result<(), String> {
        let unit = self.unit.trim();
        if unit.is_empty() || unit.eq_ignore_ascii_case("nm") {
            return Ok(());
        }
        Err(format!(
            "`{}` cannot be used as torque: it must be in Nm. {}",
            self.path,
            if unit == "%" {
                "This is a throttle map, not torque.".into()
            } else {
                format!("It is in {unit}.")
            }
        ))
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DssOptions {
    pub table_path: String,
    pub table_unit: String,
    pub range_min: f64,
    pub range_max: f64,
    pub rpm_path: String,
    pub rpm_unit: String,
    pub other_path: String,
    pub other_unit: String,
}
impl Default for DssOptions {
    fn default() -> Self {
        Self {
            table_path: "ETV.Target.TPS".into(),
            table_unit: "%".into(),
            range_min: 0.,
            range_max: 100.,
            rpm_path: "BreakPt.RPM".into(),
            rpm_unit: "1/min".into(),
            other_path: "BreakPt.GAS".into(),
            other_unit: "%".into(),
        }
    }
}
fn number(cell: &str) -> Result<f64, String> {
    let n: f64 = cell
        .trim()
        .parse()
        .map_err(|_| format!("Cell {cell:?} is empty or is not a number."))?;
    if !n.is_finite() {
        return Err("Cells must be finite numbers.".into());
    }
    Ok(n)
}
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_CELLS: usize = 20_000;
fn read_limited(path: &Path) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() as u64 > MAX_FILE_BYTES {
        return Err("File exceeds the 16 MB import limit.".into());
    }
    Ok(data)
}
fn from_rows(mut rows: Vec<Vec<String>>) -> Result<Table, String> {
    rows.retain(|r| r.iter().any(|c| !c.trim().is_empty()));
    if rows.len() < 2 || rows[0].len() < 2 {
        return Err("A table needs column breakpoints and at least one row of values.".into());
    }
    if (rows.len() - 1)
        .checked_mul(rows[0].len() - 1)
        .is_none_or(|n| n > MAX_CELLS)
    {
        return Err("Table exceeds the 20,000-cell limit.".into());
    }
    let axis = rows[0][1..]
        .iter()
        .map(|s| number(s))
        .collect::<Result<Vec<_>, _>>()?;
    let mut rpm = Vec::new();
    let mut values = Vec::new();
    for (i, row) in rows.iter().enumerate().skip(1) {
        if row.len() != axis.len() + 1 {
            return Err(format!("Row {} does not match the header width.", i + 1));
        }
        rpm.push(number(&row[0])?);
        values.push(
            row[1..]
                .iter()
                .map(|s| number(s))
                .collect::<Result<Vec<_>, _>>()?,
        );
    }
    Table::new(rpm, axis, values)
}
pub fn read_csv(text: &str) -> Result<Table, String> {
    if text.len() as u64 > MAX_FILE_BYTES {
        return Err("CSV exceeds the 16 MB import limit.".into());
    }
    let text = text.trim_start_matches('\u{feff}');
    let delimiter = if text.lines().next().unwrap_or("").contains(';') {
        b';'
    } else {
        b','
    };
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delimiter)
        .from_reader(text.as_bytes());
    let rows = reader
        .records()
        .map(|r| {
            r.map(|r| {
                r.iter()
                    .map(|s| {
                        if delimiter == b';' {
                            s.replace(',', ".")
                        } else {
                            s.to_owned()
                        }
                    })
                    .collect()
            })
            .map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    from_rows(rows)
}
pub fn to_csv(table: &Table) -> Result<String, String> {
    table.validate()?;
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    writer
        .write_record(
            std::iter::once("RPM\\Pedal[%]".into())
                .chain(table.axis.iter().map(ToString::to_string)),
        )
        .map_err(|e| e.to_string())?;
    for (rpm, row) in table.rpm.iter().zip(&table.values) {
        writer
            .write_record(
                std::iter::once(rpm.to_string()).chain(row.iter().map(ToString::to_string)),
            )
            .map_err(|e| e.to_string())?;
    }
    String::from_utf8(writer.into_inner().map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn child<'a, 'i>(n: roxmltree::Node<'a, 'i>, name: &str) -> &'a str {
    n.children()
        .find(|c| c.has_tag_name(name))
        .and_then(|c| c.text())
        .unwrap_or("")
        .trim()
}
fn numbers(n: roxmltree::Node<'_, '_>) -> Result<Vec<f64>, String> {
    n.children()
        .filter(|c| c.has_tag_name("value"))
        .map(|v| number(v.text().unwrap_or("")))
        .collect()
}
pub fn parse_dss(text: &str) -> Result<Vec<ImportedTable>, String> {
    if text.len() as u64 > MAX_FILE_BYTES {
        return Err("DataSubset exceeds the 16 MB import limit.".into());
    }
    let doc = roxmltree::Document::parse_with_options(
        text.trim_start_matches('\u{feff}'),
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 500_000,
        },
    )
    .map_err(|e| e.to_string())?;
    let root = doc.root_element();
    if !root.has_tag_name("dataset") {
        return Err("This is not a DataSubset dataset.".into());
    }
    let mut vectors = HashMap::new();
    for v in root.children().filter(|n| n.has_tag_name("vector")) {
        let path = child(v, "path");
        if path.is_empty() || vectors.contains_key(path) {
            return Err("Vector paths must be present and unique.".into());
        }
        vectors.insert(path, (child(v, "unit"), numbers(v)?));
    }
    let mut out = Vec::new();
    for t in root.children().filter(|n| n.has_tag_name("table_3d")) {
        let path = child(t, "path");
        if path.is_empty() || out.iter().any(|v: &ImportedTable| v.path == path) {
            return Err("Table paths must be present and unique.".into());
        }
        let xp = child(t, "AxeX");
        let yp = child(t, "AxeY");
        let (xu, x) = vectors
            .get(xp)
            .ok_or_else(|| format!("Missing axis vector {xp:?}."))?;
        let (yu, y) = vectors
            .get(yp)
            .ok_or_else(|| format!("Missing axis vector {yp:?}."))?;
        let matrix = t
            .children()
            .filter(|n| n.has_tag_name("row"))
            .map(numbers)
            .collect::<Result<Vec<_>, _>>()?;
        if x.len().checked_mul(y.len()).is_none_or(|n| n > MAX_CELLS) {
            return Err("Table exceeds the 20,000-cell limit.".into());
        }
        if x.is_empty()
            || y.is_empty()
            || matrix.len() != y.len()
            || matrix.iter().any(|r| r.len() != x.len())
        {
            return Err(format!("Table {path} dimensions do not match its vectors."));
        }
        let is_rpm = |unit: &str, path: &str| {
            unit.eq_ignore_ascii_case("1/min") || path.to_uppercase().contains("RPM")
        };
        let (rpm, axis, values, rp, ru, op, ou) = if is_rpm(yu, yp) && !is_rpm(xu, xp) {
            (y.clone(), x.clone(), matrix, yp, *yu, xp, *xu)
        } else {
            let transposed = (0..x.len())
                .map(|i| matrix.iter().map(|r| r[i]).collect())
                .collect();
            (x.clone(), y.clone(), transposed, xp, *xu, yp, *yu)
        };
        out.push(ImportedTable {
            path: path.into(),
            unit: child(t, "unit").into(),
            table: Table::new(rpm, axis, values)?,
            rpm_path: rp.into(),
            rpm_unit: ru.into(),
            other_path: op.into(),
            other_unit: ou.into(),
        });
    }
    if out.is_empty() {
        return Err("The file holds no table_3d table.".into());
    }
    Ok(out)
}
pub fn read_file(path: &Path) -> Result<Vec<ImportedTable>, String> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let bytes = read_limited(path)?;
    if extension == "dss" {
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        return parse_dss(text);
    }
    let table = if extension == "xlsx" || extension == "xlsm" {
        let mut archive =
            zip::ZipArchive::new(std::io::Cursor::new(&bytes)).map_err(|e| e.to_string())?;
        if archive.len() > 4096 {
            return Err("Excel archive contains too many entries.".into());
        }
        let mut expanded = 0u64;
        for i in 0..archive.len() {
            let entry = archive.by_index_raw(i).map_err(|e| e.to_string())?;
            expanded = expanded
                .checked_add(entry.size())
                .ok_or("Excel archive is too large.")?;
            if expanded > 64 * 1024 * 1024 {
                return Err("Excel archive exceeds the 64 MB expanded limit.".into());
            }
        }
        drop(archive);
        let mut book =
            calamine::Xlsx::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
        let sheet = book
            .sheet_names()
            .first()
            .ok_or("The workbook has no sheets.")?
            .clone();
        let mut cells = book
            .worksheet_cells_reader(&sheet)
            .map_err(|e| e.to_string())?;
        let mut used = Vec::new();
        let (mut height, mut width) = (0usize, 0usize);
        while let Some(cell) = cells.next_cell().map_err(|e| e.to_string())? {
            let (r, c) = cell.get_position();
            height = height.max(r as usize + 1);
            width = width.max(c as usize + 1);
            if height.checked_mul(width).is_none_or(|n| n > 25_000) || used.len() >= 25_000 {
                return Err("Excel worksheet exceeds the calibration-table editing limit.".into());
            }
            let value =
                match cell.get_value() {
                    calamine::DataRef::Int(n) => n.to_string(),
                    calamine::DataRef::Float(n) => n.to_string(),
                    calamine::DataRef::String(s) => s.clone(),
                    calamine::DataRef::SharedString(s) => (*s).to_owned(),
                    calamine::DataRef::Empty => String::new(),
                    _ => return Err(
                        "Excel table contains an error, boolean, date or other nonnumeric cell."
                            .into(),
                    ),
                };
            used.push((r as usize, c as usize, value));
        }
        let mut rows = vec![vec![String::new(); width]; height];
        for (r, c, value) in used {
            rows[r][c] = value;
        }
        from_rows(rows)?
    } else if extension == "xls" {
        return Err("Save the old Excel file as .xlsx or CSV.".into());
    } else {
        let text = match std::str::from_utf8(&bytes) {
            Ok(s) => s.to_owned(),
            Err(_) => encoding_rs::WINDOWS_1252.decode(&bytes).0.into_owned(),
        };
        read_csv(&text)?
    };
    Ok(vec![ImportedTable {
        path: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        unit: String::new(),
        table,
        rpm_path: String::new(),
        other_path: String::new(),
        rpm_unit: String::new(),
        other_unit: String::new(),
    }])
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
pub fn export_bytes(
    table: &Table,
    extension: &str,
    options: &DssOptions,
) -> Result<Vec<u8>, String> {
    table.validate()?;
    match extension
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "csv" => Ok(to_csv(table)?.into_bytes()),
        "xlsx" => {
            let mut book = rust_xlsxwriter::Workbook::new();
            let sheet = book.add_worksheet();
            if table.rpm.len() > 1_048_575 || table.axis.len() > 16_383 {
                return Err("The table exceeds Excel worksheet limits.".into());
            }
            sheet
                .write_string(0, 0, "RPM\\Pedal[%]")
                .map_err(|e| e.to_string())?;
            for (j, x) in table.axis.iter().enumerate() {
                sheet
                    .write_number(0, (j + 1) as u16, *x)
                    .map_err(|e| e.to_string())?;
            }
            for (i, (rpm, row)) in table.rpm.iter().zip(&table.values).enumerate() {
                sheet
                    .write_number((i + 1) as u32, 0, *rpm)
                    .map_err(|e| e.to_string())?;
                for (j, x) in row.iter().enumerate() {
                    sheet
                        .write_number((i + 1) as u32, (j + 1) as u16, *x)
                        .map_err(|e| e.to_string())?;
                }
            }
            book.save_to_buffer().map_err(|e| e.to_string())
        }
        "dss" => {
            let o = options;
            if !o.range_min.is_finite()
                || !o.range_max.is_finite()
                || o.range_min > o.range_max
                || o.table_path.trim().is_empty()
                || o.rpm_path.trim().is_empty()
                || o.other_path.trim().is_empty()
                || o.rpm_path.trim() == o.other_path.trim()
            {
                return Err("DataSubset paths or value range are invalid.".into());
            }
            let mut xml = format!(
                "<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n<dataset notes=\"\" name=\"{}\"><table_3d><path>{}</path><unit>{}</unit><rangeMin>{}</rangeMin><rangeMax>{}</rangeMax><AxeX>{}</AxeX><AxeY>{}</AxeY>",
                escape(&o.table_path),
                escape(&o.table_path),
                escape(&o.table_unit),
                o.range_min,
                o.range_max,
                escape(&o.rpm_path),
                escape(&o.other_path)
            );
            for j in 0..table.axis.len() {
                xml.push_str("<row>");
                for row in &table.values {
                    xml.push_str(&format!("<value>{}</value>", row[j]));
                }
                xml.push_str("</row>");
            }
            xml.push_str("</table_3d>");
            for (path, unit, axis) in [
                (&o.rpm_path, &o.rpm_unit, &table.rpm),
                (&o.other_path, &o.other_unit, &table.axis),
            ] {
                xml.push_str(&format!("<vector><path>{}</path><unit>{}</unit><rangeMin>0</rangeMin><rangeMax>{}</rangeMax><sizeMax>{}</sizeMax>",escape(path),escape(unit),axis.last().unwrap(),axis.len()));
                for v in axis {
                    xml.push_str(&format!("<value>{v}</value>"));
                }
                xml.push_str("</vector>");
            }
            xml.push_str("</dataset>");
            // Reject metadata containing XML-forbidden characters before offering an export.
            parse_dss(&xml)?;
            Ok(xml.into_bytes())
        }
        _ => Err("Choose CSV, XLSX, or DSS.".into()),
    }
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temp.write_all(bytes)
        .and_then(|_| temp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    temp.persist(path).map_err(|e| e.error.to_string())?;
    Ok(())
}
pub fn paste_block(table: &Table, row: usize, col: usize, text: &str) -> Result<Table, String> {
    table.validate()?;
    let mut result = table.clone();
    let lines: Vec<_> = text.trim_end_matches(['\n', '\r']).split('\n').collect();
    if lines.is_empty() || text.trim().is_empty() {
        return Err("The clipboard is empty.".into());
    }
    let width = lines[0].trim_end_matches('\r').split('\t').count();
    if row
        .checked_add(lines.len())
        .filter(|n| *n <= table.rpm.len() + 1)
        .is_none()
        || col
            .checked_add(width)
            .filter(|n| *n <= table.axis.len() + 1)
            .is_none()
    {
        return Err("The pasted block exceeds the table bounds.".into());
    }
    for (i, line) in lines.iter().enumerate() {
        let cells: Vec<_> = line.trim_end_matches('\r').split('\t').collect();
        if cells.len() != width {
            return Err("The pasted block is not rectangular.".into());
        }
        for (j, cell) in cells.iter().enumerate() {
            let (r, c) = (row + i, col + j);
            if r == 0 && c == 0 {
                continue;
            }
            let value = number(&cell.replace(',', "."))?;
            if r == 0 {
                result.axis[c - 1] = value;
            } else if c == 0 {
                result.rpm[r - 1] = value;
            } else {
                result.values[r - 1][c - 1] = value;
            }
        }
    }
    Table::new(result.rpm, result.axis, result.values)
}
