// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExp.prototype[@@replace] should not use optimized path if RegExp.prototype.unicode is modified.
info: bugzilla.mozilla.org/show_bug.cgi?id=1287524
esid: pending
---*/

Object.defineProperty(RegExp.prototype, "unicode", {
  get() {
    RegExp.prototype.exec = () => null;
  }
});

var rx = RegExp("a", "g");
var s = "abba";
var r = rx[Symbol.replace](s, "c");
assert.sameValue(r, "abba");
