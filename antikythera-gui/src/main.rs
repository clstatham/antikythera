use eframe::egui;

pub mod app;

const INITIAL_SIZE: (f32, f32) = (1200.0, 800.0);

fn main() -> eframe::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let options = eframe::NativeOptions {
        window_builder: Some(Box::new(move |wb| {
            wb.with_inner_size(egui::Vec2::new(INITIAL_SIZE.0, INITIAL_SIZE.1))
                .with_min_inner_size(egui::Vec2::new(INITIAL_SIZE.0 / 2.0, INITIAL_SIZE.1 / 2.0))
        })),
        ..Default::default()
    };
    eframe::run_native(
        "Antikythera GUI",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )
}
