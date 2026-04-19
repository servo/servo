// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.fill
description: >
  Return abrupt from setting a property value.
info: |
  22.1.3.6 Array.prototype.fill (value [ , start [ , end ] ] )

  ...
  11. Repeat, while k < final
    a. Let Pk be ToString(k).
    b. Let setStatus be Set(O, Pk, value, true).
    c. ReturnIfAbrupt(setStatus).
  ...
---*/

var a1 = [];
Object.freeze(a1);

// won't break on an empty array.
a1.fill(1);

var a2 = {
  length: 1
};
Object.defineProperty(a2, '0', {
  set: function() {
    throw new Test262Error();
  }
})
assert.throws(Test262Error, function() {
  Array.prototype.fill.call(a2);
});
