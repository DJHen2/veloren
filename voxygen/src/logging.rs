use fern::colors::{Color, ColoredLevelConfig};
use std::fs;

use crate::settings::Settings;

#[allow(clippy::or_fun_call)] // TODO: Pending review in #587
pub fn init(
    settings: &Settings,
    term_log_level: log::LevelFilter,
    file_log_level: log::LevelFilter,
) {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::Green)
        .trace(Color::BrightBlack);

    let mut base = fern::Dispatch::new()
        .level_for("dot_vox::parser", log::LevelFilter::Warn)
        .level_for("gfx_device_gl::factory", log::LevelFilter::Warn)
        .level_for("uvth", log::LevelFilter::Warn)
        .level_for("tiny_http", log::LevelFilter::Warn);

    let time = chrono::offset::Utc::now();

    let mut file_cfg =
        fern::Dispatch::new()
            .level(file_log_level)
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}:{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record
                        .line()
                        .map(|x| x.to_string())
                        .unwrap_or("X".to_string()),
                    record.level(),
                    message
                ))
            });

    // Try to create the logs file parent directories.
    let mut log_file_created = fs::create_dir_all(&settings.log.logs_path);

    if log_file_created.is_ok() {
        // Try to create the log file.
        match fern::log_file(
            settings
                .log
                .logs_path
                .join(&format!("voxygen-{}.log", time.format("%Y-%m-%d-%H"))),
        ) {
            Ok(log_file) => file_cfg = file_cfg.chain(log_file),
            Err(e) => log_file_created = Err(e),
        }
    }

    let stdout_cfg = fern::Dispatch::new()
        .level(term_log_level)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout());

    if settings.log.log_to_file {
        base = base.chain(file_cfg);
    }
    base.chain(stdout_cfg)
        .apply()
        .expect("Failed to setup logging!");

    // Incase that the log file creation failed simply print it to the console
    if let Err(e) = log_file_created {
        log::error!("Failed to create log file! {}", e);
    }
}
