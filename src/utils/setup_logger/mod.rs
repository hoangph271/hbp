pub fn setup_logger() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                record.target(),
                record.level(),
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        // ? Use this if you need an output.log file, skip for now
        // .chain(fern::log_file("output.log"))
        .apply()
        .expect("setup_logger() failed...!");
}
