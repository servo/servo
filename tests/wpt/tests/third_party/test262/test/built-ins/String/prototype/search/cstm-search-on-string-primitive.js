// Copyright (C) 2025 Luca Casonato. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.search
description: >
  If a searchValue is a string primitive, its Symbol.search property is not accessed.
info: |
  String.prototype.search ( searchValue )

  [...]
  2. If searchValue is not Object, then
    [...]
  [...]

features: [Symbol.search]
---*/

Object.defineProperty(String.prototype, Symbol.search, {
  get: function() {
    throw new Test262Error("should not be called");
  },
});

var searchValue = ",";

const searched = "a,b,c".search(searchValue);
assert.sameValue(searched, 1);
