// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: Return abrupt getting index properties
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
  ...
features: [Array.prototype.includes]
---*/

var stopped = 0;

var obj = {
  length: 3
};

Object.defineProperty(obj, "1", {
  get: function() {
    throw new Test262Error();
  }
});

Object.defineProperty(obj, "2", {
  get: function() {
    stopped++;
  }
});

assert.throws(Test262Error, function() {
  [].includes.call(obj, 7);
});

assert.sameValue(stopped, 0, "It stops the loop after the abrupt completion");
