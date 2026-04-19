// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.18
description: Object.setPrototypeOf invoked with an invalid prototype value
info: |
    1. Let O be RequireObjectCoercible(O).
    2. ReturnIfAbrupt(O).
    3. If Type(proto) is neither Object nor Null, throw a TypeError exception.
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  Object.setPrototypeOf({});
});

assert.throws(TypeError, function() {
  Object.setPrototypeOf({}, undefined);
});

assert.throws(TypeError, function() {
  Object.setPrototypeOf({}, true);
});

assert.throws(TypeError, function() {
  Object.setPrototypeOf({}, 1);
});

assert.throws(TypeError, function() {
  Object.setPrototypeOf({}, 'string');
});

assert.throws(TypeError, function() {
  Object.setPrototypeOf({}, Symbol('s'));
});
