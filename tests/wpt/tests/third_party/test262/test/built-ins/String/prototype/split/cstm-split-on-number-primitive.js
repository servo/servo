// Copyright (C) 2025 Luca Casonato. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.split
description: >
  If a separator is a number primitive, its Symbol.split property is not accessed.
info: |
  String.prototype.split ( separator, limit )

  [...]
  2. If separator is not Object, then
    [...]
  [...]

includes: [compareArray.js]
features: [Symbol.split]
---*/

Object.defineProperty(Number.prototype, Symbol.split, {
  get: function() {
    throw new Test262Error("should not be called");
  },
});

var separator = 1;

assert.compareArray("a1b1c".split(separator), ["a", "b", "c"]);
assert.compareArray("a1b1c".split(separator, 1), ["a"]);
