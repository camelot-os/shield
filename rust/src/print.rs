// SPDX-FileCopyrightText: 2024 Ledger SAS
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use core::fmt;
use uapi::{copy_to_kernel, syscall};

// XXX for a given logger, we should support multiple sink
// e.g. __sys_log syscall, other term, file, etc.
struct LogSink;

/// Write trait impl for LogSink type
impl fmt::Write for LogSink {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let raw = s.as_bytes();
        let _ = copy_to_kernel(&raw);
        syscall::log(raw.len());
        Ok(())
    }
}

/// public print entrypoint called by macro rules
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    LogSink.write_fmt(args).expect("Print failed");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        ($crate::print::_print(format_args!($($arg)*)))
    }
}

/// println macro implementation for shield/sentry based application
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
