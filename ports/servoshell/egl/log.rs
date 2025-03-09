/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Helper Module to redirect stdout/stderr to the logging sink

use std::os::fd::{AsRawFd, IntoRawFd, RawFd};
use std::thread;

use log::{debug, error, info, warn};

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum LogRedirectError {
    CreatePipeFailed(nix::Error),
    RedirectToPipeFailed(nix::Error),
}

/// Redirect stdout and stderr to the logging system
pub(crate) fn redirect_stdout_and_stderr() -> Result<(), LogRedirectError> {
    fn log_raw_msg(raw_msg: &[u8]) {
        if let Ok(utf8_msg) = std::str::from_utf8(raw_msg) {
            debug!("{utf8_msg}");
        } else {
            // Note: This could happen if the message is long, and we hit the length
            // limitation in the middle of a utf-8 codepoint. We could try to handle this
            // by using `error.valid_up_to()`, but lets see first if we hit this problem
            // in practice.
            warn!("Dropping 1 log message due to invalid encoding.");
            debug!("Raw byte content: {raw_msg:?}")
        }
    }

    // The first step is to redirect stdout and stderr to the logs.
    // We redirect stdout and stderr to a custom descriptor.
    let (readerfd, writerfd) = nix::unistd::pipe().map_err(LogRedirectError::CreatePipeFailed)?;
    // Leaks the writer fd. We want to log for the whole program lifetime.
    let raw_writerfd = writerfd.into_raw_fd();
    let _fd = nix::unistd::dup2(raw_writerfd, RawFd::from(1))
        .map_err(LogRedirectError::RedirectToPipeFailed)?;
    let _fd = nix::unistd::dup2(raw_writerfd, RawFd::from(2))
        .map_err(LogRedirectError::RedirectToPipeFailed)?;

    // Then we spawn a thread whose only job is to read from the other side of the
    // pipe and redirect to the logs.
    let _detached = thread::spawn(move || {
        const BUF_LENGTH: usize = 512;
        let mut buf = vec![b'\0'; BUF_LENGTH];

        let mut cursor = 0_usize;

        loop {
            let result = {
                let read_into = &mut buf[cursor..];
                nix::unistd::read(readerfd.as_raw_fd(), read_into)
            };

            let end = match result {
                Ok(0) => {
                    info!("Log pipe closed. Terminating log thread");
                    return;
                },
                Ok(bytes) => bytes + cursor,
                Err(nix::errno::Errno::EINTR) => continue,
                Err(e) => {
                    error!(
                        "Failed to read from redirected stdout/stderr pipe due to {e:?}. Closing log thread"
                    );
                    return;
                },
            };

            // Only modify the portion of the buffer that contains real data.
            let buf = &mut buf[0..end];

            if let Some(last_newline_pos) = buf.iter().rposition(|&c| c == b'\n') {
                log_raw_msg(&buf[0..last_newline_pos]);

                if last_newline_pos < buf.len() {
                    let pos_after_newline = last_newline_pos + 1;
                    let len_not_logged_yet = buf[pos_after_newline..].len();
                    buf.copy_within(pos_after_newline..end, 0);
                    cursor = len_not_logged_yet;
                } else {
                    cursor = 0;
                }
            } else if end == BUF_LENGTH {
                // No newline found but the buffer is full, flush it anyway.
                log_raw_msg(buf);
                cursor = 0;
            } else {
                cursor = end;
            }
        }
    });
    Ok(())
}
