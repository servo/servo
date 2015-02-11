/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(io)]

use std::old_io::process::{Command, ProcessExit, StdioContainer};


fn main() {
    let result = Command::new("make")
        .args(&["-f", "makefile.cargo"])
        .stdout(StdioContainer::InheritFd(1))
        .stderr(StdioContainer::InheritFd(2))
        .status()
        .unwrap();
    assert_eq!(result, ProcessExit::ExitStatus(0));
}
