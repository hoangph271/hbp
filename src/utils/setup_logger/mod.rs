use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};

pub fn setup_logger() {
    let colors = ColoredLevelConfig::new().info(Color::Blue);

    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                record.target(),
                colors.color(record.level()),
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        // ? Use this if you need an output.log file, skip for now
        // .chain(fern::log_file("output.log"))
        .apply()
        .unwrap_or_else(|e| panic!("setup_logger() failed: {e}"));
}
