// Copyright (C) 2025 Luca Casonato. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.match
description: >
  If a regexp property is a bigint primitive, its Symbol.match property is not accessed.
info: |
  String.prototype.match ( regexp )

  [...]
  2. If regexp is not Object, then
    [...]
  [...]

includes: [compareArray.js]
features: [Symbol.match]
---*/

Object.defineProperty(BigInt.prototype, Symbol.match, {
  get: function() {
    throw new Test262Error("should not be called");
  },
});

var separator = 1n;

const matched = "a1b1c".match(separator);
assert.sameValue(matched.index, 1);
assert.sameValue(matched.input, "a1b1c");
assert.compareArray(matched, ["1"]);
