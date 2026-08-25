// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map-iterable
description: >
  new Map returns abrupt from getting Map.prototype.set.
info: |
  Map ( [ iterable ] )

  ...
  7. Else,
    a. Let adder be Get(map, "add").
    b. ReturnIfAbrupt(adder).
---*/

Object.defineProperty(Map.prototype, 'set', {
  get: function() {
    throw new Test262Error();
  }
});

new Map();

assert.throws(Test262Error, function() {
  new Map([]);
});
