/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `Info.plist`, needed for various things on Mac OS X.

use std::ptr;

#[cfg(target_os = "macos")]
#[link_section = "__TEXT,__info_plist"]
#[no_mangle]
pub static INFO_PLIST: [u8; 617] = *br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>NSSupportsAutomaticGraphicsSwitching</key>
	<true/>
	<key>CFBundleDisplayName</key>
	<string>Servo</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleName</key>
	<string>Servo</string>
	<key>NSHumanReadableCopyright</key>
	<string>Copyright &#169; 2016 The Servo Authors</string>
	<key>CFBundleVersion</key>
	<string>0.0.1</string>
	<key>CFBundleIdentifier</key>
	<string>org.servo.servo</string>
</dict>
</plist>"#;

// An unfortunate hack to make sure the dead code stripping doesn't strip our `Info.plist`.
#[cfg(target_os = "macos")]
pub fn hack_to_prevent_linker_from_stripping_the_info_plist() {
    unsafe {
        ptr::read_volatile(&INFO_PLIST[0]);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn hack_to_prevent_linker_from_stripping_the_info_plist() {}

