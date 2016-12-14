/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gaol::profile::{Operation, PathPattern, Profile};
use servo_config::resource_files;
use std::path::PathBuf;

/// Our content process sandbox profile on Mac. As restrictive as possible.
#[cfg(target_os = "macos")]
pub fn content_process_sandbox_profile() -> Profile {
    use gaol::platform;
    Profile::new(vec![
        Operation::FileReadAll(PathPattern::Literal(PathBuf::from("/dev/urandom"))),
        Operation::FileReadAll(PathPattern::Subpath(resource_files::resources_dir_path()
                                                    .expect("Cannot find resource dir"))),
        Operation::FileReadAll(PathPattern::Subpath(PathBuf::from("/Library/Fonts"))),
        Operation::FileReadAll(PathPattern::Subpath(PathBuf::from("/System/Library/Fonts"))),
        Operation::FileReadAll(PathPattern::Subpath(PathBuf::from(
                    "/System/Library/Frameworks/ApplicationServices.framework"))),
        Operation::FileReadAll(PathPattern::Subpath(PathBuf::from(
                    "/System/Library/Frameworks/CoreGraphics.framework"))),
        Operation::FileReadMetadata(PathPattern::Literal(PathBuf::from("/"))),
        Operation::FileReadMetadata(PathPattern::Literal(PathBuf::from("/Library"))),
        Operation::FileReadMetadata(PathPattern::Literal(PathBuf::from("/System"))),
        Operation::FileReadMetadata(PathPattern::Literal(PathBuf::from("/etc"))),
        Operation::SystemInfoRead,
        Operation::PlatformSpecific(platform::macos::Operation::MachLookup(
                b"com.apple.FontServer".to_vec())),
    ]).expect("Failed to create sandbox profile!")
}

/// Our content process sandbox profile on Linux. As restrictive as possible.
#[cfg(not(target_os = "macos"))]
pub fn content_process_sandbox_profile() -> Profile {
    Profile::new(vec![
        Operation::FileReadAll(PathPattern::Literal(PathBuf::from("/dev/urandom"))),
        Operation::FileReadAll(PathPattern::Subpath(resource_files::resources_dir_path()
                                                    .expect("Cannot find resource dir"))),
    ]).expect("Failed to create sandbox profile!")
}

