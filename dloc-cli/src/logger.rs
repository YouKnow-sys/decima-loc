use std::io::{stderr, stdout, BufWriter, StderrLock, StdoutLock, Write};

use dloc_core::logger::*;

use crate::LogLevel;

pub struct CliLogger {
    pub stdout: BufWriter<StdoutLock<'static>>,
    stderr: BufWriter<StderrLock<'static>>,
    log_level: LogLevel,
}

pub struct CliProgress<'a> {
    stdout: &'a mut BufWriter<StdoutLock<'static>>,
    log_level: LogLevel,
    finished: Option<bool>,
    len: usize,
    current: usize,
    title: String,
}

impl CliLogger {
    pub fn new(log_level: LogLevel) -> Self {
        Self {
            stdout: BufWriter::with_capacity(5, stdout().lock()),
            stderr: BufWriter::with_capacity(5, stderr().lock()),
            log_level,
        }
    }
}

impl Logger for CliLogger {
    type Progress<'a> = CliProgress<'a>;

    fn create_progress(&mut self, title: String, len: usize) -> Self::Progress<'_> {
        write!(&mut self.stdout, "\u{001B}[?25l").expect("Can't write into stdout"); // hide console cursor
        Self::Progress {
            stdout: &mut self.stdout,
            log_level: self.log_level,
            finished: (self.log_level == LogLevel::A).then_some(false),
            len,
            current: 0,
            title,
        }
    }

    fn info(&mut self, str: impl AsRef<str>) {
        if !matches!(self.log_level, LogLevel::A | LogLevel::P) {
            return;
        }

        writeln!(&mut self.stdout, "[?]: {}", str.as_ref()).expect("Can't write into stdout");
        self.stdout.flush().expect("Can't flush stdout");
    }

    fn good(&mut self, str: impl AsRef<str>) {
        if !matches!(self.log_level, LogLevel::A | LogLevel::P) {
            return;
        }

        writeln!(&mut self.stdout, "[+]: {}", str.as_ref()).expect("Can't write into stdout");
        self.stdout.flush().expect("Can't flush stdout");
    }

    fn warn(&mut self, str: impl AsRef<str>) {
        if matches!(self.log_level, LogLevel::E | LogLevel::N) {
            return;
        }

        writeln!(&mut self.stdout, "[!]: {}", str.as_ref()).expect("Can't write into stdout");
        self.stdout.flush().expect("Can't flush stdout");
    }

    fn error(&mut self, str: impl AsRef<str>) {
        if self.log_level == LogLevel::N {
            return;
        }

        writeln!(&mut self.stderr, "[-]: {}", str.as_ref()).expect("Can't write into stdout");
        self.stderr.flush().expect("Can't flush stderr");
    }
}

impl<'a> Progress<'a> for CliProgress<'a> {
    fn add_progress(&mut self) {
        if self.log_level != LogLevel::A {
            return;
        }

        self.current += 1;
        self.print_progress();
    }

    fn end_progress(&mut self) {
        if self.log_level != LogLevel::A {
            return;
        }

        writeln!(&mut self.stdout, "\u{001B}[?25h").expect("Can't write into stdout"); // show console cursor + newline
        self.stdout.flush().expect("Can't flush stdout");
        self.finished = Some(true);
    }
}

impl<'a> CliProgress<'a> {
    const BAR_LEN: usize = 50;

    fn print_progress(&mut self) {
        let filled_up_length = Self::BAR_LEN * self.current / self.len;
        let percentage = 100 * self.current / self.len;

        write!(
            &mut self.stdout,
            "\r[P]: {title} [{fill}{remain}] {percentage:03}% ({current}/{len})",
            title = self.title,
            current = self.current,
            len = self.len,
            fill = "#".repeat(filled_up_length),
            remain = "-".repeat(Self::BAR_LEN - filled_up_length),
        )
        .expect("Can't write into stdout");
    }
}

impl<'a> Drop for CliProgress<'a> {
    fn drop(&mut self) {
        if !self.finished.is_some_and(|b| b) {
            self.end_progress();
        }
    }
}

impl Drop for CliLogger {
    fn drop(&mut self) {
        self.stdout.flush().expect("Can't flush stdout");
        self.stderr.flush().expect("Can't flush stderr");
    }
}
