//! NPP-SIM — VVER-1000 nuclear power plant control simulator (Rust/egui port).

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(flag) = args.get(1).map(|s| s.as_str()) {
        match flag {
            "--version" | "-v" => {
                println!(
                    "NPP-SIM {} — симулятор управления АЭС (Rust/egui, портативная версия)",
                    env!("CARGO_PKG_VERSION")
                );
                return;
            }
            "--selftest" => {
                std::process::exit(nppsim::core::selftest::main_selftest());
            }
            _ => {}
        }
    }
    gui();
}

#[cfg(not(target_arch = "wasm32"))]
fn gui() {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1360.0, 860.0])
        .with_title("NPP-SIM — Симулятор управления АЭС (ВВЭР-1000)");
    if let Ok(icon) =
        eframe::icon_data::from_png_bytes(include_bytes!("../resources/app-icon.png").as_slice())
    {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    let result = eframe::run_native(
        "NPP-SIM",
        options,
        Box::new(|cc| Ok(Box::new(nppsim::ui::NppApp::new(cc)))),
    );
    if let Err(e) = result {
        eprintln!("NPP-SIM: app error: {e}");
        std::process::exit(1);
    }
}
