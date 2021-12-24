use std::io::Write;

// This logger will write logs into a file on disk
struct FileLogger {
    level: log::Level,
    writer: std::sync::RwLock<std::io::BufWriter<std::fs::File>>,
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // Check if the logger is enabled for a certain log level
        // Here, you could also add own custom filtering based on targets or regex
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut writer = self.writer
                .write()
                .expect("Failed to unlock log file writer in write mode");
            let now = std::time::SystemTime::now();
            let timestamp = now.duration_since(std::time::UNIX_EPOCH).expect(
                "Failed to generate timestamp: This system is operating before the unix epoch",
            );
            // Write the log into the buffer
            write!(
                writer,
                "{} {} at {}: {}\n",
                record.level(),
                timestamp.as_secs(),
                record.target(),
                record.args()
            ).expect("Failed to log to file");
        }
        self.flush();
    }

    fn flush(&self) {
        // Write the buffered logs to disk
        self.writer
            .write()
            .expect("Failed to unlock log file writer in write mode")
            .flush()
            .expect("Failed to flush log file writer");
    }
}

impl FileLogger {
    // A convenience method to set everything up nicely
    fn init(level: log::Level, file_name: &str) -> Result<()> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_name)?;
        let writer = std::sync::RwLock::new(std::io::BufWriter::new(file));
        let logger = FileLogger { level, writer };
        // set the global level filter that log uses to optimize ignored logs
        log::set_max_level(level.to_level_filter());
        // set this logger as the one used by the log macros
        log::set_boxed_logger(Box::new(logger))?;
        Ok(())
    }
}

// Our custom error for our FileLogger
#[derive(Debug)]
enum FileLoggerError {
    Io(std::io::Error),
    SetLogger(log::SetLoggerError),
}

type Result<T> = std::result::Result<T, FileLoggerError>;
impl std::error::Error for FileLoggerError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            FileLoggerError::Io(ref err) => Some(err),
            FileLoggerError::SetLogger(ref err) => Some(err),
        }
    }
}

impl std::fmt::Display for FileLoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            FileLoggerError::Io(ref err) => write!(f, "IO error: {}", err),
            FileLoggerError::SetLogger(ref err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl From<std::io::Error> for FileLoggerError {
    fn from(err: std::io::Error) -> FileLoggerError {
        FileLoggerError::Io(err)
    }
}

impl From<log::SetLoggerError> for FileLoggerError {
    fn from(err: log::SetLoggerError) -> FileLoggerError {
        FileLoggerError::SetLogger(err)
    }
}

fn main() {
    FileLogger::init(log::Level::Info, "log.txt").expect("Failed to init FileLogger");
    log::trace!("Beginning the operation");
    log::info!("A lightning strikes a body");
    log::warn!("It's moving");
    log::error!("It's alive!");
    log::debug!("Dr. Frankenstein now knows how it feels to be god");
    log::trace!("End of the operation");
}
