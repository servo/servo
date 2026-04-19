// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExp.prototype[@@split] shouldn't use optimized path if limit is not number.
info: bugzilla.mozilla.org/show_bug.cgi?id=1287525
esid: pending
---*/

var rx = /a/;
var r = rx[Symbol.split]("abba", {valueOf() {
  RegExp.prototype.exec = () => null;
  return 100;
}});
assert.sameValue(r.length, 1);
