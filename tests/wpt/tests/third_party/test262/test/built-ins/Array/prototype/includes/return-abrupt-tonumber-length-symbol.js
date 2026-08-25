// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: Return abrupt from ToNumber(symbol "length")
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
features: [Symbol, Array.prototype.includes]
---*/

var obj = {
  length: Symbol("1")
};

assert.throws(TypeError, function() {
  [].includes.call(obj, 7);
});
