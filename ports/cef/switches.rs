/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// values taken from chromium/base/base_switches.cc

// Disables the crash reporting.
pub static KDISABLEBREAKPAD: &'static str               = "disable-breakpad";

// Indicates that crash reporting should be enabled. On platforms where helper
// processes cannot access to files needed to make this decision, this flag is
// generated internally.
pub static KENABLECRASHREPORTER: &'static str           = "enable-crash-reporter";

// Generates full memory crash dump.
pub static KFULLMEMORYCRASHREPORT: &'static str         = "full-memory-crash-report";

// The value of this switch determines whether the process is started as a
// renderer or plugin host.  If it's empty, it's the browser.
pub static KPROCESSTYPE: &'static str                   = "type";

// Suppresses all error dialogs when present.
pub static KNOERRDIALOGS: &'static str                = "noerrdialogs";

// When running certain tests that spawn child processes, this switch indicates
// to the test framework that the current process is a child process.
pub static KTESTCHILDPROCESS: &'static str              = "test-child-process";

// Gives the default maximal active V-logging level; 0 is the default.
// Normally positive values are used for V-logging levels.
pub static KV: &'static str                             = "v";

// Gives the per-module maximal V-logging levels to override the value
// given by --v.  E.g. "my_module=2,foo*=3" would change the logging
// level for all code in source files "my_module.*" and "foo*.*"
// ("-inl" suffixes are also disregarded for this matching).
//
// Any pattern containing a forward or backward slash will be tested
// against the whole pathname and not just the module.  E.g.,
// "*/foo/bar/*=2" would change the logging level for all code in
// source files under a "foo/bar" directory.
pub static KVMODULE: &'static str                       = "vmodule";

// Will wait for 60 seconds for a debugger to come to attach to the process.
pub static KWAITFORDEBUGGER: &'static str               = "wait-for-debugger";

// Sends a pretty-printed version of tracing info to the console.
pub static KTRACETOCONSOLE: &'static str                = "trace-to-console";

// Configure whether chrome://profiler will contain timing information. This
// option is enabled by default. A value of "0" will disable profiler timing,
// while all other values will enable it.
pub static KPROFILERTIMING: &'static str                = "profiler-timing";
// Value of the --profiler-timing flag that will disable timing information for
// chrome://profiler.
pub static KPROFILERTIMINGDISABLEDVALUE: &'static str   = "0";
