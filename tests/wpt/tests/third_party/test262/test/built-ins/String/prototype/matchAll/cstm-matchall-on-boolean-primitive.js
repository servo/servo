// Copyright (C) 2025 Luca Casonato. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.matchall
description: >
  If a regexp property is a boolean primitive, its Symbol.matchAll property is not accessed.
info: |
  String.prototype.matchAll ( regexp )

  [...]
  2. If regexp is not Object, then
    [...]
  [...]

includes: [compareArray.js]
features: [Symbol.matchAll]
---*/

Object.defineProperty(Boolean.prototype, Symbol.match, {
  get: function() {
    throw new Test262Error("should not be called");
  },
});

var matcher = true;

const matched = "atruebtruec".matchAll(matcher);
const matchesArray = Array.from(matched);
assert.sameValue(matchesArray[0].index, 1);
assert.sameValue(matchesArray[0].input, "atruebtruec");
assert.compareArray(matchesArray[0], ["true"]);
assert.sameValue(matchesArray[1].index, 6);
assert.sameValue(matchesArray[1].input, "atruebtruec");
assert.compareArray(matchesArray[1], ["true"]);
assert.sameValue(matchesArray.length, 2);
