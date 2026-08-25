// Copyright (C) 2025 Luca Casonato. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.split
description: >
  If a separator is a string primitive, its Symbol.split property is not accessed.
info: |
  String.prototype.split ( separator, limit )

  [...]
  2. If separator is not Object, then
    [...]
  [...]

includes: [compareArray.js]
features: [Symbol.split]
---*/

Object.defineProperty(String.prototype, Symbol.split, {
  get: function() {
    throw new Test262Error("should not be called");
  },
});

var separator = ",";

assert.compareArray("a,b,c".split(separator), ["a", "b", "c"]);
assert.compareArray("a,b,c".split(separator, 1), ["a"]);
