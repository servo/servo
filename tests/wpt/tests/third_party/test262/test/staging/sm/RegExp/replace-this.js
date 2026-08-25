// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExp.prototype[@@replace] should check |this| value.
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

for (var v of [null, 1, true, undefined, "", Symbol.iterator]) {
  assert.throws(TypeError, () => RegExp.prototype[Symbol.replace].call(v));
}
