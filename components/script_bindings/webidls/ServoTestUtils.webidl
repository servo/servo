/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// skip-unless CARGO_FEATURE_TESTBINDING

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.

[Exposed=(Window,Worker), Pref="dom_servo_helpers_enabled"]
namespace ServoTestUtils {
  [Exposed=Window, Pref="layout_animations_test_enabled"]
  undefined advanceClock(long millis);

  undefined crashHard();

  [Exposed=Window]
  LayoutResult forceLayout();

  undefined js_backtrace();

  undefined panic();
};

[Exposed=Window, Pref="dom_servo_helpers_enabled"]
interface LayoutResult {
    readonly attribute /* FrozenArray<DOMString> */ any phases;
    readonly attribute unsigned long rebuiltFragmentCount;
    readonly attribute unsigned long restyleFragmentCount;
};
