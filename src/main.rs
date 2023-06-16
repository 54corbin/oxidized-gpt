use app::App;
use chatgpt::err;
use eframe::IconData;
mod app;
mod settings;

pub const APP_NAME: &str = "Oxidized GPT";

fn config_font(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../media/PingFangSC.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

#[tokio::main]
async fn main() -> std::result::Result<(), err::Error> {
    let native_options = eframe::NativeOptions {
        icon_data: Some(IconData {
            rgba: include_bytes!("../media/chatgpt_logo.jpeg").to_vec(),
            height: 50,
            width: 50,
        }),
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| {
            config_font(&cc.egui_ctx);
            Box::new(App::new(cc, APP_NAME))
        }),
    );

    Ok(())
}
