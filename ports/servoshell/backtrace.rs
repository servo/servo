/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Similar to `println!("{:?}", Backtrace::new())`, but doesn’t allocate.
//!
//! Seems to fix some deadlocks: <https://github.com/servo/servo/issues/24881>
//!
//! FIXME: if/when a future version of the `backtrace` crate has
//! <https://github.com/rust-lang/backtrace-rs/pull/265>, use that instead.

use std::fmt::{self, Write};

use backtrace::{BytesOrWideString, PrintFmt};

#[inline(never)]
pub(crate) fn print(w: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
    write!(
        w,
        "{:?}",
        Print {
            print_fn_address: print as usize,
        }
    )
}

#[cfg(target_env = "ohos")]
pub(crate) fn print_ohos() {
    // Print to `hilog`
    log::error!(
        "{:?}",
        Print {
            print_fn_address: print as usize,
        }
    )
}

struct Print {
    print_fn_address: usize,
}

impl fmt::Debug for Print {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Safety: we’re in a signal handler that is about to call `libc::_exit`.
        // Potential data races from using `*_unsynchronized` functions are perhaps
        // less bad than potential deadlocks?
        unsafe {
            let mut print_fn_frame = 0;
            let mut frame_count = 0;
            backtrace::trace_unsynchronized(|frame| {
                let found = frame.symbol_address() as usize == self.print_fn_address;
                if found {
                    print_fn_frame = frame_count;
                }
                frame_count += 1;
                !found
            });

            let mode = PrintFmt::Short;
            let mut p = print_path;
            let mut f = backtrace::BacktraceFmt::new(fmt, mode, &mut p);
            f.add_context()?;
            let mut result = Ok(());
            let mut frame_count = 0;
            backtrace::trace_unsynchronized(|frame| {
                let skip = frame_count < print_fn_frame;
                frame_count += 1;
                if skip {
                    return true;
                }

                let mut frame_fmt = f.frame();
                let mut any_symbol = false;
                backtrace::resolve_frame_unsynchronized(frame, |symbol| {
                    any_symbol = true;
                    if let Err(e) = frame_fmt.symbol(frame, symbol) {
                        result = Err(e)
                    }
                });
                if !any_symbol {
                    if let Err(e) = frame_fmt.print_raw(frame.ip(), None, None, None) {
                        result = Err(e)
                    }
                }
                result.is_ok()
            });
            result?;
            f.finish()
        }
    }
}

fn print_path(fmt: &mut fmt::Formatter<'_>, path: BytesOrWideString<'_>) -> fmt::Result {
    match path {
        BytesOrWideString::Bytes(mut bytes) => loop {
            match std::str::from_utf8(bytes) {
                Ok(s) => {
                    fmt.write_str(s)?;
                    break;
                },
                Err(err) => {
                    fmt.write_char(std::char::REPLACEMENT_CHARACTER)?;
                    match err.error_len() {
                        Some(len) => bytes = &bytes[err.valid_up_to() + len..],
                        None => break,
                    }
                },
            }
        },
        BytesOrWideString::Wide(wide) => {
            for c in std::char::decode_utf16(wide.iter().cloned()) {
                fmt.write_char(c.unwrap_or(std::char::REPLACEMENT_CHARACTER))?
            }
        },
    }
    Ok(())
}
