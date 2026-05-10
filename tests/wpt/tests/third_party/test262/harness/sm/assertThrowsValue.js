/* -*- indent-tabs-mode: nil; js-indent-level: 2 -*- */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*---
defines: [assertThrowsValue]
---*/

function assertThrowsValue(f, val, msg) {
  try {
    f();
  } catch (exc) {
    assert.sameValue(exc, val, msg);
    return;
  }

  var fullmsg = "Assertion failed: expected exception, no exception thrown";
  if (msg !== void 0) {
    fullmsg += " - " + msg;
  }
  throw new Test262Error(fullmsg);
}
