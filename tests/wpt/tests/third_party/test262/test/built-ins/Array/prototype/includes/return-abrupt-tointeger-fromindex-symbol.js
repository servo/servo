// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: Return abrupt from ToInteger(fromIndex) - using symbol
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  4. Let n be ? ToInteger(fromIndex). (If fromIndex is undefined, this step
  produces the value 0.)
  ...
features: [Symbol, Array.prototype.includes]
---*/

var fromIndex = Symbol("1");

var sample = [7];

assert.throws(TypeError, function() {
  sample.includes(7, fromIndex);
});
