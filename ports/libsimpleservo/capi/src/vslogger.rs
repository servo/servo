use log::{self, Level, Metadata, Record};

extern {
    fn OutputDebugStringA(s: *const u8);
}

pub struct VSLogger;

impl log::Log for VSLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log = format!("RUST: {} - {}\r\n\0", record.level(), record.args());
            unsafe { OutputDebugStringA(log.as_ptr()); };
        }
    }

    fn flush(&self) {}
}
