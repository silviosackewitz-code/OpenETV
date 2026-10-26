use openetv::{formats::*, model::Table};
fn table() -> Table {
    Table::new(
        vec![4000., 8000.],
        vec![0., 50., 100.],
        vec![vec![0., 30.5, 100.], vec![0., 42.25, 100.]],
    )
    .unwrap()
}
#[test]
fn csv_sort_and_decimal_comma() {
    let t = read_csv("\u{feff}RPM;12,5;0\n8000;30;-4,5\n4000;20;-2\n\n").unwrap();
    assert_eq!(t.rpm, vec![4000., 8000.]);
    assert_eq!(t.axis, vec![0., 12.5]);
    assert_eq!(t.values, vec![vec![-2., 20.], vec![-4.5, 30.]]);
    assert_eq!(read_csv(&to_csv(&table()).unwrap()).unwrap(), table());
}
#[test]
fn roundtrip_all_formats() {
    let directory = tempfile::tempdir().unwrap();
    for ext in ["csv", "xlsx", "dss"] {
        let path = directory.path().join(format!("table.{ext}"));
        atomic_write(
            &path,
            &export_bytes(&table(), ext, &DssOptions::default()).unwrap(),
        )
        .unwrap();
        assert_eq!(read_file(&path).unwrap()[0].table, table());
    }
}
#[test]
fn dss_orientation_multiple_and_units() {
    let vectors = "<vector><path>gas</path><unit>%</unit><value>0</value><value>100</value></vector><vector><path>rpm</path><unit>1/min</unit><value>4000</value><value>8000</value></vector>";
    let t = "<table_3d><path>T</path><unit>Nm</unit><AxeX>gas</AxeX><AxeY>rpm</AxeY><row><value>1</value><value>2</value></row><row><value>3</value><value>4</value></row></table_3d>";
    let parsed = parse_dss(&format!(
        "<dataset>{vectors}{t}{}</dataset>",
        t.replace("<path>T</path>", "<path>T2</path>")
    ))
    .unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].table.values, vec![vec![1., 2.], vec![3., 4.]]);
    assert_eq!(parsed[0].rpm_path, "rpm");
    parsed[0].require_torque().unwrap();
    let xml =
        String::from_utf8(export_bytes(&table(), "dss", &DssOptions::default()).unwrap()).unwrap();
    assert!(parse_dss(&xml).unwrap()[0].require_torque().is_err());
}
#[test]
fn malformed_files_return_errors() {
    for text in [
        "",
        "RPM,0,50\n",
        "RPM,0,50\n4000,1\n",
        "RPM,0,50\n4000,1,\n",
        "RPM,0,0\n4000,1,2\n",
        "RPM,0\n4000,NaN\n",
    ] {
        assert!(read_csv(text).is_err(), "{text}");
    }
    for xml in [
        "",
        "<dataset/>",
        "<dataset><table_3d><path>T</path><AxeX>x</AxeX><AxeY>y</AxeY><row><value>1</value></row></table_3d></dataset>",
        "<dataset><vector><path>x</path><value>oops</value></vector></dataset>",
    ] {
        assert!(parse_dss(xml).is_err());
    }
    let bytes = export_bytes(&table(), "dss", &DssOptions::default()).unwrap();
    let xml = String::from_utf8(bytes).unwrap();
    assert!(parse_dss(&xml.replacen("<value>0</value>", "", 1)).is_err());
}
#[test]
fn xml_metadata_is_escaped() {
    let options = DssOptions {
        table_path: "T<&\"".into(),
        ..DssOptions::default()
    };
    let xml = String::from_utf8(export_bytes(&table(), "dss", &options).unwrap()).unwrap();
    assert_eq!(parse_dss(&xml).unwrap()[0].path, options.table_path);
}
#[test]
fn paste_is_atomic_validated_and_sorts_headers() {
    let original = table();
    let pasted = paste_block(&original, 1, 1, "2\t3\r\n4\t5\r\n").unwrap();
    assert_eq!(pasted.values[0], vec![2., 3., 100.]);
    assert!(paste_block(&original, 1, 1, "2\tbad").is_err());
    assert!(paste_block(&original, 0, 1, "0\t0").is_err());
    assert!(paste_block(&original, 2, 3, "1\t2").is_err());
    assert!(paste_block(&original, usize::MAX, 1, "1").is_err());
    assert_eq!(original, table());
    let sorted = paste_block(&original, 0, 1, "100\t50\t0").unwrap();
    assert_eq!(sorted.values[0], vec![100., 30.5, 0.]);
}
#[test]
fn windows1252_and_failed_save_preserve_original() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("table.csv");
    std::fs::write(&path, b"R\xe9gime;0\n4000;2,5\n").unwrap();
    assert_eq!(read_file(&path).unwrap()[0].table.values, vec![vec![2.5]]);
    atomic_write(&path, b"replacement").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"replacement");
    let existing = dir.path().join("existing");
    std::fs::create_dir(&existing).unwrap();
    std::fs::write(existing.join("keep"), b"safe").unwrap();
    assert!(atomic_write(&existing, b"bad").is_err());
    assert_eq!(std::fs::read(existing.join("keep")).unwrap(), b"safe");
}
#[test]
fn oversized_imports_are_rejected_before_table_allocation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("large.csv");
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(16 * 1024 * 1024 + 1).unwrap();
    assert!(read_file(&path).unwrap_err().contains("16 MB"));
    let text = format!(
        "RPM,0\n{}",
        (0..20_001).map(|i| format!("{i},1\n")).collect::<String>()
    );
    assert!(read_csv(&text).unwrap_err().contains("20,000"));
    let mut book = rust_xlsxwriter::Workbook::new();
    book.add_worksheet().write_number(1_000_000, 0, 1.).unwrap();
    let path = dir.path().join("sparse.xlsx");
    std::fs::write(&path, book.save_to_buffer().unwrap()).unwrap();
    assert!(read_file(&path).is_err());
}
#[test]
fn expanded_excel_archive_is_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bomb.xlsx");
    let mut book = rust_xlsxwriter::Workbook::new();
    book.add_worksheet().write_string(0, 0, "RPM").unwrap();
    book.save(&path).unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let start = bytes.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
    bytes[start + 24..start + 28].copy_from_slice(&(65u32 * 1024 * 1024).to_le_bytes());
    std::fs::write(&path, bytes).unwrap();
    assert!(read_file(&path).unwrap_err().contains("64 MB"));
}
#[test]
fn original_ecu_exports_when_reference_is_available() {
    let Some(root) = std::env::var_os("OPENETV_REFERENCE") else {
        return;
    };
    let mut root = std::path::PathBuf::from(root);
    if root.join(".reference").is_dir() {
        root = root.join(".reference");
    }
    // Private calibration values are loaded only at runtime and never become fixtures.
    let engine =
        read_file(&root.join("Enginetorque .dss")).expect("OPENETV_REFERENCE engine export");
    let demand =
        read_file(&root.join("torque demand.dss")).expect("OPENETV_REFERENCE demand export");
    assert!(!engine.is_empty());
    assert!(!demand.is_empty());
    for t in &engine {
        t.require_torque().expect("Original engine must be torque");
    }
    let temporary = tempfile::tempdir().unwrap();
    for (i, t) in engine.iter().chain(&demand).enumerate() {
        t.table.validate().unwrap();
        let options = DssOptions {
            table_path: t.path.clone(),
            table_unit: t.unit.clone(),
            rpm_path: t.rpm_path.clone(),
            rpm_unit: t.rpm_unit.clone(),
            other_path: t.other_path.clone(),
            other_unit: t.other_unit.clone(),
            ..Default::default()
        };
        for extension in ["csv", "xlsx", "dss"] {
            let path = temporary.path().join(format!("roundtrip-{i}.{extension}"));
            atomic_write(&path, &export_bytes(&t.table, extension, &options).unwrap()).unwrap();
            let roundtrip = read_file(&path).unwrap();
            assert!(
                roundtrip[0].table == t.table,
                "Original export round trip differs"
            );
            if extension == "dss" {
                assert_eq!(roundtrip[0].rpm_path, t.rpm_path);
                assert_eq!(roundtrip[0].other_path, t.other_path);
            }
        }
    }
    for t in demand {
        assert_eq!(t.unit.trim(), "%");
        assert!(
            t.require_torque().is_err(),
            "Throttle export must be refused as torque"
        );
    }
}
