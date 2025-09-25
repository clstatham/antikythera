pub mod rules;
pub mod statistics;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Debug)
        .try_init()?;

    log::info!("Logger initialized.");

    Ok(())
}
