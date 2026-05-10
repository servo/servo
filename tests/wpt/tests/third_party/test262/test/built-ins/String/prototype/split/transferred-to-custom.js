// Copyright (C) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.split
description: >
    split method can be "transferred" to another object
    whose this value can be coerced to a string.
info: |
    String.prototype.split(separator, limit):

    Let O be ? RequireObjectCoercible(this value).
    ...
    Let S be ? ToString(O).

includes: [compareArray.js]
---*/


function Splittable(value) {
  this.toString = function() {
    return value + "";
  };
  this.valueOf = function() {
    throw new Test262Error();
  };
}

Splittable.prototype.split = String.prototype.split;

let splittable = new Splittable(void 0);

assert.compareArray(splittable.split(""), ["u","n","d","e","f","i","n","e","d"]);
