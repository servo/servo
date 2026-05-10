// Copyright (C) 2025 Luca Casonato. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  If a searchValue is a number primitive, its Symbol.replace property is not accessed.
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  [...]
  2. If searchValue is not Object, then
    [...]
  [...]

features: [Symbol.replace]
---*/

Object.defineProperty(Number.prototype, Symbol.replace, {
  get: function() {
    throw new Test262Error("should not be called");
  },
});

var searchValue = 1;

const replaced = "a1b1c".replaceAll(searchValue, "X");
assert.sameValue(replaced, "aXbXc");
