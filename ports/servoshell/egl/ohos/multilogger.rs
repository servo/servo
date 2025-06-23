/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//  Code from [https://github.com/davechallis/multi_log] originally under MIT.

/// Logger that writes log messages to all the loggers it encapsulates.
pub struct MultiLogger {
    loggers: Vec<Box<dyn log::Log>>,
}

impl MultiLogger {
    /// Creates a MultiLogger from any number of other loggers.
    ///
    /// Once initialised, this will need setting as the
    /// [`log`](https://docs.rs/log/0.4.1/log/) crate's global logger using
    /// [`log::set_boxed_logger`](https://docs.rs/log/0.4.1/log/fn.set_boxed_logger.html).
    pub(crate) fn new(loggers: Vec<Box<dyn log::Log>>) -> Self {
        MultiLogger { loggers }
    }

    /// Initialises the [`log`](https://docs.rs/log/0.4.1/log/) crate's global logging facility
    /// with a MultiLogger built from any number of given loggers.
    ///
    /// The log level threshold of individual loggers can't always be determined, so a `level`
    /// parameter is provided as an optimisation to avoid sending unnecessary messages to
    /// loggers that will discard them.
    ///
    /// # Arguments
    /// * `loggers` - one more more boxed loggers
    /// * `level` - minimum log level to send to all loggers
    pub(crate) fn init(
        loggers: Vec<Box<dyn log::Log>>,
        level: log::Level,
    ) -> Result<(), log::SetLoggerError> {
        log::set_max_level(level.to_level_filter());
        log::set_boxed_logger(Box::new(MultiLogger::new(loggers)))
    }
}

impl log::Log for MultiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.loggers.iter().any(|logger| logger.enabled(metadata))
    }

    fn log(&self, record: &log::Record) {
        self.loggers.iter().for_each(|logger| logger.log(record));
    }

    fn flush(&self) {
        self.loggers.iter().for_each(|logger| logger.flush());
    }
}
