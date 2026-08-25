// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.7
description: >
    A property cannot be reported as non-existent, if it exists as a
    non-configurable own property of the target object.
info: |
    [[HasProperty]] (P)

    ...
    11. If booleanTrapResult is false, then
        ...
        c. If targetDesc is not undefined, then
            i. If targetDesc.[[Configurable]] is false, throw a TypeError
            exception.
    ...
features: [Proxy]
---*/

var target = {};
var handler = {
  has: function(t, prop) {
    return 0;
  }
};
var p = new Proxy(target, handler);

Object.defineProperty(target, "attr", {
  configurable: false,
  value: 1
});

assert.throws(TypeError, function() {
  "attr" in p;
});
