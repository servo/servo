// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    super method calls in object literal setters
---*/
var proto = {
  _x: 0,
  set x(v) {
    return this._x = v;
  }
};

var object = {
  set x(v) {
    super.x = v;
  }
};

Object.setPrototypeOf(object, proto);

assert.sameValue(object.x = 1, 1, "`object.x = 1` is `1`, after executing `Object.setPrototypeOf(object, proto);`");
assert.sameValue(object._x, 1, "The value of `object._x` is `1`, after executing `Object.setPrototypeOf(object, proto);`");
assert.sameValue(
  Object.getPrototypeOf(object)._x,
  0,
  "The value of `Object.getPrototypeOf(object)._x` is `0`"
);
