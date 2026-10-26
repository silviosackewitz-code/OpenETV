use std::io::{self, Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version") {
        println!("OpenETV {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--batch") {
        let mut input = String::new();
        io::stdin()
            .take(16 * 1024 * 1024 + 1)
            .read_to_string(&mut input)?;
        if input.len() > 16 * 1024 * 1024 {
            return Err("Batch input exceeds 16 MB".into());
        }
        let jobs: Vec<serde_json::Value> = serde_json::from_str(&input)?;
        let results: Vec<_> = jobs
            .iter()
            .map(|v| {
                openetv::batch::execute(v)
                    .unwrap_or_else(|error| serde_json::json!({"error":error}))
            })
            .collect();
        serde_json::to_writer(io::stdout().lock(), &results)?;
        return Ok(());
    }
    if args.iter().any(|a| a == "--help") {
        println!(
            "OpenETV — native torque-to-throttle map builder\n  --version  Print version\n  --batch    JSON operations on stdin (no window)\n  --help     Show help"
        );
        return Ok(());
    }
    if !args.is_empty() {
        return Err(format!("Unknown argument: {}", args.join(" ")).into());
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("OpenETV")
            .with_app_id("com.openetv.native")
            .with_inner_size([1400., 900.])
            .with_min_inner_size([900., 600.]),
        ..Default::default()
    };
    eframe::run_native(
        "OpenETV",
        options,
        Box::new(|cc| Ok(Box::new(openetv::app::App::start(&cc.egui_ctx)))),
    )?;
    Ok(())
}
