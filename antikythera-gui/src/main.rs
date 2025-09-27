pub mod app;

fn main() -> eframe::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Antikythera GUI",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )
}
