pub mod log {
use std::{fmt::{Display}, sync::{LazyLock, Mutex}};

use num_derive::FromPrimitive;

    pub static GLOBAL_LOGGER: LazyLock<Mutex<Logger>> = LazyLock::new(|| {
        Mutex::new(Logger::default())
    });

    pub struct Logger {
        pub output: LogOutput,
        pub show_verbose: bool,
        pub enabled_channels: u64, 
    }

    impl Logger {
        pub fn enable_channel(&mut self, channel: LogChannel) {
            self.enabled_channels |= channel as u64;
        }

        pub fn enable_all_log_channels(&mut self) {
            self.enabled_channels = 
            LogChannel::Default as u64 | 
            LogChannel::Render as u64 | 
            LogChannel::Interop as u64 |
            LogChannel::App as u64;
        }
    }

    pub struct Log {
        message: String,
        verbose: bool,
        level: LogLevel,
        channel: LogChannel,
    }

    impl Log {
        pub fn new(message: String, verbose: bool, level: LogLevel, channel: LogChannel) -> Self {
            Self {
                message,
                verbose,
                level,
                channel,
            }
        }
    }

    impl Default for Logger{
        fn default() -> Self {
            Self {
                output: LogOutput::Console,
                show_verbose: false,
                enabled_channels: 0,
            }
        }
    }

    #[repr(u64)]
    #[derive(Clone, Copy, Debug, FromPrimitive)]
    pub enum LogChannel {
        Default = 1 << 0,
        Render = 1 << 1,
        Interop = 1 << 2,
        App = 1 << 3,
    }
    impl Display for LogChannel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    pub enum LogOutput {
        Console,
    }

    pub enum LogLevel {
        Message,
        Warning,
        Error,
    }

    pub fn log(channel: LogChannel, messsage: String, ) {
        log_internal(channel, messsage, false, LogLevel::Message);
    }
    pub fn verbose_log(channel: LogChannel, messsage: String, ) {
        log_internal(channel, messsage, true, LogLevel::Message);
    }

    pub fn warning(channel: LogChannel, messsage: String, ) {
        log_internal(channel, messsage, false, LogLevel::Warning);
    }
    pub fn verbose_warning(channel: LogChannel, messsage: String, ) {
        log_internal(channel, messsage, true, LogLevel::Warning);
    }

    pub fn error(channel: LogChannel, messsage: String, ) {
        log_internal(channel, messsage, false, LogLevel::Error);
    }
    pub fn verbose_error(channel: LogChannel, messsage: String, ) {
        log_internal(channel, messsage, true, LogLevel::Error);
    }

    pub fn log_internal(channel: LogChannel, messsage: String, verbose: bool, level: LogLevel, ) {
        let log = Log::new(messsage, verbose, level, channel);
        _log_internal(&log);
    }

    pub fn _log_internal(log: &Log, ) {
        let logger = GLOBAL_LOGGER.lock().unwrap();
        if log.verbose && !logger.show_verbose {
            return;
        }
        if logger.enabled_channels & (log.channel as u64) == 0 {
            return;
        }
        let output: String;
        match log.level {
            LogLevel::Message => {
                output = format!("[Channel: {}] {}", log.channel.to_string(), log.message);
            },
            LogLevel::Warning => {
                output = format!("[Warning!, Channel: {}] {}", log.channel.to_string(), log.message);
            },
            LogLevel::Error => {
                output = format!("[Error!, Channel: {}]  {}", log.channel.to_string(), log.message);
            },
        }
        match logger.output {
            LogOutput::Console => {
                println!("{}", output);
            },
        }
    }

    pub fn init_logger() {
        
    }
}