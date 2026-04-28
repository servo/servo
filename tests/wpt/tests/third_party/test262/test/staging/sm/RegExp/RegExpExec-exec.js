// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExpExec should throw if exec property of non-RegExp is not callable
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

for (var exec of [null, 0, false, undefined, ""]) {
  // RegExp with non-callable exec
  var re = /a/;
  re.exec = exec;
  RegExp.prototype[Symbol.match].call(re, "foo");

  // non-RegExp with non-callable exec
  assert.throws(TypeError, () => RegExp.prototype[Symbol.match].call({ exec }, "foo"));
}
