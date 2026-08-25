// Copyright (C) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.split
description: >
  Calls valueOf() of limit argument
info: |
  String.prototype.split(separator, limit):

  If limit is undefined, let lim be 232 - 1; else let lim be ℝ(? ToUint32(limit)).
features: [arrow-function]
---*/


let limit = {
  toString() {},
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => {
  "".split("", limit);
});
