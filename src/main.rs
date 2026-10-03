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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_title("NPP-SIM — Симулятор управления АЭС (ВВЭР-1000)")
            .with_icon(std::sync::Arc::new(app_icon())),
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

/// Baked-in app icon: black square with a thin frame (matches the original).
fn app_icon() -> egui::IconData {
    const S: usize = 32;
    let mut rgba = Vec::with_capacity(S * S * 4);
    for y in 0..S {
        for x in 0..S {
            let border = x == 0 || y == 0 || x == S - 1 || y == S - 1;
            let inner = x == 2 || y == 2 || x == S - 3 || y == S - 3;
            let (r, g, b) = if border {
                (0x4c, 0xc9, 0xf0) // cyan frame
            } else if inner {
                (0x22, 0x30, 0x3d) // dark inner frame
            } else {
                (0x0a, 0x0e, 0x12) // near-black body
            };
            rgba.extend_from_slice(&[r, g, b, 0xff]);
        }
    }
    egui::IconData {
        width: S as u32,
        height: S as u32,
        rgba,
    }
}
